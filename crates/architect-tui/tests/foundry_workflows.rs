use architect_tui::config::TuiConfig;
use architect_tui::interactive::InteractiveWorkflowEngine;
use architect_tui::orchestrator::Orchestrator;
use std::process::Command;

#[tokio::test]
async fn foundry_plan_defaults_private_and_requires_file_review() {
    let Some((orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready private repo app")
        .await
        .expect("new app");
    let blocked = engine
        .apply_input("foundry plan launchpad")
        .await
        .expect_err("file review required");
    assert!(
        blocked
            .to_string()
            .contains("review files before foundry plan")
    );

    run_gate_to_file_review(&mut engine).await;

    let update = engine
        .apply_input("foundry plan launchpad owner=tonycdr-prog")
        .await
        .expect("foundry plan");
    let transcript = update.transcript.join("\n");
    assert!(transcript.contains("foundry plan ready"));
    assert!(transcript.contains("repo: tonycdr-prog/launchpad"));
    assert!(transcript.contains("visibility: private"));
    assert!(transcript.contains("artifact: AGENTS.md"));
    assert!(transcript.contains("artifact: README.md"));
    assert!(transcript.contains("artifact: .env.example"));
    assert!(transcript.contains("artifact: .github/workflows/ci.yml"));
    let session = update.session.expect("session");
    let plan = session.foundry_plan.expect("plan");
    assert_eq!(plan.repo_name, "launchpad");
    assert_eq!(plan.owner.as_deref(), Some("tonycdr-prog"));
    assert!(!session.foundry_approved);
    assert!(plan.mutation_commands.iter().any(|command| command
        == "gh repo create tonycdr-prog/launchpad --private --source <staged-repo> --remote origin"));
    assert!(
        plan.first_pr
            .body_sections
            .iter()
            .any(|section| section.contains("Verification evidence"))
    );
}

#[tokio::test]
async fn foundry_create_is_approval_gated_and_preview_only_by_default() {
    let Some((orchestrator, temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready private repo app")
        .await
        .expect("new app");
    run_gate_to_file_review(&mut engine).await;
    engine
        .apply_input("foundry plan launchpad owner=tonycdr-prog")
        .await
        .expect("foundry plan");

    let blocked = engine
        .apply_input("foundry create")
        .await
        .expect_err("approval required");
    assert!(
        blocked
            .to_string()
            .contains("approve repo creation before foundry create")
    );

    let update = engine
        .apply_input("foundry approve reviewed private repo creation plan")
        .await
        .expect("foundry approve");
    assert!(update.session.expect("session").foundry_approved);

    let update = engine
        .apply_input("foundry create")
        .await
        .expect("dry-run create");
    let transcript = update.transcript.join("\n");
    assert!(transcript.contains("dry-run only; no GitHub mutation performed"));
    assert!(transcript.contains(
        "would run: gh repo create tonycdr-prog/launchpad --private --source <staged-repo> --remote origin"
    ));
    assert!(!temp.path().join("launchpad").exists());
}

#[tokio::test]
async fn foundry_stage_materializes_scaffold_and_consumes_approval() {
    let Some((orchestrator, temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready private repo app")
        .await
        .expect("new app");
    run_gate_to_file_review(&mut engine).await;
    engine
        .apply_input("foundry plan launchpad owner=tonycdr-prog")
        .await
        .expect("foundry plan");
    let blocked = engine
        .apply_input("foundry stage")
        .await
        .expect_err("approval required");
    assert!(
        blocked
            .to_string()
            .contains("approve local repo staging before foundry stage")
    );
    engine
        .apply_input("foundry approve reviewed local scaffold writes")
        .await
        .expect("approve stage");
    let update = engine.apply_input("foundry stage").await.expect("stage");
    let transcript = update.transcript.join("\n");
    assert!(transcript.contains("foundry staged repo:"));
    let session = update.session.expect("session");
    assert!(!session.foundry_approved);
    let stage = session.foundry_stage.expect("stage");
    assert!(stage.path.starts_with(temp.path()));
    assert!(stage.path.join("AGENTS.md").exists());
    assert!(stage.path.join("README.md").exists());
    assert!(stage.path.join(".env.example").exists());
    assert!(stage.path.join(".github/workflows/ci.yml").exists());
    assert!(stage.path.join("docs/first-pr-draft.md").exists());
    assert_eq!(git_current_branch(&stage.path), "architect/bootstrap");

    let create_blocked = engine
        .apply_input("foundry create --execute")
        .await
        .expect_err("second approval required");
    assert!(
        create_blocked
            .to_string()
            .contains("approve repo creation before foundry create")
    );
}

#[tokio::test]
async fn foundry_state_clears_when_brief_changes() {
    let Some((orchestrator, _temp)) = fake_mcp_orchestrator() else {
        return;
    };
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready private repo app")
        .await
        .expect("new app");
    run_gate_to_file_review(&mut engine).await;
    engine
        .apply_input("foundry plan launchpad")
        .await
        .expect("foundry plan");
    engine
        .apply_input("foundry approve reviewed")
        .await
        .expect("foundry approve");
    let update = engine
        .apply_input("answer users=maintainers with a new target")
        .await
        .expect("answer");
    let session = update.session.expect("session");
    assert!(session.foundry_plan.is_none());
    assert!(session.foundry_stage.is_none());
    assert!(!session.foundry_approved);
}

fn git_current_branch(path: &std::path::Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["branch", "--show-current"])
        .output()
        .expect("git branch");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

async fn run_gate_to_file_review(engine: &mut InteractiveWorkflowEngine) {
    engine.apply_input("grill").await.expect("grill");
    engine.apply_input("contract").await.expect("contract");
    engine.apply_input("review plan").await.expect("plan");
    engine.apply_input("review files").await.expect("files");
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
      nextQuestion: null,
      updatedBrief: {
        idea: msg.params.arguments.brief.idea,
        users: 'maintainers',
        coreFlows: ['plan repo', 'approve creation'],
        stack: { backend: 'Rust TUI', database: 'none' },
        verification: ['cargo test --workspace --test foundry_workflows']
      },
      contract: { name: 'Foundry App' },
      buildPlan: {
        archetype: 'custom-app',
        slices: [{
          id: 'foundry-plan',
          title: 'Foundry plan',
          order: 1,
          goal: 'Plan a private repo without mutation',
          inputs: ['brief'],
          outputs: ['foundry plan'],
          allowedDirectories: ['crates/architect-tui/src', 'crates/architect-tui/tests'],
          forbiddenFiles: [],
          files: ['crates/architect-tui/src/foundry.rs'],
          checks: ['cargo test --workspace --test foundry_workflows'],
          stopAfter: 'Stop before GitHub mutation.'
        }]
      },
      scaffoldPlan: [
        { path: 'crates/architect-tui/src/foundry.rs', action: 'create-file', rationale: 'Foundry planning logic.' },
        { path: 'crates/architect-tui/tests/foundry_workflows.rs', action: 'create-file', rationale: 'Approval and dry-run tests.' }
      ],
      artifacts: [{ path: 'AGENTS.md', description: 'Agent rules.' }]
    });
  } else if (msg.method === 'tools/call') {
    tool(msg.id, { ok: true, name: msg.params.name });
  }
});
"#
}
