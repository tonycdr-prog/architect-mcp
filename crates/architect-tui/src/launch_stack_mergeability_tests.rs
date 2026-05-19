use serde_json::json;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{LaunchStackItemStatus, build_report_from_items};
use crate::launch_stack_github::{pr_from_value, pr_from_value_with_required_checks};

#[test]
fn launch_stack_allows_mergeable_unstable_prs_with_explicit_green_required_checks() {
    let pr = pr_from_value_with_required_checks(
        10,
        &json!({
            "number": 10,
            "title": "green but unstable slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "UNSTABLE",
            "mergeable": "MERGEABLE",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "install-smoke (ubuntu-latest)", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
        &["verify".to_string()],
    );

    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(
        report.pull_requests[0].status,
        LaunchStackItemStatus::Passed
    );
    assert_eq!(
        report.pull_requests[0].mergeable.as_deref(),
        Some("MERGEABLE")
    );
    assert!(report.next_actions.is_empty());
}

#[test]
fn launch_stack_keeps_mergeable_unstable_prs_conditional_without_explicit_required_checks() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "green but unstable slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "UNSTABLE",
            "mergeable": "MERGEABLE",
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
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("supply explicit --required-check evidence"))
    );
}
