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

pub(crate) fn launch_summary(result: LaunchJudgeResult) -> LaunchReadinessPublicSummary {
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
            unresolved_review_thread_count: 0,
            unresolved_review_threads: Vec::new(),
            missing_required_check_count: 0,
            missing_required_checks: Vec::new(),
            blocker_issues: Vec::new(),
        },
        terminal_evidence: LaunchReadinessPublicTerminalEvidence {
            issue: None,
            report_count: 0,
            platforms: Vec::new(),
            reports: Vec::new(),
            issues: Vec::new(),
        },
        terminal_evidence_waiver: None,
        findings: Vec::new(),
        next_actions: vec!["collect final release evidence".to_string()],
    }
}

pub(crate) fn governance_summary(status: GovernanceAuditStatus) -> GovernanceAuditPublicSummary {
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
