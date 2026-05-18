use std::collections::BTreeMap;

use serde_json::json;

use crate::issue_terminal_evidence::build_issue_terminal_evidence_report_from_value;
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness::{
    LaunchReadinessTerminalEvidenceWaiver, build_launch_readiness_report_from_reports,
    build_launch_readiness_report_from_reports_with_terminal_waiver,
    parse_terminal_evidence_waiver,
};
use crate::launch_stack::{LaunchStackItemStatus, build_report_from_items_with_waivers};
use crate::launch_stack_github::{issue_from_value, pr_from_value};

#[test]
fn readiness_stays_conditional_when_stack_is_clean_but_terminal_evidence_is_missing() {
    let stack = clean_stack_report(BTreeMap::new(), "CLOSED");
    let evidence = build_issue_terminal_evidence_report_from_value(
        Some("example/repo".to_string()),
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "Please run terminal QA.",
            "comments": [{"body": "No JSON yet."}]
        }),
    );

    let report = build_launch_readiness_report_from_reports(
        Some("example/repo".to_string()),
        stack,
        Some(evidence),
    );

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert_eq!(report.launch_stack.result, LaunchJudgeResult::Go);
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("Linux and Windows testers"))
    );
}

#[test]
fn readiness_is_no_go_when_terminal_evidence_is_malformed() {
    let stack = clean_stack_report(BTreeMap::new(), "CLOSED");
    let evidence = build_issue_terminal_evidence_report_from_value(
        None,
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "",
            "comments": [
                {
                    "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\"\n```"
                }
            ]
        }),
    );

    let report = build_launch_readiness_report_from_reports(None, stack, Some(evidence));

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert!(
        report
            .terminal_evidence_issue
            .as_ref()
            .expect("terminal evidence issue")
            .extracted_blocks[0]
            .issues
            .iter()
            .any(|finding| finding.contains("JSON could not be parsed"))
    );
}

#[test]
fn readiness_is_no_go_when_terminal_evidence_is_unsafe() {
    let stack = clean_stack_report(BTreeMap::new(), "CLOSED");
    let evidence = build_issue_terminal_evidence_report_from_value(
        None,
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "",
            "comments": [
                {
                    "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux summary at /home/tester/run\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on linux\",\"collectedAt\":\"2026-05-17\"},{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"issue #136 windows public-safe summary\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on windows\",\"collectedAt\":\"2026-05-17\"}]}\n```"
                }
            ]
        }),
    );

    let report = build_launch_readiness_report_from_reports(None, stack, Some(evidence));

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert!(
        report
            .terminal_evidence_issue
            .as_ref()
            .expect("terminal evidence issue")
            .extracted_blocks[0]
            .issues
            .iter()
            .any(|finding| finding.contains("local-path content"))
    );
}

#[test]
fn readiness_go_when_stack_and_linux_windows_terminal_evidence_pass() {
    let stack = clean_stack_report(BTreeMap::new(), "CLOSED");
    let evidence = complete_terminal_evidence();

    let report = build_launch_readiness_report_from_reports(
        Some("example/repo".to_string()),
        stack,
        Some(evidence),
    );

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert!(report.next_actions.is_empty());
    assert_eq!(
        report
            .terminal_evidence_issue
            .as_ref()
            .expect("terminal evidence")
            .terminal_evidence
            .reports
            .len(),
        2
    );
}

#[test]
fn readiness_keeps_waiver_visible_separate_from_missing_terminal_evidence() {
    let stack = clean_stack_report(
        BTreeMap::from([(
            136,
            "maintainer accepted temporary launch waiver while waiting for testers".to_string(),
        )]),
        "OPEN",
    );
    let evidence = build_issue_terminal_evidence_report_from_value(
        Some("example/repo".to_string()),
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "Please run terminal QA.",
            "comments": []
        }),
    );

    let report = build_launch_readiness_report_from_reports(
        Some("example/repo".to_string()),
        stack,
        Some(evidence),
    );

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert_eq!(report.launch_stack.result, LaunchJudgeResult::Go);
    assert_eq!(
        report.launch_stack.blocker_issues[0].status,
        LaunchStackItemStatus::Waived
    );
    assert_eq!(
        report.launch_stack.blocker_issues[0]
            .waiver_reason
            .as_deref(),
        Some("maintainer accepted temporary launch waiver while waiting for testers")
    );
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("Linux and Windows testers"))
    );
}

#[test]
fn readiness_go_when_missing_terminal_evidence_is_explicitly_waived() {
    let stack = clean_stack_report(
        BTreeMap::from([(
            136,
            "maintainer accepted temporary launch waiver while waiting for testers".to_string(),
        )]),
        "OPEN",
    );
    let evidence = build_issue_terminal_evidence_report_from_value(
        Some("example/repo".to_string()),
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "Please run terminal QA.",
            "comments": []
        }),
    );

    let report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        Some("example/repo".to_string()),
        stack,
        Some(evidence),
        Some(terminal_waiver()),
        Vec::new(),
    );

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(report.launch_stack.result, LaunchJudgeResult::Go);
    assert!(
        report
            .terminal_evidence_waiver
            .as_ref()
            .expect("terminal evidence waiver")
            .applied
    );
    assert_eq!(
        report
            .terminal_evidence_issue
            .as_ref()
            .expect("terminal evidence")
            .terminal_evidence
            .reports
            .len(),
        0
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("does not contain terminal-evidence JSON blocks"))
    );
    assert!(
        !report
            .next_actions
            .iter()
            .any(|action| action.contains("Linux and Windows testers"))
    );
}

#[test]
fn readiness_terminal_waiver_does_not_override_unsafe_evidence() {
    let stack = clean_stack_report(BTreeMap::new(), "CLOSED");
    let evidence = build_issue_terminal_evidence_report_from_value(
        None,
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "",
            "comments": [
                {
                    "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux summary at /home/tester/run\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on linux\",\"collectedAt\":\"2026-05-17\"},{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"issue #136 windows public-safe summary\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on windows\",\"collectedAt\":\"2026-05-17\"}]}\n```"
                }
            ]
        }),
    );

    let report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        None,
        stack,
        Some(evidence),
        Some(terminal_waiver()),
        Vec::new(),
    );

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert!(
        !report
            .terminal_evidence_waiver
            .as_ref()
            .expect("terminal evidence waiver")
            .applied
    );
    assert!(
        report
            .terminal_evidence_issue
            .as_ref()
            .expect("terminal evidence issue")
            .extracted_blocks[0]
            .issues
            .iter()
            .any(|finding| finding.contains("local-path content"))
    );
}

#[test]
fn terminal_evidence_waiver_parser_fails_closed_for_unmatched_or_missing_issue() {
    let (_, unmatched_findings) =
        parse_terminal_evidence_waiver(Some(136), &["137=wrong issue".to_string()]);
    assert!(
        unmatched_findings
            .iter()
            .any(|finding| finding.contains("terminal evidence issue is #136"))
    );

    let (_, missing_issue_findings) =
        parse_terminal_evidence_waiver(None, &["136=maintainer decision".to_string()]);
    assert!(
        missing_issue_findings
            .iter()
            .any(|finding| finding.contains("--terminal-evidence-issue was not supplied"))
    );
}

#[test]
fn terminal_evidence_waiver_parser_fails_closed_for_duplicate_or_empty_reason() {
    let (_, duplicate_findings) = parse_terminal_evidence_waiver(
        Some(136),
        &[
            "136=first maintainer decision".to_string(),
            "136=second maintainer decision".to_string(),
        ],
    );
    assert!(
        duplicate_findings
            .iter()
            .any(|finding| finding.contains("duplicate terminal-evidence waiver"))
    );

    let (_, empty_reason_findings) =
        parse_terminal_evidence_waiver(Some(136), &["136=".to_string()]);
    assert!(
        empty_reason_findings
            .iter()
            .any(|finding| finding.contains("needs a public reason"))
    );
}

fn clean_stack_report(
    waivers: BTreeMap<u64, String>,
    blocker_state: &str,
) -> crate::launch_stack::LaunchStackReport {
    let pr = pr_from_value(
        150,
        &json!({
            "number": 150,
            "title": "ready launch slice",
            "url": "https://github.com/example/repo/pull/150",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let blocker = issue_from_value(
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "state": blocker_state
        }),
    );
    build_report_from_items_with_waivers(
        Some("example/repo".to_string()),
        vec![pr],
        vec![blocker],
        Vec::new(),
        waivers,
    )
}

fn complete_terminal_evidence() -> crate::issue_terminal_evidence::IssueTerminalEvidenceReport {
    build_issue_terminal_evidence_report_from_value(
        Some("example/repo".to_string()),
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "",
            "comments": [
                {
                    "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux public-safe summary\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on linux\",\"collectedAt\":\"2026-05-17\",\"notes\":\"summary only, no raw logs\"}]}\n```"
                },
                {
                    "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"issue #136 windows public-safe summary\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on windows\",\"collectedAt\":\"2026-05-17\",\"notes\":\"summary only, no raw logs\"}]}\n```"
                }
            ]
        }),
    )
}

fn terminal_waiver() -> LaunchReadinessTerminalEvidenceWaiver {
    LaunchReadinessTerminalEvidenceWaiver {
        issue: 136,
        reason: "maintainer accepted terminal QA waiver while waiting for testers".to_string(),
        applied: false,
    }
}
