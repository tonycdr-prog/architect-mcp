use serde_json::json;

use crate::issue_terminal_evidence::build_issue_terminal_evidence_report_from_value;
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness::{
    LaunchReadinessTerminalEvidenceWaiver,
    build_launch_readiness_report_from_reports_with_terminal_waiver,
};
use crate::launch_readiness_public_summary::build_public_summary_at;
use crate::launch_stack::{
    LaunchStackCheckSummary, LaunchStackIssue, LaunchStackItemStatus, LaunchStackPullRequest,
    LaunchStackReport,
};
use crate::launch_stack_discovery::LaunchStackDiscovery;

#[test]
fn readiness_public_summary_preserves_decision_counts_and_terminal_platforms() {
    let report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        Some("example/repo".to_string()),
        stack_report(),
        Some(complete_terminal_evidence()),
        None,
        Vec::new(),
    );

    let summary = build_public_summary_at(&report, 1_779_000_000);

    assert_eq!(summary.schema_version, 1);
    assert_eq!(summary.generated_at_unix_seconds, 1_779_000_000);
    assert_eq!(summary.result, LaunchJudgeResult::ConditionalGo);
    assert_eq!(summary.launch_stack.from_pr, Some(194));
    assert_eq!(summary.launch_stack.pull_request_count, 2);
    assert_eq!(summary.launch_stack.pull_request_order, vec![150, 194]);
    assert_eq!(summary.launch_stack.pull_request_status.passed, 1);
    assert_eq!(summary.launch_stack.pull_request_status.warning, 1);
    assert_eq!(summary.launch_stack.blocker_issues.len(), 1);
    assert_eq!(summary.terminal_evidence.report_count, 2);
    assert_eq!(
        summary.terminal_evidence.platforms,
        vec!["linux", "windows"]
    );
}

#[test]
fn readiness_public_summary_omits_raw_rollups_and_redacts_public_text() {
    let report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        Some("/Users/example/private/repo npm_SECRET".to_string()),
        stack_report_with_noise(),
        Some(complete_terminal_evidence()),
        None,
        Vec::new(),
    );

    let summary = build_public_summary_at(&report, 1);
    let text = serde_json::to_string_pretty(&summary).expect("serialize public summary");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("statusCheckRollup"));
    assert!(!text.contains("failedNames"));
    assert!(!text.contains("pendingNames"));
    assert!(!text.contains("commandSummary"));
    assert!(!text.contains("\"source\""));
    assert!(!text.contains("\"notes\""));
    assert!(!text.contains("/Users/example"));
    assert!(!text.contains("npm_SECRET"));
}

#[test]
fn readiness_public_summary_keeps_terminal_waiver_separate_from_missing_evidence() {
    let report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        Some("example/repo".to_string()),
        waived_stack_report(),
        Some(missing_terminal_evidence()),
        Some(LaunchReadinessTerminalEvidenceWaiver {
            issue: 136,
            reason: "maintainer accepted launch without manual Linux Windows terminal evidence"
                .to_string(),
            applied: false,
        }),
        Vec::new(),
    );

    let summary = build_public_summary_at(&report, 1);

    assert_eq!(summary.result, LaunchJudgeResult::Go);
    assert_eq!(
        summary
            .terminal_evidence
            .issue
            .as_ref()
            .expect("terminal evidence issue")
            .result,
        LaunchJudgeResult::ConditionalGo
    );
    assert_eq!(summary.terminal_evidence.report_count, 0);
    assert!(
        summary
            .terminal_evidence_waiver
            .as_ref()
            .expect("terminal evidence waiver")
            .applied
    );
}

fn stack_report() -> LaunchStackReport {
    LaunchStackReport {
        schema_version: 1,
        result: LaunchJudgeResult::ConditionalGo,
        repository: Some("example/repo".to_string()),
        stack_discovery: Some(LaunchStackDiscovery {
            from_pr: 194,
            pull_requests: vec![150, 194],
            stopped_at_base: "main".to_string(),
        }),
        pull_requests: vec![
            pr(150, LaunchStackItemStatus::Passed),
            pr(194, LaunchStackItemStatus::Warning),
        ],
        blocker_issues: vec![blocker(136, LaunchStackItemStatus::Warning, None)],
        findings: Vec::new(),
        next_actions: vec!["resolve blocker #136 before launch".to_string()],
    }
}

fn stack_report_with_noise() -> LaunchStackReport {
    let mut report = stack_report();
    report.repository = Some("/Users/example/private/repo npm_SECRET".to_string());
    report.findings =
        vec!["manual evidence referenced /Users/example/private and npm_SECRET".to_string()];
    report
}

fn waived_stack_report() -> LaunchStackReport {
    LaunchStackReport {
        schema_version: 1,
        result: LaunchJudgeResult::Go,
        repository: Some("example/repo".to_string()),
        stack_discovery: Some(LaunchStackDiscovery {
            from_pr: 194,
            pull_requests: vec![150, 194],
            stopped_at_base: "main".to_string(),
        }),
        pull_requests: vec![
            pr(150, LaunchStackItemStatus::Passed),
            pr(194, LaunchStackItemStatus::Passed),
        ],
        blocker_issues: vec![blocker(
            136,
            LaunchStackItemStatus::Waived,
            Some("maintainer accepted launch with platform QA waiver"),
        )],
        findings: Vec::new(),
        next_actions: Vec::new(),
    }
}

fn pr(number: u64, status: LaunchStackItemStatus) -> LaunchStackPullRequest {
    LaunchStackPullRequest {
        number,
        title: format!("PR #{number} raw title that summary should omit"),
        url: format!("https://github.com/example/repo/pull/{number}"),
        is_draft: false,
        review_decision: Some("APPROVED".to_string()),
        merge_state_status: "CLEAN".to_string(),
        checks: LaunchStackCheckSummary {
            total: 8,
            passed: 8,
            pending: 0,
            failed: 0,
            names: vec!["verify".to_string(), "live-qa (ubuntu-latest)".to_string()],
            pending_names: Vec::new(),
            failed_names: Vec::new(),
            missing_required_names: Vec::new(),
        },
        status,
        next_action: None,
    }
}

fn blocker(
    number: u64,
    status: LaunchStackItemStatus,
    waiver_reason: Option<&str>,
) -> LaunchStackIssue {
    LaunchStackIssue {
        number,
        title: "Run post-release TUI terminal QA on Windows and Linux".to_string(),
        url: format!("https://github.com/example/repo/issues/{number}"),
        state: "OPEN".to_string(),
        status,
        waiver_reason: waiver_reason.map(str::to_string),
        next_action: None,
    }
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

fn missing_terminal_evidence() -> crate::issue_terminal_evidence::IssueTerminalEvidenceReport {
    build_issue_terminal_evidence_report_from_value(
        Some("example/repo".to_string()),
        136,
        &json!({
            "number": 136,
            "title": "Run post-release TUI terminal QA on Windows and Linux",
            "url": "https://github.com/example/repo/issues/136",
            "body": "Please run terminal QA.",
            "comments": []
        }),
    )
}
