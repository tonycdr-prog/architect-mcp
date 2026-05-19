use std::fs;
use std::path::PathBuf;

use serde_json::json;

use crate::approval::{
    REQUIRED_REVIEW_GATES, promote_approved_changes, promotion_readiness, promotion_status_lines,
};
use crate::session::{SessionStore, TuiSession};

#[test]
fn promotion_requires_approval_and_isolated_worktree() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(worktree.join("docs")).expect("worktree");
    fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

    let mut session = TuiSession::new("build", "codex");
    session.worktree = Some(worktree);
    session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
    assert!(promote_approved_changes(&mut session, workspace).is_err());

    session.approve("reviewed");
    session.set_required_verification(vec!["npm test".to_string()]);
    session
        .verification
        .insert("npm test".to_string(), "passed".to_string());
    for gate in REQUIRED_REVIEW_GATES {
        session.set_gate(gate, json!({ "ok": true }));
    }
    session.set_gate(
        "review_agent_session",
        json!({ "status": "pass", "valid": true, "summary": { "errors": 0, "warnings": 0 } }),
    );
    let promoted = promote_approved_changes(&mut session, workspace).expect("promote");
    assert_eq!(promoted, vec![PathBuf::from("docs/result.md")]);
    let receipt = session.promotion_receipt.as_ref().expect("receipt");
    assert_eq!(receipt.decision, "approved");
    assert_eq!(receipt.reason.as_deref(), Some("reviewed"));
    assert_eq!(receipt.promoted_files, vec!["docs/result.md"]);
    assert_eq!(receipt.changed_files[0]["path"], "docs/result.md");
    assert_eq!(receipt.verification["npm test"], "passed");
    assert_eq!(
        receipt.review_gates["review_agent_session"]
            .status
            .as_deref(),
        Some("pass")
    );
    assert_eq!(
        receipt.review_gates["review_agent_session"].valid,
        Some(true)
    );
    assert_eq!(
        fs::read_to_string(workspace.join("docs/result.md")).expect("promoted"),
        "done\n"
    );

    let store = SessionStore::for_workspace(workspace);
    let path = store.save(&mut session).expect("save session");
    let body = fs::read_to_string(path).expect("session json");
    assert!(body.contains("\"promotionReceipt\""));
    assert!(body.contains("\"promotedFiles\""));
    assert!(!body.contains("promotion_receipt"));
    let loaded = store.load(&session.id).expect("load session");
    let loaded_receipt = loaded.promotion_receipt.as_ref().expect("loaded receipt");
    assert_eq!(loaded_receipt.decision, "approved");
    assert_eq!(loaded_receipt.promoted_files, vec!["docs/result.md"]);
    let loaded_session_gate = &loaded_receipt.review_gates["review_agent_session"];
    assert_eq!(loaded_session_gate.status.as_deref(), Some("pass"));
}

#[test]
fn promotion_readiness_reports_blockers_and_next_actions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let session = TuiSession::new("build", "codex");

    let readiness = promotion_readiness(&session, workspace);

    assert!(!readiness.ready);
    assert!(
        readiness
            .blockers
            .contains(&"promotion approval missing".to_string())
    );
    assert!(
        readiness
            .blockers
            .contains(&"adapter isolated worktree evidence missing".to_string())
    );
    assert!(
        readiness
            .blockers
            .contains(&"changed-file evidence missing".to_string())
    );
    assert!(
        readiness
            .next_actions
            .contains(&"run adapter after execution approval".to_string())
    );
}

#[test]
fn promotion_requires_changed_file_evidence_even_with_override() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(&worktree).expect("worktree");

    let mut session = TuiSession::new("build", "codex");
    session.worktree = Some(worktree);
    session.override_approval("maintainer override still needs files");

    let error = promote_approved_changes(&mut session, workspace).expect_err("blocked");
    assert!(error.to_string().contains("changed-file evidence missing"));
}

#[test]
fn promotion_override_requires_explicit_reason_before_bypassing_gates() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(worktree.join("docs")).expect("worktree");
    fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

    let mut session = TuiSession::new("build", "codex");
    session.worktree = Some(worktree);
    session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
    session.override_approval("manual TUI override");

    let readiness = promotion_readiness(&session, workspace);
    assert!(!readiness.ready);
    assert!(
        readiness
            .blockers
            .contains(&"explicit promotion override reason missing".to_string())
    );
    assert!(
        readiness
            .blockers
            .contains(&"missing review gate: review_implementation_against_contract".to_string())
    );
    assert!(
        readiness
            .next_actions
            .contains(&"run override <reason>".to_string())
    );
    assert!(promote_approved_changes(&mut session, workspace).is_err());

    session.override_approval("maintainer inspected missing gate evidence");
    assert!(promote_approved_changes(&mut session, workspace).is_ok());
}

#[test]
fn promotion_rejects_unsafe_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(&worktree).expect("worktree");

    let mut session = TuiSession::new("build", "codex");
    session.approve("reviewed");
    session.worktree = Some(worktree);
    for gate in REQUIRED_REVIEW_GATES {
        session.set_gate(gate, json!({ "ok": true }));
    }
    session.changed_files = vec![json!({ "path": "../secrets.env", "lines": 1 })];
    assert!(promote_approved_changes(&mut session, workspace).is_err());
}

#[test]
fn promotion_requires_review_gates_unless_overridden() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(worktree.join("docs")).expect("worktree");
    fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

    let mut session = TuiSession::new("build", "codex");
    session.approve("reviewed");
    session.worktree = Some(worktree);
    session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
    assert!(promote_approved_changes(&mut session, workspace).is_err());

    session.override_approval("maintainer override for smoke fixture");
    promote_approved_changes(&mut session, workspace).expect("override promote");
    let receipt = session.promotion_receipt.as_ref().expect("receipt");
    assert_eq!(receipt.decision, "override");
    assert_eq!(
        receipt.reason.as_deref(),
        Some("maintainer override for smoke fixture")
    );
    assert!(
        !receipt
            .review_gates
            .get("review_implementation_against_contract")
            .expect("gate")
            .present
    );
}

#[test]
fn promotion_blocks_failed_adapter_runs_without_override() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(worktree.join("docs")).expect("worktree");
    fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

    let mut session = TuiSession::new("build", "codex");
    session.approve("reviewed");
    session.worktree = Some(worktree);
    session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
    session.set_required_verification(vec!["npm test".to_string()]);
    session
        .verification
        .insert("npm test".to_string(), "passed".to_string());
    for gate in REQUIRED_REVIEW_GATES {
        session.set_gate(gate, json!({ "ok": true }));
    }
    session.record_adapter_run_issue("adapter exited with code 2");

    let readiness = promotion_readiness(&session, workspace);
    assert!(!readiness.ready);
    assert!(
        readiness
            .blockers
            .contains(&"adapter run issue: adapter exited with code 2".to_string())
    );
    assert!(
        readiness
            .next_actions
            .contains(&"rerun adapter successfully or record an explicit override".to_string())
    );
    assert!(promote_approved_changes(&mut session, workspace).is_err());

    session.override_approval("maintainer accepts failed adapter evidence");
    let status = promotion_status_lines(&session, workspace).join("\n");
    assert!(status.contains("adapter-run, review, and verification blockers are bypassed"));
    promote_approved_changes(&mut session, workspace).expect("override promote");
    let receipt = session.promotion_receipt.as_ref().expect("receipt");
    assert_eq!(
        receipt.adapter_run_issues,
        vec!["adapter exited with code 2".to_string()]
    );
}

#[test]
fn promotion_readiness_uses_structured_review_status_not_text_matches() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(worktree.join("src/features")).expect("worktree");
    fs::write(
        worktree.join("src/features/result.ts"),
        "export const ok = true;\n",
    )
    .expect("file");

    let mut session = TuiSession::new("build", "codex");
    session.approve("reviewed");
    session.worktree = Some(worktree);
    session.changed_files = vec![json!({ "path": "src/features/result.ts", "lines": 1 })];
    session.set_required_verification(vec!["npm test".to_string()]);
    session
        .verification
        .insert("npm test".to_string(), "passed".to_string());
    session.set_gate(
        "review_implementation_against_contract",
        json!({
            "valid": true,
            "violations": [{
                "severity": "warning",
                "message": "Report skipped/failed checks honestly before completion."
            }]
        }),
    );
    session.set_gate(
        "review_repo_structure",
        json!({
            "report": { "gate": { "status": "pass" } },
            "summary": { "errors": 0, "warnings": 0 },
            "violations": []
        }),
    );
    session.set_gate(
        "review_agent_final_response",
        json!({ "status": "warn", "valid": true, "summary": { "errors": 0, "warnings": 1 } }),
    );
    session.set_gate(
        "review_agent_session",
        json!({ "status": "warn", "valid": true, "summary": { "fail": 0, "warn": 1 } }),
    );

    let readiness = promotion_readiness(&session, workspace);

    assert!(readiness.ready, "{readiness:?}");
}
