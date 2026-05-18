use serde_json::json;

use crate::issue_terminal_evidence::{
    IssueTerminalEvidenceReport, build_issue_terminal_evidence_report_from_value,
};
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{
    LaunchStackCheckSummary, LaunchStackIssue, LaunchStackItemStatus, LaunchStackPullRequest,
    LaunchStackReport,
};
use crate::launch_stack_discovery::LaunchStackDiscovery;

pub(crate) fn stack_report() -> LaunchStackReport {
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

pub(crate) fn stack_report_with_noise() -> LaunchStackReport {
    let mut report = stack_report();
    report.repository = Some("/Users/example/private/repo npm_SECRET".to_string());
    report.findings =
        vec!["manual evidence referenced /Users/example/private and npm_SECRET".to_string()];
    report
}

pub(crate) fn waived_stack_report() -> LaunchStackReport {
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

pub(crate) fn complete_terminal_evidence() -> IssueTerminalEvidenceReport {
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

pub(crate) fn missing_terminal_evidence() -> IssueTerminalEvidenceReport {
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

fn pr(number: u64, status: LaunchStackItemStatus) -> LaunchStackPullRequest {
    LaunchStackPullRequest {
        number,
        title: format!("PR #{number} raw title that summary should omit"),
        url: format!("https://github.com/example/repo/pull/{number}"),
        is_draft: false,
        review_decision: Some("APPROVED".to_string()),
        unresolved_review_threads: 0,
        merge_state_status: "CLEAN".to_string(),
        mergeable: Some("MERGEABLE".to_string()),
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
