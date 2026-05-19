use architect_tui::adapter::AdapterConfig;
use architect_tui::config::TuiConfig;
use architect_tui::interactive::InteractiveWorkflowEngine;
use architect_tui::orchestrator::{HeadlessRunOptions, Orchestrator};
use architect_tui::session::{ApprovalStatus, SessionPhase};
use serde_json::Value;
use std::process::Command;

#[test]
fn new_app_workflow_has_golden_gate_order() {
    let orchestrator = Orchestrator::new(".", TuiConfig::default());
    let workflow = orchestrator.new_app_workflow("offline recipe planner");
    let gates: Vec<_> = workflow
        .steps
        .iter()
        .map(|step| step.gate.as_str())
        .collect();
    assert_eq!(gates[0], "grill_me");
    assert!(gates.contains(&"create_pre_edit_contract"));
    assert!(gates.contains(&"review_build_plan"));
    assert!(gates.contains(&"review_proposed_file_plan"));
    assert!(gates.contains(&"review_implementation_against_contract"));
    assert_eq!(gates.last(), Some(&"review_agent_session"));
}

#[cfg(not(windows))]
#[tokio::test]
async fn headless_execute_uses_isolated_worktree_and_review_gates() {
    let Some((mut orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(_temp.path());
    std::fs::create_dir_all(_temp.path().join("docs")).expect("docs dir");
    std::fs::write(
        _temp.path().join("docs/main-dirty.md"),
        "not adapter work\n",
    )
    .expect("dirty main workspace file");
    let mut shell = orchestrator
        .config_mut()
        .adapters
        .get("shell")
        .expect("shell adapter")
        .clone();
    let writer = shell_writer_portable("docs/live-qa.md", "agent work");
    shell.command = writer.command;
    shell.args = writer.args;
    let config = orchestrator.config_mut();
    config.agents.default_adapter = "shell".to_string();
    config.adapters.insert("shell".to_string(), shell);
    let output_path = _temp.path().join("execute.jsonl");
    let mut output = tokio::fs::File::create(&output_path)
        .await
        .expect("output file");

    orchestrator
        .run_headless_to_writer(
            HeadlessRunOptions {
                prompt: "ready app with users flows stack risks verification".to_string(),
                adapter: "shell".to_string(),
                jsonl: true,
                concurrency: 1,
                execute: true,
            },
            &mut output,
        )
        .await
        .expect("headless execute");
    drop(output);

    let events = read_jsonl(&output_path);
    let calls = mcp_calls(&events);
    assert!(calls.contains(&"review_implementation_against_contract"));
    assert!(calls.contains(&"review_repo_structure"));
    assert!(calls.contains(&"review_agent_final_response"));
    assert!(calls.contains(&"review_agent_session"));
    let diff = events
        .iter()
        .find(|event| event["type"] == "diff_evidence")
        .expect("diff evidence");
    assert!(
        diff["worktree"]
            .as_str()
            .unwrap()
            .contains(".architect-mcp/worktrees")
    );
    assert_eq!(diff["changed_files"][0]["path"], "docs/live-qa.md");
    assert!(
        !diff["changed_files"]
            .as_array()
            .expect("changed files")
            .iter()
            .any(|file| file["path"] == "docs/main-dirty.md")
    );
    assert!(
        events
            .iter()
            .any(|event| event["status"] == "review_required")
    );
}

#[test]
fn arena_candidates_use_isolated_worktrees() {
    let orchestrator = Orchestrator::new("/tmp/work", TuiConfig::default());
    let candidates = orchestrator.arena_candidates("session-1", &["codex".into(), "claude".into()]);
    assert!(candidates[0].worktree.ends_with("session-1/codex"));
    assert!(candidates[1].worktree.ends_with("session-1/claude"));
}

#[tokio::test]
async fn interactive_approval_commands_cover_reject_cancel_and_failed_review() {
    let orchestrator = Orchestrator::new(".", TuiConfig::default());
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    let update = engine
        .apply_input("new app offline planner")
        .await
        .expect("new app");
    assert_eq!(
        update.session.expect("session").approval_status,
        ApprovalStatus::Pending
    );

    let too_early = engine
        .apply_input("approve review gates passed")
        .await
        .expect_err("approval too early");
    assert!(
        too_early
            .to_string()
            .contains("approval is available after file review")
    );

    let failed = engine.apply_input("promote").await.expect_err("blocked");
    assert!(
        failed
            .to_string()
            .contains("approval is required before promotion")
    );

    let update = engine
        .apply_input("reject not acceptable")
        .await
        .expect("reject");
    assert_eq!(
        update.session.expect("session").approval_status,
        ApprovalStatus::Rejected
    );

    let update = engine.apply_input("cancel").await.expect("cancel");
    assert_eq!(
        update.session.expect("session").phase,
        SessionPhase::Cancelled
    );
}

#[cfg(unix)]
#[tokio::test]
async fn interactive_adapter_execution_requires_distinct_approval() {
    let Some((mut orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(_temp.path());
    let mut shell = shell_writer("docs/approved-run.md", "agent work\\n");
    shell.available = Some(true);
    orchestrator
        .config_mut()
        .adapters
        .insert("shell".to_string(), shell);
    orchestrator.config_mut().agents.default_adapter = "shell".to_string();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready app with users flows stack risks verification")
        .await
        .expect("new app");
    engine.apply_input("grill").await.expect("grill");
    engine.apply_input("contract").await.expect("contract");
    engine.apply_input("review plan").await.expect("plan");
    engine.apply_input("review files").await.expect("files");

    let blocked = engine
        .apply_input("run adapter")
        .await
        .expect_err("execution approval required");
    assert!(
        blocked
            .to_string()
            .contains("approve adapter execution before run adapter")
    );

    let update = engine
        .apply_input("approve run isolated adapter")
        .await
        .expect("approve execution");
    let session = update.session.expect("session");
    assert!(session.execution_approved);
    assert_eq!(session.approval_status, ApprovalStatus::Pending);

    let update = engine
        .apply_input("review files")
        .await
        .expect("re-review files");
    let session = update.session.expect("session");
    assert!(!session.execution_approved);

    let blocked = engine
        .apply_input("run adapter")
        .await
        .expect_err("execution approval required after re-review");
    assert!(
        blocked
            .to_string()
            .contains("approve adapter execution before run adapter")
    );

    engine
        .apply_input("approve rerun isolated adapter")
        .await
        .expect("re-approve execution");

    let update = engine
        .apply_input("run adapter")
        .await
        .expect("run adapter");
    let session = update.session.expect("session");
    assert_eq!(session.phase, SessionPhase::ReviewRequired);
    assert!(!session.execution_approved);
    assert_eq!(session.approval_status, ApprovalStatus::Pending);
    assert!(session.worktree.is_some());
    assert!(
        session
            .changed_files
            .iter()
            .any(|file| file["path"] == "docs/approved-run.md")
    );
    assert!(
        session
            .gates
            .contains_key("review_implementation_against_contract")
    );
    assert!(session.gates.contains_key("review_agent_session"));

    let promote_blocked = engine.apply_input("promote").await.expect_err("blocked");
    assert!(
        promote_blocked
            .to_string()
            .contains("approval is required before promotion")
    );

    let approval_blocked = engine
        .apply_input("approve promote reviewed diff")
        .await
        .expect_err("approval blocked");
    assert!(
        approval_blocked
            .to_string()
            .contains("run final review and session review")
    );

    let final_blocked = engine
        .apply_input("final review Changed files: docs/approved-run.md. Verification: npm test passed. Assumptions: isolated review. Not done: promotion remains pending.")
        .await
        .expect_err("verification required");
    assert!(
        final_blocked
            .to_string()
            .contains("record passed verification")
    );

    let bad_status = engine
        .apply_input("record verification npm test=blocked")
        .await
        .expect_err("status validation");
    assert!(
        bad_status
            .to_string()
            .contains("verification status must be one of")
    );

    engine
        .apply_input("record verification npm test=failed")
        .await
        .expect("failed verification recorded");
    let still_blocked = engine
        .apply_input("final review Changed files: docs/approved-run.md. Verification: npm test passed. Assumptions: isolated review. Not done: promotion remains pending.")
        .await
        .expect_err("failed verification blocks final");
    assert!(still_blocked.to_string().contains("npm test=failed"));

    engine
        .apply_input("record verification npm test=passed")
        .await
        .expect("passed verification recorded");
    let session_before_final = engine
        .apply_input("session review")
        .await
        .expect_err("final review first");
    assert!(
        session_before_final
            .to_string()
            .contains("run final review after passed verification")
    );

    engine
        .apply_input("final review Changed files: docs/approved-run.md. Verification: npm test passed. Assumptions: isolated review. Not done: promotion remains pending.")
        .await
        .expect("final review");
    let update = engine
        .apply_input("session review")
        .await
        .expect("session review");
    let session = update.session.expect("session");
    assert_eq!(session.phase, SessionPhase::Complete);
    let review_request = &session.gates["review_agent_session"]["received"];
    assert_eq!(
        review_request["finalResponse"].as_str(),
        Some(
            "Changed files: docs/approved-run.md. Verification: npm test passed. Assumptions: isolated review. Not done: promotion remains pending."
        )
    );
    let verification = review_request["verification"]
        .as_array()
        .expect("verification array");
    assert_eq!(verification.len(), 1);
    assert_eq!(verification[0]["check"].as_str(), Some("npm test"));
    assert_eq!(verification[0]["status"].as_str(), Some("passed"));

    let update = engine
        .apply_input("approve promote reviewed diff")
        .await
        .expect("approve promotion");
    assert_eq!(
        update.session.expect("session").approval_status,
        ApprovalStatus::Approved
    );
}

#[tokio::test]
async fn interactive_flow_blocks_until_grill_is_ready_and_shows_blockers() {
    let Some((orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app build an app")
        .await
        .expect("new app");
    let missing_grill = engine.apply_input("contract").await.expect_err("blocked");
    assert!(missing_grill.to_string().contains("run grill"));

    let update = engine.apply_input("grill").await.expect("grill");
    let transcript = update.transcript.join("\n");
    assert!(transcript.contains("grill_me: needs input"));
    assert!(transcript.contains("blocker: Missing users"));
    assert!(transcript.contains("next question: Who uses it?"));
    assert_eq!(
        update.session.expect("session").phase,
        SessionPhase::IntakeBlocked
    );

    let blocked_contract = engine.apply_input("contract").await.expect_err("blocked");
    assert!(
        blocked_contract
            .to_string()
            .contains("grill_me must be ready before this command")
    );
}

#[tokio::test]
async fn interactive_ready_flow_enforces_gate_order_and_persists_resume() {
    let Some((orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let mut engine = InteractiveWorkflowEngine::new(orchestrator.clone());

    let created = engine
        .apply_input("new app ready app with users flows stack risks verification")
        .await
        .expect("new app");
    let session_id = created.session.expect("session").id;

    let adapter_blocked = engine
        .apply_input("run adapter")
        .await
        .expect_err("blocked");
    assert!(
        adapter_blocked
            .to_string()
            .contains("review files before run adapter")
    );
    engine.apply_input("grill").await.expect("grill");
    let files_before_plan = engine
        .apply_input("review files")
        .await
        .expect_err("blocked");
    assert!(
        files_before_plan
            .to_string()
            .contains("review plan before review files")
    );
    engine.apply_input("contract").await.expect("contract");
    engine.apply_input("review plan").await.expect("plan");
    let update = engine.apply_input("review files").await.expect("files");
    let session = update.session.expect("session");
    assert_eq!(session.phase, SessionPhase::FilePlanReviewed);
    assert!(session.gates.contains_key("grill_me"));
    assert!(session.gates.contains_key("create_pre_edit_contract"));
    assert!(session.gates.contains_key("review_build_plan"));
    assert!(session.gates.contains_key("review_proposed_file_plan"));

    let mut resumed = InteractiveWorkflowEngine::new(orchestrator);
    let update = resumed
        .apply_input(&format!("resume {session_id}"))
        .await
        .expect("resume");
    let session = update.session.expect("session");
    assert_eq!(session.id, session_id);
    assert_eq!(session.phase, SessionPhase::FilePlanReviewed);
    assert!(update.transcript.join("\n").contains("session resumed"));
}

#[cfg(unix)]
#[tokio::test]
async fn interactive_arena_run_records_and_ranks_multiple_candidates() {
    let Some((mut orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(_temp.path());
    let mut shell_a = shell_writer("docs/arena-a.md", "candidate a\\n");
    let mut shell_b = shell_writer("docs/arena-b.md", "candidate b\\n");
    shell_a.available = Some(true);
    shell_b.available = Some(true);
    let config = orchestrator.config_mut();
    config.adapters.insert("shell-a".to_string(), shell_a);
    config.adapters.insert("shell-b".to_string(), shell_b);
    config.agents.default_adapter = "shell-a".to_string();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready app with users flows stack risks verification")
        .await
        .expect("new app");
    engine.apply_input("grill").await.expect("grill");
    engine.apply_input("contract").await.expect("contract");
    engine.apply_input("review plan").await.expect("plan");
    engine.apply_input("review files").await.expect("files");
    let update = engine
        .apply_input("arena run shell-a,shell-b")
        .await
        .expect("arena run");
    let session = update.session.expect("session");
    assert_eq!(session.arena_candidates.len(), 2);
    assert!(session.arena_candidates.iter().all(|candidate| {
        candidate
            .worktree
            .as_deref()
            .unwrap_or("")
            .contains(".architect-mcp/worktrees")
    }));

    let update = engine.apply_input("arena rank").await.expect("arena rank");
    let transcript = update.transcript.join("\n");
    assert!(transcript.contains("candidate evidence"));
    assert!(transcript.contains("shell-a"));
    assert!(transcript.contains("shell-b"));
}

#[tokio::test]
async fn headless_vague_prompt_stops_after_live_grill_gate() {
    let Some((orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let output_path = _temp.path().join("vague.jsonl");
    let mut output = tokio::fs::File::create(&output_path)
        .await
        .expect("output file");

    orchestrator
        .run_headless_to_writer(
            HeadlessRunOptions {
                prompt: "build an app".to_string(),
                adapter: "codex".to_string(),
                jsonl: true,
                concurrency: 1,
                execute: false,
            },
            &mut output,
        )
        .await
        .expect("headless run");
    drop(output);

    let events = read_jsonl(&output_path);
    let calls = mcp_calls(&events);
    assert_eq!(calls, vec!["grill_me"]);
    assert!(
        events
            .iter()
            .any(|event| event["type"] == "approval_required")
    );
    assert!(
        events
            .iter()
            .any(|event| event["status"] == "approval_required")
    );
}

#[tokio::test]
async fn headless_ready_prompt_reaches_plan_reviews_and_skips_adapter_without_execute() {
    let Some((orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let output_path = _temp.path().join("ready.jsonl");
    let mut output = tokio::fs::File::create(&output_path)
        .await
        .expect("output file");

    orchestrator
        .run_headless_to_writer(
            HeadlessRunOptions {
                prompt: "ready app with users flows stack risks verification".to_string(),
                adapter: "codex".to_string(),
                jsonl: true,
                concurrency: 1,
                execute: false,
            },
            &mut output,
        )
        .await
        .expect("headless run");
    drop(output);

    let events = read_jsonl(&output_path);
    let calls = mcp_calls(&events);
    assert_eq!(
        calls,
        vec![
            "grill_me",
            "create_pre_edit_contract",
            "review_build_plan",
            "review_proposed_file_plan"
        ]
    );
    assert!(
        events
            .iter()
            .any(|event| event["type"] == "adapter_skipped")
    );
    assert!(
        events
            .iter()
            .any(|event| event["status"] == "approval_required")
    );
}

fn fake_mcp_orchestrator() -> Option<(Orchestrator, tempfile::TempDir)> {
    if Command::new("node").arg("--version").output().is_err() {
        return None;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let server_path = temp.path().join("fake-mcp.mjs");
    std::fs::write(&server_path, fake_mcp_server_script()).expect("fake server");
    let mut config = TuiConfig::default();
    config.architect_mcp.command = Some("node".to_string());
    config.architect_mcp.args = vec![server_path.display().to_string()];
    Some((Orchestrator::new(temp.path(), config), temp))
}

fn read_jsonl(path: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .expect("read jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).expect("json line"))
        .collect()
}

fn mcp_calls(events: &[Value]) -> Vec<&str> {
    events
        .iter()
        .filter(|event| event["type"] == "mcp_call")
        .filter_map(|event| event["name"].as_str())
        .collect()
}

fn init_git_repo(path: &std::path::Path) {
    run_git(path, ["init"]);
    run_git(path, ["config", "user.email", "test@example.com"]);
    run_git(path, ["config", "user.name", "architect mcp test"]);
    std::fs::write(path.join("README.md"), "test\n").expect("readme");
    run_git(path, ["add", "README.md"]);
    run_git(path, ["commit", "-m", "init"]);
}

#[cfg(unix)]
fn shell_writer(path: &str, content: &str) -> AdapterConfig {
    AdapterConfig {
        command: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            format!(
                "mkdir -p \"$(dirname {path})\" && printf '{}' > {path}",
                content.replace('\'', "'\\''")
            ),
        ],
        ..AdapterConfig::default()
    }
}

fn shell_writer_portable(path: &str, content: &str) -> AdapterConfig {
    #[cfg(windows)]
    {
        let path = path.replace('\'', "''");
        let content = content.replace('\'', "''");
        AdapterConfig {
            command: "pwsh".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                format!(
                    "$p='{path}'; New-Item -ItemType Directory -Force -Path (Split-Path $p) | Out-Null; Set-Content -NoNewline -Path $p -Value '{content}'"
                ),
            ],
            ..AdapterConfig::default()
        }
    }
    #[cfg(not(windows))]
    {
        AdapterConfig {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                format!(
                    "mkdir -p \"$(dirname {path})\" && printf '{}' > {path}",
                    content.replace('\'', "'\\''")
                ),
            ],
            ..AdapterConfig::default()
        }
    }
}

fn run_git<const N: usize>(path: &std::path::Path, args: [&str; N]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fake_mcp_server_script() -> &'static str {
    r#"
import readline from 'node:readline';
const rl = readline.createInterface({ input: process.stdin });
function respond(id, result) {
  console.log(JSON.stringify({ jsonrpc: '2.0', id, result }));
}
function tool(id, value) {
  respond(id, { content: [{ type: 'text', text: JSON.stringify(value) }], structuredContent: value });
}
rl.on('line', (line) => {
  const msg = JSON.parse(line);
  if (!msg.id) return;
  if (msg.method === 'initialize') {
    respond(msg.id, { protocolVersion: '2025-11-25', capabilities: {}, serverInfo: { name: 'fake', version: '0.0.0' } });
  } else if (msg.method === 'tools/list') {
    respond(msg.id, { tools: [{ name: 'grill_me' }, { name: 'create_pre_edit_contract' }, { name: 'review_build_plan' }, { name: 'review_proposed_file_plan' }] });
  } else if (msg.method === 'tools/call' && msg.params.name === 'grill_me') {
    const idea = msg.params.arguments.brief.idea;
    if (idea.startsWith('ready')) {
      tool(msg.id, {
        ready: true,
        blockers: [],
        challenges: [],
        nextQuestion: null,
        updatedBrief: {
          idea,
          stack: { frontend: 'React', backend: 'TypeScript' },
          verification: ['npm test']
        },
        contract: { name: 'Ready App' },
        buildPlan: {
          archetype: 'custom-app',
          slices: [{
            id: 'agent-harness',
            title: 'Harness',
            order: 1,
            goal: 'Confirm contract',
            inputs: ['brief'],
            outputs: ['contract'],
            allowedDirectories: ['docs'],
            forbiddenFiles: ['src/App.tsx'],
            files: ['docs/architecture-contract.md'],
            checks: ['npm test'],
            stopAfter: 'Stop after review.'
          }]
        },
        scaffoldPlan: [{ path: 'docs/architecture-contract.md', action: 'create-file', rationale: 'Write contract.' }],
        artifacts: [{ path: 'AGENTS.md', description: 'Agent rules.' }]
      });
    } else {
      tool(msg.id, {
        ready: false,
        blockers: ['Missing users'],
        challenges: [],
        nextQuestion: { question: 'Who uses it?', recommendedAnswer: 'Name the primary users.' }
      });
    }
  } else if (msg.method === 'tools/call' && msg.params.name === 'review_agent_session') {
    tool(msg.id, { ok: true, name: msg.params.name, received: msg.params.arguments.request });
  } else if (msg.method === 'tools/call') {
    tool(msg.id, { ok: true, name: msg.params.name });
  }
});
"#
}
