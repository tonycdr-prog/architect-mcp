use crate::launch_judge_report::{
    LaunchJudgeResult, LaunchJudgeTerminalEvidenceEnvironment, LaunchJudgeTerminalEvidenceStatus,
};
use crate::launch_readiness::{
    LaunchReadinessTerminalEvidenceWaiver,
    build_launch_readiness_report_from_reports_with_terminal_waiver,
};
use crate::launch_readiness_public_summary::build_public_summary_at;
use crate::launch_readiness_public_summary_test_support::{
    complete_terminal_evidence, missing_terminal_evidence, stack_report, stack_report_with_noise,
    waived_stack_report,
};
use crate::launch_stack::LaunchStackItemStatus;

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
    let terminal = &summary.terminal_evidence;
    assert_eq!(terminal.report_count, 2);
    assert_eq!(terminal.platforms, vec!["linux", "windows"]);
    assert_eq!(
        terminal
            .reports
            .iter()
            .map(|report| (
                report.platform.as_str(),
                &report.status,
                report.environment.as_ref(),
                report.collected_at.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "linux",
                &LaunchJudgeTerminalEvidenceStatus::Passed,
                Some(&LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal),
                Some("2026-05-17")
            ),
            (
                "windows",
                &LaunchJudgeTerminalEvidenceStatus::Passed,
                Some(&LaunchJudgeTerminalEvidenceEnvironment::VmOrCloudTerminal),
                Some("2026-05-17")
            )
        ]
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

#[test]
fn readiness_public_summary_exposes_missing_required_checks_without_raw_rollups() {
    let mut report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        Some("example/repo".to_string()),
        stack_report(),
        None,
        None,
        Vec::new(),
    );
    report.launch_stack.pull_requests[1]
        .checks
        .missing_required_names = vec![
        "verify".to_string(),
        "live-qa /Users/example npm_SECRET".to_string(),
    ];
    report.launch_stack.pull_requests[1].status = LaunchStackItemStatus::Failed;

    let summary = build_public_summary_at(&report, 1);
    let text = serde_json::to_string_pretty(&summary).expect("serialize public summary");

    assert_eq!(summary.launch_stack.missing_required_check_count, 2);
    assert_eq!(summary.launch_stack.missing_required_checks.len(), 1);
    assert_eq!(
        summary.launch_stack.missing_required_checks[0].pull_request,
        194
    );
    assert_eq!(
        summary.launch_stack.missing_required_checks[0].names,
        vec![
            "verify".to_string(),
            "live-qa [redacted-local-path] [redacted-secret]".to_string()
        ]
    );
    assert!(!text.contains("statusCheckRollup"));
    assert!(!text.contains("pendingNames"));
    assert!(!text.contains("failedNames"));
    assert!(!text.contains("/Users/example"));
    assert!(!text.contains("npm_SECRET"));
}

#[test]
fn readiness_public_summary_exposes_unresolved_review_threads_without_raw_text() {
    let mut report = build_launch_readiness_report_from_reports_with_terminal_waiver(
        Some("example/repo".to_string()),
        stack_report(),
        None,
        None,
        Vec::new(),
    );
    report.launch_stack.pull_requests[1].unresolved_review_threads = 3;
    report.launch_stack.pull_requests[1].status = LaunchStackItemStatus::Warning;

    let summary = build_public_summary_at(&report, 1);
    let text = serde_json::to_string_pretty(&summary).expect("serialize public summary");

    assert_eq!(summary.launch_stack.unresolved_review_thread_count, 3);
    assert_eq!(summary.launch_stack.unresolved_review_threads.len(), 1);
    assert_eq!(
        summary.launch_stack.unresolved_review_threads[0].pull_request,
        194
    );
    assert_eq!(summary.launch_stack.unresolved_review_threads[0].count, 3);
    assert!(!text.contains("reviewThreads"));
    assert!(!text.contains("comments"));
    assert!(!text.contains("body"));
}
