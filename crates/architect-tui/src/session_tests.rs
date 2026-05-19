use serde_json::json;

use crate::session::{ApprovalStatus, SessionPhase, SessionStore, TuiSession};

#[test]
fn session_store_round_trips_without_secrets() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = SessionStore::for_workspace(temp.path());
    let mut session = TuiSession::new("build a recipe app", "codex");
    session.set_answer("users", "home cooks");
    let path = store.save(&mut session).expect("save");
    assert!(path.exists());

    let loaded = store.load(&session.id).expect("load");
    assert_eq!(loaded.prompt, "build a recipe app");
    assert_eq!(loaded.brief["users"], "home cooks");
}

#[test]
fn approval_state_controls_promotion_readiness() {
    let mut session = TuiSession::new("build a recipe app", "codex");
    assert!(!session.can_promote());
    session.approve("review gates passed");
    assert!(session.can_promote());
    session.mark_promoted();
    assert_eq!(session.approval_status, ApprovalStatus::Promoted);
}

#[test]
fn answers_shape_project_brief_for_live_grill() {
    let mut session = TuiSession::new("build controlled TUI", "codex");
    session.set_answer("users", "maintainers need to control agents");
    session.set_answer("coreFlows", "grill; review plan; promote");
    session.set_answer(
        "verification",
        "cargo test --workspace; npm run release:check",
    );
    session.set_answer("stack", "frontend=Rust Ratatui; backend=TypeScript MCP");
    session.set_answer(
        "repoLayout",
        "tui=crates/architect-tui/src; tests=crates/architect-tui/tests",
    );

    assert_eq!(session.brief["users"], "maintainers need to control agents");
    assert_eq!(
        session.brief["coreFlows"].as_array().expect("flows").len(),
        3
    );
    assert_eq!(
        session.brief["verification"]
            .as_array()
            .expect("verification")
            .len(),
        2
    );
    assert_eq!(session.brief["stack"]["backend"], "TypeScript MCP");
    assert_eq!(
        session.brief["repoLayout"]["pathMap"]["tui"][0],
        "crates/architect-tui/src"
    );
}

#[test]
fn legacy_session_json_defaults_new_fields() {
    let legacy = json!({
        "id": "session-1",
        "prompt": "build controlled TUI",
        "adapter": "codex",
        "phase": "review_required",
        "brief": { "idea": "build controlled TUI" },
        "gates": {},
        "verification": {},
        "worktree": null,
        "diffStat": null,
        "changedFiles": [],
        "createdAt": 1,
        "updatedAt": 1
    });

    let session: TuiSession =
        serde_json::from_value(legacy).expect("legacy session should deserialize");
    assert!(!session.execution_approved);
    assert!(session.execution_approval_reason.is_none());
    assert!(session.required_verification.is_empty());
    assert!(session.final_response.is_none());
    assert!(session.adapter_run_issues.is_empty());
    assert_eq!(session.approval_status, ApprovalStatus::Pending);
    assert!(session.approval_reason.is_none());
}

#[test]
fn clear_adapter_run_evidence_resets_stale_promotion_state() {
    let mut session = TuiSession::new("build controlled TUI", "codex");
    session.phase = SessionPhase::Complete;
    session.worktree = Some("/tmp/old-worktree".into());
    session.diff_stat = Some("docs/old.md | 1 +".to_string());
    session.changed_files = vec![json!({ "path": "docs/old.md", "lines": 1 })];
    session
        .verification
        .insert("npm test".to_string(), "passed".to_string());
    session.set_final_response("old final response");
    session.record_adapter_run_issue("adapter exited with code 2");
    session.approve("old approval");
    session.set_gate("grill_me", json!({ "ready": true }));
    session.set_gate("review_build_plan", json!({ "ok": true }));
    session.set_gate(
        "review_implementation_against_contract",
        json!({ "ok": true }),
    );
    session.set_gate("review_repo_structure", json!({ "ok": true }));
    session.set_gate("review_agent_final_response", json!({ "ok": true }));
    session.set_gate("review_agent_session", json!({ "ok": true }));

    session.clear_adapter_run_evidence();

    assert!(session.worktree.is_none());
    assert!(session.diff_stat.is_none());
    assert!(session.changed_files.is_empty());
    assert!(session.verification.is_empty());
    assert!(session.final_response.is_none());
    assert!(!session.adapter_crashed);
    assert!(session.adapter_run_issues.is_empty());
    assert_eq!(session.approval_status, ApprovalStatus::Pending);
    assert!(session.approval_reason.is_none());
    assert!(session.gates.contains_key("grill_me"));
    assert!(session.gates.contains_key("review_build_plan"));
    assert!(
        !session
            .gates
            .contains_key("review_implementation_against_contract")
    );
    assert!(!session.gates.contains_key("review_repo_structure"));
    assert!(!session.gates.contains_key("review_agent_final_response"));
    assert!(!session.gates.contains_key("review_agent_session"));
}

#[test]
fn session_store_only_persists_mcp_integration_metadata() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = SessionStore::for_workspace(temp.path());
    let mut session = TuiSession::new("build controlled TUI", "codex");
    session.set_mcp_recommendation(json!({
        "status": "pass",
        "recommendations": [{
            "serverId": "supabase",
            "provider": "Supabase",
            "confidence": "high",
            "name": "Supabase",
            "requiredEnv": ["SUPABASE_ACCESS_TOKEN"]
        }],
        "policy": ["policy"]
    }));
    session.set_mcp_install_plan(json!({
        "id": "mcp-install-supabase",
        "serverId": "supabase",
        "serverName": "supabase",
        "targetClient": "codex",
        "status": "dry-run",
        "hostedMode": false,
        "localOnly": true,
        "requiresApproval": true,
        "writeFiles": false,
        "packagePin": "npm:@supabase/mcp-server-supabase@0.5.4",
        "env": ["SUPABASE_ACCESS_TOKEN"],
        "postInstall": ["Set SUPABASE_ACCESS_TOKEN"],
        "warnings": [],
        "mcpConfig": { "mcpServers": { "supabase": { "command": "npx", "args": ["secret-token"] } } },
        "clientConfig": { "mcpServers": { "supabase": { "env": { "SUPABASE_ACCESS_TOKEN": "secret-token" } } } }
    }));
    session.set_mcp_install_review(json!({
        "status": "pass",
        "findings": [{
            "code": "SAFE",
            "severity": "warning",
            "message": "checked",
            "recommendation": "keep dry-run",
            "raw": "secret-token"
        }],
        "security": { "raw": "secret-token" }
    }));
    session.set_mcp_install_apply_result(json!({
        "status": "dry-run",
        "targetPath": "/tmp/.mcp.json",
        "files": [{ "path": "/tmp/.mcp.json", "content": "{ secret-token }" }],
        "review": {
            "status": "pass",
            "findings": [{ "code": "SAFE", "severity": "warning", "message": "checked" }]
        }
    }));

    let path = store.save(&mut session).expect("save");
    let body = std::fs::read_to_string(path).expect("session json");
    assert!(!body.contains("secret-token"));
    assert!(!body.contains("\"clientConfig\""));
    assert!(!body.contains("\"mcpConfig\""));
    assert!(!body.contains("\"files\""));
    assert!(body.contains("\"packagePin\""));
    assert!(body.contains("\"SUPABASE_ACCESS_TOKEN\""));
}
