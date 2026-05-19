use crate::evidence_index_markdown::render_markdown;
use crate::evidence_index_report::build_evidence_index_report_from_public_summaries_at;
use crate::governance_audit_public_summary::{
    GovernanceAuditPublicFindingCounts, GovernanceAuditPublicGateSummary,
    GovernanceAuditPublicMemorySummary, GovernanceAuditPublicSummary,
};
use crate::governance_audit_report::GovernanceAuditStatus;
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness_public_summary::{
    LaunchReadinessPublicMissingRequiredCheck, LaunchReadinessPublicStack,
    LaunchReadinessPublicStatusCounts, LaunchReadinessPublicSummary,
    LaunchReadinessPublicTerminalEvidence, LaunchReadinessPublicUnresolvedReviewThreads,
};

#[test]
fn evidence_index_markdown_shows_missing_required_checks_without_raw_payloads() {
    let report = build_evidence_index_report_from_public_summaries_at(
        launch_summary_with_missing_required_checks(),
        governance_summary(),
        1,
    );

    let markdown = render_markdown(&report);

    assert!(markdown.contains("- Missing required checks: `2`"));
    assert!(
        markdown
            .contains("  - PR #198: `verify`, `live-qa [redacted-local-path] [redacted-secret]`")
    );
    assert!(!markdown.contains("statusCheckRollup"));
    assert!(!markdown.contains("pendingNames"));
    assert!(!markdown.contains("failedNames"));
    assert!(!markdown.contains("/Users/example"));
    assert!(!markdown.contains("npm_SECRET"));
}

#[test]
fn evidence_index_markdown_shows_unresolved_review_threads_without_raw_text() {
    let mut launch = launch_summary_with_missing_required_checks();
    launch.launch_stack.unresolved_review_thread_count = 2;
    launch.launch_stack.unresolved_review_threads =
        vec![LaunchReadinessPublicUnresolvedReviewThreads {
            pull_request: 198,
            count: 2,
        }];
    let report =
        build_evidence_index_report_from_public_summaries_at(launch, governance_summary(), 1);

    let markdown = render_markdown(&report);

    assert!(markdown.contains("- Unresolved review threads: `2`"));
    assert!(markdown.contains("  - PR #198: `2`"));
    assert!(!markdown.contains("reviewThreads"));
    assert!(!markdown.contains("comments"));
    assert!(!markdown.contains("body"));
}

fn launch_summary_with_missing_required_checks() -> LaunchReadinessPublicSummary {
    LaunchReadinessPublicSummary {
        schema_version: 1,
        generated_at_unix_seconds: 1,
        result: LaunchJudgeResult::NoGo,
        repository: Some("example/repo".to_string()),
        read_only: true,
        launch_stack: LaunchReadinessPublicStack {
            result: LaunchJudgeResult::NoGo,
            from_pr: Some(198),
            stopped_at_base: Some("main".to_string()),
            pull_request_count: 1,
            pull_request_order: vec![198],
            pull_request_status: LaunchReadinessPublicStatusCounts {
                passed: 0,
                waived: 0,
                warning: 0,
                failed: 1,
            },
            unresolved_review_thread_count: 0,
            unresolved_review_threads: Vec::new(),
            missing_required_check_count: 2,
            missing_required_checks: vec![LaunchReadinessPublicMissingRequiredCheck {
                pull_request: 198,
                names: vec![
                    "verify".to_string(),
                    "live-qa /Users/example npm_SECRET".to_string(),
                ],
            }],
            blocker_issues: Vec::new(),
        },
        terminal_evidence: LaunchReadinessPublicTerminalEvidence {
            issue: None,
            report_count: 0,
            platforms: Vec::new(),
            issues: Vec::new(),
        },
        terminal_evidence_waiver: None,
        findings: Vec::new(),
        next_actions: Vec::new(),
    }
}

fn governance_summary() -> GovernanceAuditPublicSummary {
    GovernanceAuditPublicSummary {
        schema_version: 1,
        status: GovernanceAuditStatus::Passed,
        read_only: true,
        categories: Vec::new(),
        deterministic_gates: GovernanceAuditPublicGateSummary {
            count: 0,
            required_for_release_count: 0,
            names: Vec::new(),
        },
        smoke_evidence: GovernanceAuditPublicGateSummary {
            count: 0,
            required_for_release_count: 0,
            names: Vec::new(),
        },
        memory: GovernanceAuditPublicMemorySummary {
            proposal_count: 0,
            safe_to_store_count: 0,
            unsafe_proposal_count: 0,
        },
        mcp_review: None,
        finding_counts: GovernanceAuditPublicFindingCounts {
            info: 0,
            warnings: 0,
            errors: 0,
        },
        findings: Vec::new(),
        next_actions: Vec::new(),
    }
}
