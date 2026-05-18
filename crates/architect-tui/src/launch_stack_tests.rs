use serde_json::json;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{LaunchStackItemStatus, build_report_from_items};
use crate::launch_stack_github::{issue_from_value, pr_from_value, summarize_checks};

#[test]
fn launch_stack_go_requires_clean_non_draft_prs_and_closed_blockers() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "ready slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let blocker = issue_from_value(
        20,
        &json!({
            "number": 20,
            "title": "external terminal evidence",
            "url": "https://github.com/example/repo/issues/20",
            "state": "CLOSED"
        }),
    );

    let report = build_report_from_items(
        Some("example/repo".to_string()),
        vec![pr],
        vec![blocker],
        Vec::new(),
    );

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert!(report.next_actions.is_empty());
}

#[test]
fn launch_stack_is_conditional_for_drafts_pending_checks_and_open_blockers() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "draft slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": true,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "IN_PROGRESS", "conclusion": ""}
            ]
        }),
    );
    let blocker = issue_from_value(
        20,
        &json!({
            "number": 20,
            "title": "external terminal evidence",
            "url": "https://github.com/example/repo/issues/20",
            "state": "OPEN"
        }),
    );

    let report = build_report_from_items(None, vec![pr], vec![blocker], Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("mark PR #10 ready"))
    );
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("blocker issue #20"))
    );
}

#[test]
fn launch_stack_keeps_pending_unstable_prs_conditional() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "pending windows live qa",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "UNSTABLE",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "live-qa (windows-latest)", "status": "IN_PROGRESS", "conclusion": ""}
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
            .any(|action| action.contains("checks to finish"))
    );
}

#[test]
fn launch_stack_is_no_go_for_dirty_merge_state_or_failed_checks() {
    let dirty_pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "conflicted slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "DIRTY",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let failed_pr = pr_from_value(
        11,
        &json!({
            "number": 11,
            "title": "failed slice",
            "url": "https://github.com/example/repo/pull/11",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "live-qa", "status": "COMPLETED", "conclusion": "FAILURE"}
            ]
        }),
    );

    let report = build_report_from_items(None, vec![dirty_pr, failed_pr], Vec::new(), Vec::new());

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(
        report.pull_requests[0].status,
        LaunchStackItemStatus::Failed
    );
    assert_eq!(report.pull_requests[1].checks.failed_names, vec!["live-qa"]);
}

#[test]
fn launch_stack_redacts_local_paths_and_secret_shaped_text() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "uses /Users/example/private npm_TOKEN path=/tmp/workspace (/private/tmp/proof)",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify /home/example cache=/var/folders/build [artifact=/opt/project] win=C:/Users/example", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let text = serde_json::to_string(&pr).expect("serialize pr");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(text.contains("https://github.com/example/repo/pull/10"));
    assert!(!text.contains("/Users/"));
    assert!(!text.contains("/home/"));
    assert!(!text.contains("/tmp/"));
    assert!(!text.contains("/private/"));
    assert!(!text.contains("/var/"));
    assert!(!text.contains("/opt/"));
    assert!(!text.contains("C:/Users"));
    assert!(!text.contains("npm_TOKEN"));
}

#[test]
fn check_summary_classifies_pending_failed_and_successful_checks() {
    let summary = summarize_checks(Some(&json!([
        {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"},
        {"name": "live-qa", "status": "IN_PROGRESS", "conclusion": ""},
        {"name": "install-smoke", "status": "COMPLETED", "conclusion": "FAILURE"}
    ])));

    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.pending, 1);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.names, vec!["verify", "live-qa", "install-smoke"]);
    assert_eq!(summary.pending_names, vec!["live-qa"]);
    assert_eq!(summary.failed_names, vec!["install-smoke"]);
    assert!(summary.missing_required_names.is_empty());
}
