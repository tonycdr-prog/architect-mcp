use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use architect_tui::adapter::AdapterConfig;
use architect_tui::config::TuiConfig;
use architect_tui::promotion_smoke::{
    PromotionSmokeOptions, PromotionSmokeStatus, build_promotion_smoke_report,
};
use architect_tui::session::{ApprovalStatus, SessionPhase};

#[tokio::test]
async fn promotion_smoke_with_fixture_adapter_promotes_changed_file() {
    let Some((source_workspace, mut config, _temp)) = fake_mcp_config() else {
        return;
    };
    config
        .adapters
        .insert("walkthrough".to_string(), fixture_adapter());
    let options = PromotionSmokeOptions {
        json: true,
        adapter: "walkthrough".to_string(),
        keep_workspace: false,
        timeout_seconds: 30,
    };

    let report = build_promotion_smoke_report(source_workspace, config, &options)
        .await
        .expect("promotion smoke");

    assert_eq!(report.status, PromotionSmokeStatus::Passed);
    assert_eq!(report.final_phase, Some(SessionPhase::Complete));
    assert_eq!(report.final_approval, Some(ApprovalStatus::Promoted));
    assert_eq!(
        report.promoted_files,
        vec!["docs/codex-adapter-smoke.md".to_string()]
    );
}

fn fixture_adapter() -> AdapterConfig {
    #[cfg(windows)]
    {
        AdapterConfig {
            command: "pwsh".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "New-Item -ItemType Directory -Force -Path docs | Out-Null; Set-Content -NoNewline -Path docs/codex-adapter-smoke.md -Value 'codex-adapter-smoke'".to_string(),
            ],
            pty: false,
            ..AdapterConfig::default()
        }
    }
    #[cfg(not(windows))]
    {
        AdapterConfig {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "mkdir -p docs && printf codex-adapter-smoke > docs/codex-adapter-smoke.md"
                    .to_string(),
            ],
            pty: false,
            ..AdapterConfig::default()
        }
    }
}

fn fake_mcp_config() -> Option<(PathBuf, TuiConfig, tempfile::TempDir)> {
    if Command::new("node").arg("--version").output().is_err() {
        return None;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let server_path = temp.path().join("fake-mcp.mjs");
    std::fs::write(&server_path, fake_mcp_server_script()).expect("fake server");
    let mut config = TuiConfig::default();
    config.architect_mcp.command = Some("node".to_string());
    config.architect_mcp.args = vec![server_path.display().to_string()];
    config.adapters = BTreeMap::new();
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
    respond(msg.id, { tools: [] });
  } else if (msg.method === 'tools/call' && msg.params.name === 'grill_me') {
    tool(msg.id, {
      ready: true,
      blockers: [],
      challenges: [],
      nextQuestion: null,
      updatedBrief: {
        idea: msg.params.arguments.brief.idea,
        stack: { artifact: 'Markdown' },
        repoLayout: { pathMap: { docs: ['docs/codex-adapter-smoke.md'] } },
        verification: ['npm test', 'review_repo_structure']
      },
      contract: { name: 'Promotion Smoke', allowedFiles: ['docs/codex-adapter-smoke.md'] },
      buildPlan: {
        slices: [{
          files: ['docs/codex-adapter-smoke.md'],
          checks: ['npm test', 'review_repo_structure']
        }]
      },
      scaffoldPlan: [{ path: 'docs/codex-adapter-smoke.md', action: 'create-file', rationale: 'Smoke evidence.' }],
      artifacts: []
    });
  } else if (msg.method === 'tools/call') {
    tool(msg.id, { ok: true, name: msg.params.name });
  }
});
"#
}
