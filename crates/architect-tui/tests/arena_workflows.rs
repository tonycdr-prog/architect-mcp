use architect_tui::adapter::AdapterConfig;
use architect_tui::config::TuiConfig;
use architect_tui::interactive::InteractiveWorkflowEngine;
use architect_tui::orchestrator::Orchestrator;
use architect_tui::session::{ApprovalStatus, SessionPhase};
use std::process::Command;

#[test]
fn arena_candidates_use_isolated_worktrees() {
    let orchestrator = Orchestrator::new("/tmp/work", TuiConfig::default());
    let candidates = orchestrator.arena_candidates("session-1", &["codex".into(), "claude".into()]);
    assert!(candidates[0].worktree.ends_with("session-1/codex"));
    assert!(candidates[1].worktree.ends_with("session-1/claude"));
}

#[tokio::test]
async fn interactive_arena_run_records_and_ranks_multiple_candidates() {
    let Some((mut orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(_temp.path());
    let mut shell_a = shell_writer_portable("docs/arena-a.md", "candidate a\n");
    let mut shell_b = shell_writer_portable("docs/arena-b.md", "candidate b\n");
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
    engine
        .apply_input("approve run arena candidates")
        .await
        .expect("approve arena");
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
async fn interactive_arena_requires_approval_and_selects_one_candidate_for_manual_promotion() {
    let Some((mut orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(_temp.path());
    let mut node_a = shell_writer_portable("docs/arena-a.md", "candidate a\n");
    let mut node_b = shell_writer_portable("docs/arena-b.md", "candidate b\n");
    node_a.available = Some(true);
    node_b.available = Some(true);
    let config = orchestrator.config_mut();
    config.adapters.insert("node-a".to_string(), node_a);
    config.adapters.insert("node-b".to_string(), node_b);
    config.agents.default_adapter = "node-a".to_string();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready app with users flows stack risks verification")
        .await
        .expect("new app");
    engine.apply_input("grill").await.expect("grill");
    engine.apply_input("contract").await.expect("contract");
    engine.apply_input("review plan").await.expect("plan");
    engine.apply_input("review files").await.expect("files");

    let too_few = engine
        .apply_input("arena run node-a")
        .await
        .expect_err("arena requires multiple adapters");
    assert!(too_few.to_string().contains("at least two adapters"));

    let blocked = engine
        .apply_input("arena run node-a,node-b")
        .await
        .expect_err("approval required");
    assert!(blocked.to_string().contains("approve arena execution"));

    engine
        .apply_input("approve run arena candidates")
        .await
        .expect("approve arena");
    let update = engine
        .apply_input("arena run node-a,node-b")
        .await
        .expect("arena run");
    let session = update.session.expect("session");
    assert_eq!(session.phase, SessionPhase::ReviewRequired);
    assert_eq!(session.approval_status, ApprovalStatus::Pending);
    assert!(!session.execution_approved);
    assert!(session.worktree.is_none());
    assert!(session.changed_files.is_empty());
    assert_eq!(session.arena_candidates.len(), 2);
    assert!(session.arena_candidates.iter().all(|candidate| {
        candidate
            .worktree
            .as_deref()
            .unwrap_or("")
            .contains(".architect-mcp")
    }));

    let promote_blocked = engine
        .apply_input("promote")
        .await
        .expect_err("arena cannot auto-promote");
    assert!(
        promote_blocked
            .to_string()
            .contains("promotion approval missing")
    );

    let rank = engine
        .apply_input("arena rank")
        .await
        .expect("arena rank")
        .transcript
        .join("\n");
    assert!(rank.contains("#1 node-a") || rank.contains("#1 node-b"));
    assert!(rank.contains("candidate evidence"));

    let update = engine
        .apply_input("arena select node-a")
        .await
        .expect("select candidate");
    let session = update.session.expect("session");
    assert_eq!(session.phase, SessionPhase::ReviewRequired);
    assert_eq!(session.adapter, "node-a");
    assert_eq!(session.approval_status, ApprovalStatus::Pending);
    assert!(
        session
            .worktree
            .as_ref()
            .unwrap()
            .display()
            .to_string()
            .contains("node-a")
    );
    assert!(
        session
            .changed_files
            .iter()
            .any(|file| file["path"] == "docs/arena-a.md")
    );
    assert!(!session.gates.contains_key("review_agent_session"));

    engine
        .apply_input("record verification npm test=passed")
        .await
        .expect("verification");
    engine
        .apply_input("final review Changed files: docs/arena-a.md. Verification: npm test passed. Assumptions: selected arena candidate node-a. Not done: node-b was not promoted.")
        .await
        .expect("final review");
    engine
        .apply_input("session review")
        .await
        .expect("session review");
    engine
        .apply_input("approve promote selected arena candidate")
        .await
        .expect("approve promotion");
    engine.apply_input("promote").await.expect("promote");

    assert_eq!(
        std::fs::read_to_string(_temp.path().join("docs/arena-a.md")).expect("arena-a"),
        "candidate a\n"
    );
    assert!(!_temp.path().join("docs/arena-b.md").exists());
}

#[tokio::test]
async fn interactive_arena_blocks_failed_candidate_selection() {
    let Some((mut orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(_temp.path());
    let mut node_ok = shell_writer_portable("docs/arena-ok.md", "ok\n");
    let mut node_fail = failing_shell_writer_portable("docs/arena-fail.md", "partial\n", 2);
    node_ok.available = Some(true);
    node_fail.available = Some(true);
    let config = orchestrator.config_mut();
    config.adapters.insert("node-ok".to_string(), node_ok);
    config.adapters.insert("node-fail".to_string(), node_fail);
    config.agents.default_adapter = "node-ok".to_string();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready app with users flows stack risks verification")
        .await
        .expect("new app");
    engine.apply_input("grill").await.expect("grill");
    engine.apply_input("contract").await.expect("contract");
    engine.apply_input("review plan").await.expect("plan");
    engine.apply_input("review files").await.expect("files");
    engine
        .apply_input("approve run arena candidates")
        .await
        .expect("approve arena");
    engine
        .apply_input("arena run node-ok,node-fail")
        .await
        .expect("arena run");

    let blocked = engine
        .apply_input("arena select node-fail")
        .await
        .expect_err("failed candidate blocked");
    assert!(blocked.to_string().contains("blocking issues"));

    let update = engine
        .apply_input("arena select node-ok")
        .await
        .expect("ok candidate selected");
    assert_eq!(update.session.expect("session").adapter, "node-ok");
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

fn init_git_repo(path: &std::path::Path) {
    run_git(path, ["init"]);
    run_git(path, ["config", "user.email", "test@example.com"]);
    run_git(path, ["config", "user.name", "architect mcp test"]);
    std::fs::write(path.join("README.md"), "test\n").expect("readme");
    run_git(path, ["add", "README.md"]);
    run_git(path, ["commit", "-m", "init"]);
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

fn failing_shell_writer_portable(path: &str, content: &str, exit_code: i32) -> AdapterConfig {
    let mut writer = shell_writer_portable(path, content);
    #[cfg(windows)]
    {
        if let Some(script) = writer.args.get_mut(2) {
            script.push_str(&format!("; exit {exit_code}"));
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(script) = writer.args.get_mut(1) {
            script.push_str(&format!("; exit {exit_code}"));
        }
    }
    writer
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
