use std::fs;

use serde_json::json;

use crate::approval::{REQUIRED_REVIEW_GATES, promote_approved_changes, promotion_readiness};
use crate::session::TuiSession;

#[test]
fn promotion_rejects_unsafe_paths_without_missing_verification_noise() {
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
    mark_verification_passed(&mut session);
    session.changed_files = vec![json!({ "path": "../secrets.env", "lines": 1 })];

    let readiness = promotion_readiness(&session, workspace);
    assert!(!readiness.ready);
    assert!(readiness.blockers.iter().any(|blocker| {
        blocker.contains("changed-file evidence invalid")
            && blocker.contains("unsafe changed file path")
    }));
    assert!(
        !readiness
            .blockers
            .iter()
            .any(|blocker| blocker.contains("no required checks captured"))
    );

    let error = promote_approved_changes(&mut session, workspace).expect_err("unsafe path blocked");
    assert!(error.to_string().contains("unsafe changed file path"));
}

#[test]
fn promotion_requires_review_gates_without_missing_verification_noise() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path();
    let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
    fs::create_dir_all(worktree.join("docs")).expect("worktree");
    fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

    let mut session = TuiSession::new("build", "codex");
    session.approve("reviewed");
    session.worktree = Some(worktree);
    session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
    mark_verification_passed(&mut session);

    let readiness = promotion_readiness(&session, workspace);
    assert!(!readiness.ready);
    for gate in REQUIRED_REVIEW_GATES {
        assert!(
            readiness
                .blockers
                .contains(&format!("missing review gate: {gate}"))
        );
    }
    assert!(
        !readiness
            .blockers
            .iter()
            .any(|blocker| blocker.contains("no required checks captured"))
    );
    let error =
        promote_approved_changes(&mut session, workspace).expect_err("review gates blocked");
    assert!(
        error
            .to_string()
            .contains("missing review gate: review_implementation_against_contract")
    );

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

fn mark_verification_passed(session: &mut TuiSession) {
    session.set_required_verification(vec!["npm test".to_string()]);
    session
        .verification
        .insert("npm test".to_string(), "passed".to_string());
}
