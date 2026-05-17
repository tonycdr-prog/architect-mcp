use architect_tui::config::TuiConfig;
use architect_tui::foundry_smoke::{
    FoundrySmokeOptions, FoundrySmokeStatus, build_foundry_smoke_report,
};
use std::process::Command;

#[tokio::test]
async fn foundry_smoke_dry_run_stages_repo_without_github_execution() {
    let Some((source_workspace, config, _temp)) = fake_mcp_config() else {
        return;
    };
    let report = build_foundry_smoke_report(
        source_workspace,
        config,
        &FoundrySmokeOptions {
            json: true,
            public_summary: false,
            owner: "tonycdr-prog".to_string(),
            repo: Some("architect-mcp-foundry-dry-run".to_string()),
            execute: false,
            confirm_private_repo_mutation: false,
            keep_workspace: true,
        },
    )
    .await
    .expect("foundry smoke report");

    assert_eq!(report.status, FoundrySmokeStatus::Passed);
    assert_eq!(report.mode, "dry_run");
    assert_eq!(report.target, "tonycdr-prog/architect-mcp-foundry-dry-run");
    assert!(
        report
            .staged_repo
            .as_ref()
            .is_some_and(|path| path.exists())
    );
    assert!(report.staged_artifacts >= 8);
    assert!(report.execution_status.is_none());
    assert!(report.github.is_none());
    assert!(
        report
            .commands
            .iter()
            .any(|command| command.command == "foundry create")
    );
}

fn fake_mcp_config() -> Option<(std::path::PathBuf, TuiConfig, tempfile::TempDir)> {
    if Command::new("node").arg("--version").output().is_err() {
        return None;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let server_path = temp.path().join("fake-mcp.mjs");
    std::fs::write(&server_path, fake_mcp_server_script()).expect("fake server");
    let mut config = TuiConfig::default();
    config.architect_mcp.command = Some("node".to_string());
    config.architect_mcp.args = vec![server_path.display().to_string()];
    Some((temp.path().to_path_buf(), config, temp))
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
    tool(msg.id, {
      ready: true,
      blockers: [],
      challenges: [],
      updatedBrief: {
        idea: msg.params.arguments.brief.idea,
        users: 'maintainers',
        coreFlows: ['plan repo', 'approve creation'],
        stack: { backend: 'Rust TUI', database: 'none' },
        verification: ['cargo test --workspace --test foundry_smoke']
      },
      contract: { name: 'Foundry Smoke App' },
      buildPlan: {
        archetype: 'custom-app',
        slices: [{
          id: 'foundry-smoke',
          title: 'Foundry smoke',
          order: 1,
          goal: 'Plan a private repo without mutation',
          inputs: ['brief'],
          outputs: ['foundry plan'],
          allowedDirectories: ['crates/architect-tui/src', 'crates/architect-tui/tests'],
          forbiddenFiles: [],
          files: ['crates/architect-tui/src/foundry_smoke.rs'],
          checks: ['cargo test --workspace --test foundry_smoke'],
          stopAfter: 'Stop before GitHub mutation.'
        }]
      },
      scaffoldPlan: [
        { path: 'crates/architect-tui/src/foundry_smoke.rs', action: 'create-file', rationale: 'Foundry smoke logic.' },
        { path: 'crates/architect-tui/tests/foundry_smoke.rs', action: 'create-file', rationale: 'Dry-run smoke test.' }
      ],
      artifacts: [{ path: 'AGENTS.md', description: 'Agent rules.' }]
    });
  } else if (msg.method === 'tools/call') {
    tool(msg.id, { ok: true, name: msg.params.name });
  }
});
"#
}
