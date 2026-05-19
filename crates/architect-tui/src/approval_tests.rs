use std::fs;
use std::path::PathBuf;

use serde_json::json;

use crate::approval::{REQUIRED_REVIEW_GATES, promote_approved_changes, promotion_readiness};
use crate::session::TuiSession;

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
    let promoted = promote_approved_changes(&mut session, workspace).expect("promote");
    assert_eq!(promoted, vec![PathBuf::from("docs/result.md")]);
    assert_eq!(
        fs::read_to_string(workspace.join("docs/result.md")).expect("promoted"),
        "done\n"
    );
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
    assert!(promote_approved_changes(&mut session, workspace).is_ok());
}
