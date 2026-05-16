use architect_tui::config::TuiConfig;
use architect_tui::orchestrator::{HeadlessRunOptions, Orchestrator};
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

#[test]
fn arena_candidates_use_isolated_worktrees() {
    let orchestrator = Orchestrator::new("/tmp/work", TuiConfig::default());
    let candidates = orchestrator.arena_candidates("session-1", &["codex".into(), "claude".into()]);
    assert!(candidates[0].worktree.ends_with("session-1/codex"));
    assert!(candidates[1].worktree.ends_with("session-1/claude"));
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
  } else if (msg.method === 'tools/call') {
    tool(msg.id, { ok: true, name: msg.params.name });
  }
});
"#
}
