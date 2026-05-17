use serde_json::json;

use crate::session::{ApprovalStatus, SessionStore, TuiSession};

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
