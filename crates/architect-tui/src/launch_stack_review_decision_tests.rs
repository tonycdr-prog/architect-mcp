use serde_json::json;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{LaunchStackItemStatus, build_report_from_items};
use crate::launch_stack_github::pr_from_value;

#[test]
fn launch_stack_review_decisions_affect_pr_readiness() {
    let changes_requested = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "needs review follow-up",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "CHANGES_REQUESTED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let review_required = pr_from_value(
        11,
        &json!({
            "number": 11,
            "title": "waiting for approval",
            "url": "https://github.com/example/repo/pull/11",
            "isDraft": false,
            "reviewDecision": "REVIEW_REQUIRED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let approved = pr_from_value(
        12,
        &json!({
            "number": 12,
            "title": "approved slice",
            "url": "https://github.com/example/repo/pull/12",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let absent = pr_from_value(
        13,
        &json!({
            "number": 13,
            "title": "no required review",
            "url": "https://github.com/example/repo/pull/13",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );

    assert_eq!(changes_requested.status, LaunchStackItemStatus::Failed);
    assert_eq!(
        changes_requested.next_action.as_deref(),
        Some("resolve requested changes on PR #10 before launch stack can be go")
    );
    assert_eq!(review_required.status, LaunchStackItemStatus::Warning);
    assert_eq!(
        review_required.next_action.as_deref(),
        Some("complete required review on PR #11 before final launch go")
    );
    assert_eq!(approved.status, LaunchStackItemStatus::Passed);
    assert_eq!(approved.review_decision.as_deref(), Some("APPROVED"));
    assert_eq!(absent.status, LaunchStackItemStatus::Passed);
    assert_eq!(absent.review_decision, None);

    let report = build_report_from_items(
        None,
        vec![changes_requested, review_required, approved, absent],
        Vec::new(),
        Vec::new(),
    );

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
}

#[test]
fn launch_stack_unknown_review_decision_is_conditional() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "new review state",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "NEW_GITHUB_STATE",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );

    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert_eq!(
        report.pull_requests[0].next_action.as_deref(),
        Some("inspect PR #10 review decision before final launch go")
    );
}

#[test]
fn launch_stack_unresolved_review_threads_block_final_go() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "review comments still open",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "unresolvedReviewThreads": 2,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );

    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert_eq!(
        report.pull_requests[0].status,
        LaunchStackItemStatus::Warning
    );
    assert_eq!(report.pull_requests[0].unresolved_review_threads, 2);
    assert_eq!(
        report.pull_requests[0].next_action.as_deref(),
        Some("resolve 2 unresolved review thread(s) on PR #10 before final launch go")
    );
}
