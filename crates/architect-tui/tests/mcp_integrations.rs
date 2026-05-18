use architect_tui::config::TuiConfig;
use architect_tui::interactive::InteractiveWorkflowEngine;
use architect_tui::orchestrator::Orchestrator;
use architect_tui::session::SessionStore;
use std::process::Command;

#[tokio::test]
async fn integrations_recommend_database_needs_provider_before_supabase_plan() {
    let (orchestrator, _temp) = fake_mcp_orchestrator();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input("new app ready app with database users flows stack risks verification")
        .await
        .expect("new app");
    let update = engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend");
    let transcript = update.transcript.join("\n");
    assert!(transcript.contains("MCP recommendations: needs-clarification"));
    assert!(transcript.contains("Which database or backend provider"));

    let blocked = engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect_err("provider clarification required");
    assert!(
        blocked
            .to_string()
            .contains("answer MCP recommendation questions")
    );

    engine
        .apply_input("answer database=supabase")
        .await
        .expect("provider answer");
    let update = engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend with provider");
    assert!(
        update
            .transcript
            .join("\n")
            .contains("recommended: supabase (high)")
    );
    let update = engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect("install plan");
    assert!(update.transcript.join("\n").contains("package: npm:"));
}

#[tokio::test]
async fn integrations_review_and_approval_gate_config_writes() {
    let (orchestrator, temp) = fake_mcp_orchestrator();
    let target = temp.path().join(".mcp.json");
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input(
            "new app ready app with database=supabase users flows stack risks verification",
        )
        .await
        .expect("new app");
    engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend");
    engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect("plan");

    let blocked = engine
        .apply_input("integrations write .mcp.json")
        .await
        .expect_err("review required");
    assert!(blocked.to_string().contains("review MCP install plan"));
    assert!(!target.exists());

    engine
        .apply_input("integrations review")
        .await
        .expect("review");
    let dry_run = engine
        .apply_input("integrations apply .mcp.json")
        .await
        .expect("dry run");
    assert!(dry_run.transcript.join("\n").contains("dry-run only"));
    assert!(!target.exists());

    let blocked = engine
        .apply_input("integrations write .mcp.json")
        .await
        .expect_err("approval required");
    assert!(blocked.to_string().contains("approve MCP install"));
    assert!(!target.exists());

    engine
        .apply_input("integrations approve reviewed pinned Supabase plan")
        .await
        .expect("approve install");
    let written = engine
        .apply_input("integrations write .mcp.json")
        .await
        .expect("write");
    let session = written.session.expect("session");
    assert!(!session.mcp_install_approved);
    assert!(target.exists());
    assert!(
        std::fs::read_to_string(target)
            .expect("config")
            .contains("supabase")
    );
    let blocked = engine
        .apply_input("integrations write .mcp.json")
        .await
        .expect_err("approval consumed after write");
    assert!(blocked.to_string().contains("approve MCP install"));
}

#[tokio::test]
async fn integrations_write_does_not_record_apply_gate_when_write_is_not_completed() {
    let (orchestrator, temp) = fake_mcp_orchestrator();
    let target = temp.path().join("not-written.mcp.json");
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input(
            "new app ready app with database=supabase users flows stack risks verification",
        )
        .await
        .expect("new app");
    engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend");
    engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect("plan");
    engine
        .apply_input("integrations review")
        .await
        .expect("review");
    let approved = engine
        .apply_input("integrations approve reviewed pinned Supabase plan")
        .await
        .expect("approve");
    let session_id = approved.session.expect("session").id;

    let blocked = engine
        .apply_input("integrations write not-written.mcp.json")
        .await
        .expect_err("non-written status blocks write");

    assert!(
        blocked
            .to_string()
            .contains("MCP install write did not complete: dry-run")
    );
    assert!(!target.exists());
    let stored = SessionStore::for_workspace(temp.path())
        .load(&session_id)
        .expect("stored session");
    assert!(!stored.gates.contains_key("apply_mcp_install_plan"));
    assert!(stored.mcp_install_approved);
}

#[tokio::test]
async fn integrations_fail_closed_for_unknown_and_failed_install_reviews() {
    let (orchestrator, _temp) = fake_mcp_orchestrator();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input(
            "new app ready app with database=supabase users flows stack risks verification",
        )
        .await
        .expect("new app");
    engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend");

    let unknown = engine
        .apply_input("integrations plan unknown target=codex")
        .await
        .expect_err("unknown server blocked");
    assert!(unknown.to_string().contains("was not recommended"));

    engine
        .apply_input("integrations plan supabase target=cursor")
        .await
        .expect("plan");
    let review = engine
        .apply_input("integrations review")
        .await
        .expect("review");
    assert!(
        review
            .transcript
            .join("\n")
            .contains("MCP install review: fail")
    );

    let blocked = engine
        .apply_input("integrations apply .mcp.json")
        .await
        .expect_err("failed review blocks apply");
    assert!(blocked.to_string().contains("review failed"));

    let blocked = engine
        .apply_input("integrations approve unsafe plan")
        .await
        .expect_err("failed review blocks approval");
    assert!(blocked.to_string().contains("review failed"));

    let blocked = engine
        .apply_input("integrations write .mcp.json")
        .await
        .expect_err("failed review blocks write");
    assert!(blocked.to_string().contains("review failed"));
}

#[tokio::test]
async fn changing_answers_clears_stale_integration_state() {
    let (orchestrator, _temp) = fake_mcp_orchestrator();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input(
            "new app ready app with database=supabase users flows stack risks verification",
        )
        .await
        .expect("new app");
    engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend");
    engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect("plan");
    engine
        .apply_input("integrations review")
        .await
        .expect("review");
    engine
        .apply_input("integrations apply .mcp.json")
        .await
        .expect("dry run");
    engine
        .apply_input("integrations approve reviewed pinned Supabase plan")
        .await
        .expect("approve");

    let answer = engine
        .apply_input("answer users=operators with stricter review workflow")
        .await
        .expect("answer");
    let session = answer.session.expect("session");
    assert!(session.mcp_recommendation.is_none());
    assert!(session.mcp_install_plan.is_none());
    assert!(session.mcp_install_review.is_none());
    assert!(!session.mcp_install_approved);
    assert!(!session.gates.contains_key("recommend_mcp_servers"));
    assert!(!session.gates.contains_key("create_mcp_install_plan"));
    assert!(!session.gates.contains_key("review_mcp_install_plan"));
    assert!(!session.gates.contains_key("apply_mcp_install_plan"));

    let blocked = engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect_err("stale recommendation cleared");
    assert!(blocked.to_string().contains("run integrations recommend"));
}

#[tokio::test]
async fn unrelated_answers_preserve_reviewed_mcp_integration_state() {
    let (orchestrator, _temp) = fake_mcp_orchestrator();
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    engine
        .apply_input(
            "new app ready app with database=supabase users flows stack risks verification",
        )
        .await
        .expect("new app");
    engine
        .apply_input("integrations recommend")
        .await
        .expect("recommend");
    engine
        .apply_input("integrations plan supabase target=codex")
        .await
        .expect("plan");
    engine
        .apply_input("integrations review")
        .await
        .expect("review");
    engine
        .apply_input("integrations approve reviewed pinned Supabase plan")
        .await
        .expect("approve");

    let answer = engine
        .apply_input("answer verification=cargo test -p architect-tui mcp_integrations")
        .await
        .expect("answer");
    let session = answer.session.expect("session");

    assert!(session.mcp_recommendation.is_some());
    assert!(session.mcp_install_plan.is_some());
    assert!(session.mcp_install_review.is_some());
    assert!(session.mcp_install_approved);
    assert!(session.gates.contains_key("recommend_mcp_servers"));
    assert!(session.gates.contains_key("create_mcp_install_plan"));
    assert!(session.gates.contains_key("review_mcp_install_plan"));
}

fn fake_mcp_orchestrator() -> (Orchestrator, tempfile::TempDir) {
    Command::new("node")
        .arg("--version")
        .output()
        .expect("node must be available to run MCP integration fixture tests");
    let temp = tempfile::tempdir().expect("tempdir");
    let server_path = temp.path().join("fake-mcp.mjs");
    std::fs::write(&server_path, fake_mcp_server_script()).expect("fake server");
    let mut config = TuiConfig::default();
    config.architect_mcp.command = Some("node".to_string());
    config.architect_mcp.args = vec![server_path.display().to_string()];
    (Orchestrator::new(temp.path(), config), temp)
}

fn fake_mcp_server_script() -> &'static str {
    r#"
import fs from 'node:fs';
import path from 'node:path';
import readline from 'node:readline';
const rl = readline.createInterface({ input: process.stdin });
function respond(id, result) {
  console.log(JSON.stringify({ jsonrpc: '2.0', id, result }));
}
function tool(id, value) {
  respond(id, { content: [{ type: 'text', text: JSON.stringify(value) }], structuredContent: value });
}
function toolError(id, message) {
  respond(id, { isError: true, content: [{ type: 'text', text: message }] });
}
function plan(targetClient) {
  return {
    id: 'mcp-install-supabase',
    serverId: 'supabase',
    serverName: 'supabase',
    targetClient,
    status: 'dry-run',
    hostedMode: false,
    localOnly: true,
    requiresApproval: true,
    writeFiles: false,
    packagePin: 'npm:@supabase/mcp-server-supabase@0.5.4',
    mcpConfig: { mcpServers: { supabase: { command: 'npx', args: ['-y', '@supabase/mcp-server-supabase@0.5.4'] } } },
    clientConfig: { mcpServers: { supabase: { command: 'npx', args: ['-y', '@supabase/mcp-server-supabase@0.5.4'] } } },
    env: ['SUPABASE_ACCESS_TOKEN'],
    postInstall: ['Set SUPABASE_ACCESS_TOKEN in the local environment.'],
    warnings: []
  };
}
rl.on('line', (line) => {
  const msg = JSON.parse(line);
  if (!msg.id) return;
  if (msg.method === 'initialize') {
    respond(msg.id, { protocolVersion: '2025-11-25', capabilities: {}, serverInfo: { name: 'fake', version: '0.0.0' } });
    return;
  }
  if (msg.method === 'tools/list') {
    respond(msg.id, { tools: [
      { name: 'recommend_mcp_servers' },
      { name: 'create_mcp_install_plan' },
      { name: 'review_mcp_install_plan' },
      { name: 'apply_mcp_install_plan' }
    ] });
    return;
  }
  if (msg.method !== 'tools/call') return;
  const name = msg.params.name;
  const request = msg.params.arguments.request;
  if (name === 'recommend_mcp_servers') {
    const text = JSON.stringify(request).toLowerCase();
    if (text.includes('supabase')) {
      tool(msg.id, {
        status: 'pass',
        questions: [],
        recommendations: [{ serverId: 'supabase', confidence: 'high', name: 'Supabase', provider: 'Supabase' }],
        policy: []
      });
    } else if (text.includes('database')) {
      tool(msg.id, {
        status: 'needs-clarification',
        questions: ['Which database or backend provider should this project use before adding an MCP server?'],
        recommendations: [],
        policy: []
      });
    } else {
      tool(msg.id, { status: 'no-match', questions: [], recommendations: [], policy: [] });
    }
    return;
  }
  if (name === 'create_mcp_install_plan') {
    if (request.serverId !== 'supabase') {
      toolError(msg.id, `Unknown MCP server id: ${request.serverId}`);
      return;
    }
    tool(msg.id, plan(request.targetClient ?? 'generic-json'));
    return;
  }
  if (name === 'review_mcp_install_plan') {
    if (request.plan.targetClient === 'cursor') {
      tool(msg.id, {
        status: 'fail',
        findings: [{ code: 'MCPINSTALL_TEST_FAIL', severity: 'error', message: 'fixture failed review' }],
        security: {}
      });
    } else {
      tool(msg.id, { status: 'pass', findings: [], security: {} });
    }
    return;
  }
  if (name === 'apply_mcp_install_plan') {
    const targetPath = request.targetPath ?? '.mcp.json';
    const resolved = path.resolve(process.cwd(), targetPath);
    if (targetPath.includes('not-written')) {
      tool(msg.id, {
        status: 'dry-run',
        targetPath: resolved,
        files: [{ path: resolved, operation: 'create', content: JSON.stringify(request.plan.clientConfig, null, 2) }],
        review: { status: 'pass', findings: [] }
      });
      return;
    }
    if (request.writeFiles && request.explicitApproval) {
      fs.writeFileSync(resolved, `${JSON.stringify(request.plan.clientConfig, null, 2)}\n`, 'utf8');
      tool(msg.id, { status: 'written', targetPath: resolved, review: { status: 'pass', findings: [] } });
    } else {
      tool(msg.id, {
        status: 'dry-run',
        targetPath: resolved,
        files: [{ path: resolved, operation: 'create', content: JSON.stringify(request.plan.clientConfig, null, 2) }],
        review: { status: 'pass', findings: [] }
      });
    }
  }
});
"#
}
