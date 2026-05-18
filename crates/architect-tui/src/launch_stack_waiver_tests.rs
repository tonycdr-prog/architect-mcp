use std::collections::BTreeMap;
use std::path::Path;

use serde_json::json;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{
    LaunchStackItemStatus, LaunchStackOptions, build_launch_stack_report,
    build_report_from_items_with_waivers, parse_waivers,
};
use crate::launch_stack_github::{issue_from_value, pr_from_value};

#[test]
fn launch_stack_go_allows_explicit_open_blocker_waiver() {
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
            "state": "OPEN"
        }),
    );
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
    assert!(report.next_actions.is_empty());
}

#[test]
fn malformed_or_unmatched_blocker_waivers_fail_closed() {
    let (waivers, findings) = parse_waivers(&[
        "20=".to_string(),
        "not-a-number=maintainer reason".to_string(),
        "21=maintainer reason".to_string(),
        "22:maintainer reason".to_string(),
    ]);
    let blocker = issue_from_value(
        20,
        &json!({
            "number": 20,
            "title": "external terminal evidence",
            "url": "https://github.com/example/repo/issues/20",
            "state": "OPEN"
        }),
    );
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
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("must use ISSUE=reason"))
    );
}

#[test]
fn unmatched_blocker_waiver_fails_closed_through_launch_stack_report() {
    let report = build_launch_stack_report(
        Path::new("."),
        &LaunchStackOptions {
            json: true,
            merge_plan: false,
            repo: None,
            stack_from_pr: None,
            prs: Vec::new(),
            blockers: Vec::new(),
            waived_blockers: vec!["21=maintainer reason".to_string()],
            required_checks: Vec::new(),
        },
    );

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("waiver supplied for issue #21"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("no PRs or blocker issues"))
    );
}
