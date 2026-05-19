use serde_json::json;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{LaunchStackItemStatus, build_report_from_items};
use crate::launch_stack_github::pr_from_value_with_required_checks;

#[test]
fn launch_stack_required_checks_preserve_go_when_present() {
    let pr = pr_from_value_with_required_checks(
        10,
        &json!({
            "number": 10,
            "title": "ready slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "live-qa (ubuntu-latest)", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
        &["verify".to_string(), "live-qa (ubuntu-latest)".to_string()],
    );

    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert!(
        report.pull_requests[0]
            .checks
            .missing_required_names
            .is_empty()
    );
}

#[test]
fn launch_stack_required_checks_match_legacy_status_contexts() {
    let pr = pr_from_value_with_required_checks(
        10,
        &json!({
            "number": 10,
            "title": "ready legacy status slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"__typename": "StatusContext", "context": "ci/lint", "state": "SUCCESS", "createdAt": "2026-05-19T00:00:00Z"}
            ]
        }),
        &["ci/lint".to_string()],
    );

    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert!(
        report.pull_requests[0]
            .checks
            .missing_required_names
            .is_empty()
    );
}

#[test]
fn launch_stack_missing_required_check_is_no_go() {
    let pr = pr_from_value_with_required_checks(
        10,
        &json!({
            "number": 10,
            "title": "ready except missing windows evidence",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "live-qa (ubuntu-latest)", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
        &["verify".to_string(), "live-qa (windows-latest)".to_string()],
    );

    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(
        report.pull_requests[0].status,
        LaunchStackItemStatus::Failed
    );
    assert_eq!(
        report.pull_requests[0].checks.missing_required_names,
        vec!["live-qa (windows-latest)"]
    );
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("restore required checks on PR #10"))
    );
}
