use crate::evidence_index_report::build_evidence_index_report_from_public_summaries_at;
use crate::governance_audit_public_summary::{
    GovernanceAuditPublicCategory, GovernanceAuditPublicFinding,
    GovernanceAuditPublicFindingCounts, GovernanceAuditPublicGateSummary,
    GovernanceAuditPublicMcpReview, GovernanceAuditPublicMemorySummary,
    GovernanceAuditPublicSummary,
};
use crate::governance_audit_report::{GovernanceAuditStatus, GovernanceFindingSeverity};
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness_public_summary::{
    LaunchReadinessPublicStack, LaunchReadinessPublicStatusCounts, LaunchReadinessPublicSummary,
    LaunchReadinessPublicTerminalEvidence,
};

#[test]
fn evidence_index_combines_launch_and_governance_public_summaries() {
    let report = build_evidence_index_report_from_public_summaries_at(
        launch_summary(LaunchJudgeResult::Go),
        governance_summary(GovernanceAuditStatus::PassedWithWarnings),
        1_779_000_000,
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.generated_at_unix_seconds, 1_779_000_000);
    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert!(report.read_only);
    assert_eq!(report.repository, Some("example/repo".to_string()));
    assert_eq!(report.sections.len(), 2);
    assert_eq!(report.sections[0].name, "launch readiness");
    assert_eq!(report.sections[0].status, "go");
    assert_eq!(report.sections[1].name, "governance audit");
    assert_eq!(report.sections[1].status, "passed_with_warnings");
    assert!(report.sections[0].summary.contains("2 PRs"));
    assert!(report.sections[1].summary.contains("1 categories"));
}

#[test]
fn evidence_index_fails_when_any_section_is_no_go() {
    let report = build_evidence_index_report_from_public_summaries_at(
        launch_summary(LaunchJudgeResult::Go),
        governance_summary(GovernanceAuditStatus::Failed),
        1,
    );

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(report.sections[1].result, LaunchJudgeResult::NoGo);
}

#[test]
fn evidence_index_redacts_public_text_and_omits_raw_payloads() {
    let mut launch = launch_summary(LaunchJudgeResult::ConditionalGo);
    launch.repository = Some("/Users/example/private/repo npm_SECRET".to_string());
    launch
        .findings
        .push("manual note referenced /Users/example/private and ghp_example".to_string());
    launch
        .next_actions
        .push("inspect /Users/example/private before publishing npm_SECRET".to_string());

    let report = build_evidence_index_report_from_public_summaries_at(
        launch,
        governance_summary(GovernanceAuditStatus::Passed),
        1,
    );
    let text = serde_json::to_string_pretty(&report).expect("serialize evidence index");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("/Users/example"));
    assert!(!text.contains("npm_SECRET"));
    assert!(!text.contains("ghp_example"));
    assert!(!text.contains("workspace"));
    assert!(!text.contains("commandSummary"));
    assert!(!text.contains("stdout"));
    assert!(!text.contains("stderr"));
    assert!(!text.contains("memory proposal text"));
}

fn launch_summary(result: LaunchJudgeResult) -> LaunchReadinessPublicSummary {
    LaunchReadinessPublicSummary {
        schema_version: 1,
        generated_at_unix_seconds: 1,
        result: result.clone(),
        repository: Some("example/repo".to_string()),
        read_only: true,
        launch_stack: LaunchReadinessPublicStack {
            result,
            from_pr: Some(198),
            stopped_at_base: Some("main".to_string()),
            pull_request_count: 2,
            pull_request_order: vec![150, 198],
            pull_request_status: LaunchReadinessPublicStatusCounts {
                passed: 2,
                waived: 0,
                warning: 0,
                failed: 0,
            },
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
        next_actions: vec!["collect final release evidence".to_string()],
    }
}

fn governance_summary(status: GovernanceAuditStatus) -> GovernanceAuditPublicSummary {
    let finding_severity = if status == GovernanceAuditStatus::Failed {
        GovernanceFindingSeverity::Error
    } else {
        GovernanceFindingSeverity::Warning
    };

    GovernanceAuditPublicSummary {
        schema_version: 1,
        status,
        read_only: true,
        categories: vec![GovernanceAuditPublicCategory {
            name: "release".to_string(),
            status: GovernanceAuditStatus::PassedWithWarnings,
            summary: "release evidence still needs maintainer review".to_string(),
        }],
        deterministic_gates: GovernanceAuditPublicGateSummary {
            count: 2,
            required_for_release_count: 1,
            names: vec!["release gate".to_string(), "typecheck".to_string()],
        },
        smoke_evidence: GovernanceAuditPublicGateSummary {
            count: 1,
            required_for_release_count: 0,
            names: vec!["tui live qa".to_string()],
        },
        memory: GovernanceAuditPublicMemorySummary {
            proposal_count: 0,
            safe_to_store_count: 0,
            unsafe_proposal_count: 0,
        },
        mcp_review: Some(GovernanceAuditPublicMcpReview {
            status: "passed".to_string(),
            gate_status: Some("pass".to_string()),
            files_reviewed: Some(42),
            errors: Some(0),
            warnings: Some(1),
            violation_count: Some(0),
        }),
        finding_counts: GovernanceAuditPublicFindingCounts {
            info: 0,
            warnings: usize::from(finding_severity == GovernanceFindingSeverity::Warning),
            errors: usize::from(finding_severity == GovernanceFindingSeverity::Error),
        },
        findings: vec![GovernanceAuditPublicFinding {
            severity: finding_severity,
            category: "release".to_string(),
            code: "GOV_RELEASE_EVIDENCE".to_string(),
            message: "release evidence needs maintainer review".to_string(),
            next_action: "collect final release evidence".to_string(),
        }],
        next_actions: vec!["collect final release evidence".to_string()],
    }
}
