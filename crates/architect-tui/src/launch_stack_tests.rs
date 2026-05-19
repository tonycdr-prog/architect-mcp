use std::collections::BTreeMap;

use serde_json::json;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{
    LaunchStackIssue, LaunchStackItemStatus, LaunchStackPullRequest, build_report_from_items,
    build_report_from_items_with_waivers, parse_waivers,
};
use crate::launch_stack_github::{issue_from_value, pr_from_value, summarize_checks};

fn ready_pr(number: u64) -> LaunchStackPullRequest {
    pr_from_value(
        number,
        &json!({
            "number": number,
            "title": "ready slice",
            "url": format!("https://github.com/example/repo/pull/{number}"),
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    )
}

fn blocker_issue(number: u64, state: &str) -> LaunchStackIssue {
    issue_from_value(
        number,
        &json!({
            "number": number,
            "title": "external terminal evidence",
            "url": format!("https://github.com/example/repo/issues/{number}"),
            "state": state
        }),
    )
}

#[test]
fn launch_stack_go_requires_clean_non_draft_prs_and_closed_blockers() {
    let pr = ready_pr(10);
    let blocker = blocker_issue(20, "CLOSED");

    let report = build_report_from_items(
        Some("example/repo".to_string()),
        vec![pr],
        vec![blocker],
        Vec::new(),
    );

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(report.schema_version, 2);
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
fn launch_stack_go_allows_explicit_open_blocker_waiver() {
    let pr = ready_pr(10);
    let blocker = blocker_issue(20, "OPEN");
    let waivers = BTreeMap::from([(
        20,
        "maintainer accepted temporary Linux terminal waiver".to_string(),
    )]);

    let report =
        build_report_from_items_with_waivers(None, vec![pr], vec![blocker], Vec::new(), waivers);

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(
        report.blocker_issues[0].status,
        LaunchStackItemStatus::Waived
    );
    assert_eq!(
        report.blocker_issues[0].waiver_reason.as_deref(),
        Some("maintainer accepted temporary Linux terminal waiver")
    );
    let report_json = serde_json::to_value(&report).expect("serialize launch-stack report");
    assert_eq!(
        report_json["blockerIssues"][0]["waiverReason"],
        "maintainer accepted temporary Linux terminal waiver"
    );
    assert!(report.next_actions.is_empty());
}

#[test]
fn launch_stack_omits_waiver_reason_when_not_waived() {
    let pr = ready_pr(10);
    let blocker = blocker_issue(20, "CLOSED");
    let report = build_report_from_items(None, vec![pr], vec![blocker], Vec::new());
    let report_json = serde_json::to_value(&report).expect("serialize launch-stack report");
    assert!(
        report_json["blockerIssues"][0]
            .get("waiverReason")
            .is_none()
    );
}

#[test]
fn malformed_or_unmatched_blocker_waivers_fail_closed() {
    let (waivers, findings) = parse_waivers(&[
        "20=".to_string(),
        "not-a-number=maintainer reason".to_string(),
        "21=maintainer reason".to_string(),
    ]);
    let blocker = blocker_issue(20, "OPEN");
    let mut combined_findings = findings;
    combined_findings.push(
        "waiver supplied for issue #21, but that issue was not supplied as a blocker".to_string(),
    );

    let report = build_report_from_items_with_waivers(
        None,
        Vec::new(),
        vec![blocker],
        combined_findings,
        waivers,
    );

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(
        report.blocker_issues[0].status,
        LaunchStackItemStatus::Warning
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("needs a public reason"))
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
            "title": "uses /Users/example/private npm_TOKEN",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify /home/example", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let text = serde_json::to_string(&pr).expect("serialize pr");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("/Users/"));
    assert!(!text.contains("/home/"));
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
    assert_eq!(summary.pending_names, vec!["live-qa"]);
    assert_eq!(summary.failed_names, vec!["install-smoke"]);
}
