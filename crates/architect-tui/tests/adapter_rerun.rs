use architect_tui::adapter::AdapterConfig;
use architect_tui::config::TuiConfig;
use architect_tui::interactive::InteractiveWorkflowEngine;
use architect_tui::orchestrator::Orchestrator;
use std::path::Path;
use std::process::Command;

#[tokio::test]
async fn interactive_adapter_rerun_replaces_failed_worktree_and_clears_issues() {
    let Some((mut orchestrator, temp)) = fake_mcp_orchestrator() else {
        return;
    };
    init_git_repo(temp.path());
    let script = temp.path().join("flaky-adapter.mjs");
    let state = temp.path().join("flaky-state.txt");
    std::fs::write(&script, flaky_node_adapter_script()).expect("flaky adapter");
    orchestrator.config_mut().adapters.insert(
        "walkthrough".to_string(),
        flaky_node_writer(&script, &state),
    );
    orchestrator.config_mut().agents.default_adapter = "walkthrough".to_string();
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
        .apply_input("approve run isolated adapter")
        .await
        .expect("approve execution");
    let first = engine
        .apply_input("run adapter")
        .await
        .expect("first adapter run")
        .session
        .expect("session");
    assert_eq!(first.adapter_run_issues, vec!["adapter exited with code 2"]);
    assert!(first.adapter_crashed);

    engine
        .apply_input("review files")
        .await
        .expect("re-review files before rerun");
    engine
        .apply_input("approve rerun isolated adapter")
        .await
        .expect("approve rerun");
    let second = engine
        .apply_input("run adapter")
        .await
        .expect("second adapter run")
        .session
        .expect("session");
    assert!(second.adapter_run_issues.is_empty());
    assert!(!second.adapter_crashed);
    assert!(
        second
            .changed_files
            .iter()
            .any(|file| file["path"] == "docs/retry-run.md")
    );
    let worktree = second.worktree.as_ref().expect("worktree");
    assert_eq!(
        std::fs::read_to_string(worktree.join("docs/retry-run.md")).expect("rerun file"),
        "clean rerun\n"
    );

    let status = engine
        .apply_input("promotion status")
        .await
        .expect("promotion status")
        .transcript
        .join("\n");
    assert!(!status.contains("adapter run issue"));
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

fn init_git_repo(path: &Path) {
    run_git(path, ["init"]);
    run_git(path, ["config", "user.email", "test@example.com"]);
    run_git(path, ["config", "user.name", "architect mcp test"]);
    std::fs::write(path.join("README.md"), "test\n").expect("readme");
    run_git(path, ["add", "README.md"]);
    run_git(path, ["commit", "-m", "init"]);
}

fn run_git<const N: usize>(path: &Path, args: [&str; N]) {
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

fn flaky_node_writer(script: &Path, state: &Path) -> AdapterConfig {
    AdapterConfig {
        command: "node".to_string(),
        args: vec![
            script.display().to_string(),
            "docs/retry-run.md".to_string(),
            state.display().to_string(),
        ],
        available: Some(true),
        pty: false,
        ..AdapterConfig::default()
    }
}

fn flaky_node_adapter_script() -> &'static str {
    r#"
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

const [target, state] = process.argv.slice(2);
mkdirSync(dirname(target), { recursive: true });
if (!existsSync(state)) {
  writeFileSync(target, 'partial work\n');
  writeFileSync(state, 'failed once\n');
  process.exit(2);
}
writeFileSync(target, 'clean rerun\n');
"#
}

fn fake_mcp_server_script() -> &'static str {
    r#"
import readline from 'node:readline';
const rl = readline.createInterface({ input: process.stdin });
const tools = ['grill_me', 'create_pre_edit_contract', 'review_build_plan', 'review_proposed_file_plan', 'review_implementation_against_contract', 'review_repo_structure', 'review_agent_final_response', 'review_agent_session'];
function respond(id, result) { console.log(JSON.stringify({ jsonrpc: '2.0', id, result })); }
function tool(id, value) { respond(id, { content: [{ type: 'text', text: JSON.stringify(value) }], structuredContent: value }); }
rl.on('line', (line) => {
  const msg = JSON.parse(line);
  if (!msg.id) return;
  if (msg.method === 'initialize') return respond(msg.id, { protocolVersion: '2025-11-25', capabilities: {}, serverInfo: { name: 'fake', version: '0.0.0' } });
  if (msg.method === 'tools/list') return respond(msg.id, { tools: tools.map((name) => ({ name })) });
  if (msg.method !== 'tools/call') return respond(msg.id, {});
  const name = msg.params.name;
  if (name === 'grill_me') return tool(msg.id, {
    ready: true,
    blockers: [],
    challenges: [],
    nextQuestion: null,
    contract: { name: 'Ready App' },
    buildPlan: { slices: [{ id: 'slice', title: 'Slice', order: 1, verification: ['npm test'], files: ['docs/retry-run.md'] }] },
    scaffoldPlan: [{ path: 'docs/retry-run.md', responsibility: 'rerun evidence' }]
  });
  if (name === 'create_pre_edit_contract') return tool(msg.id, { intent: { decision: 'proceed' }, contract: { name: 'Ready App' } });
  if (name === 'review_repo_structure') return tool(msg.id, { report: { gate: { status: 'pass' }, summary: { errors: 0, warnings: 0 }, violations: [] }, summary: { errors: 0, warnings: 0 }, violations: [] });
  if (name === 'review_agent_final_response') return tool(msg.id, { status: 'pass', valid: true, summary: { errors: 0, warnings: 0 } });
  if (name === 'review_agent_session') return tool(msg.id, { status: 'pass', valid: true, summary: { fail: 0, warn: 0, pass: 1 } });
  return tool(msg.id, { valid: true, violations: [] });
});
"#
}
