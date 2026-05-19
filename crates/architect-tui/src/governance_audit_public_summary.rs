use serde::Serialize;

use crate::governance_audit_report::{
    GovernanceAuditReport, GovernanceAuditStatus, GovernanceFinding, GovernanceFindingSeverity,
    GovernanceGateEvidence, GovernanceMcpReview,
};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicSummary {
    pub schema_version: u8,
    pub status: GovernanceAuditStatus,
    pub read_only: bool,
    pub categories: Vec<GovernanceAuditPublicCategory>,
    pub deterministic_gates: GovernanceAuditPublicGateSummary,
    pub smoke_evidence: GovernanceAuditPublicGateSummary,
    pub memory: GovernanceAuditPublicMemorySummary,
    pub mcp_review: Option<GovernanceAuditPublicMcpReview>,
    pub finding_counts: GovernanceAuditPublicFindingCounts,
    pub findings: Vec<GovernanceAuditPublicFinding>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicCategory {
    pub name: String,
    pub status: GovernanceAuditStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicGateSummary {
    pub count: usize,
    pub required_for_release_count: usize,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicMemorySummary {
    pub proposal_count: usize,
    pub safe_to_store_count: usize,
    pub unsafe_proposal_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicMcpReview {
    pub status: String,
    pub gate_status: Option<String>,
    pub files_reviewed: Option<u64>,
    pub errors: Option<u64>,
    pub warnings: Option<u64>,
    pub violation_count: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicFindingCounts {
    pub info: usize,
    pub warnings: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditPublicFinding {
    pub severity: GovernanceFindingSeverity,
    pub category: String,
    pub code: String,
    pub message: String,
    pub next_action: String,
}

pub(crate) fn build_public_summary(report: &GovernanceAuditReport) -> GovernanceAuditPublicSummary {
    GovernanceAuditPublicSummary {
        schema_version: 1,
        status: report.status.clone(),
        read_only: report.read_only,
        categories: report
            .categories
            .iter()
            .map(|category| GovernanceAuditPublicCategory {
                name: public_text(&category.name, 120),
                status: category.status.clone(),
                summary: public_text(&category.summary, 240),
            })
            .collect(),
        deterministic_gates: gate_summary(&report.deterministic_gates),
        smoke_evidence: gate_summary(&report.smoke_evidence),
        memory: GovernanceAuditPublicMemorySummary {
            proposal_count: report.memory_proposals.len(),
            safe_to_store_count: report
                .memory_proposals
                .iter()
                .filter(|proposal| proposal.safe_to_store)
                .count(),
            unsafe_proposal_count: report
                .memory_proposals
                .iter()
                .filter(|proposal| !proposal.safe_to_store)
                .count(),
        },
        mcp_review: report.mcp_review.as_ref().map(public_mcp_review),
        finding_counts: finding_counts(&report.findings),
        findings: report.findings.iter().map(public_finding).collect(),
        next_actions: next_actions(&report.findings),
    }
}

fn gate_summary(gates: &[GovernanceGateEvidence]) -> GovernanceAuditPublicGateSummary {
    GovernanceAuditPublicGateSummary {
        count: gates.len(),
        required_for_release_count: gates
            .iter()
            .filter(|gate| gate.required_for_release)
            .count(),
        names: gates
            .iter()
            .map(|gate| public_text(&gate.name, 120))
            .collect(),
    }
}

fn public_mcp_review(review: &GovernanceMcpReview) -> GovernanceAuditPublicMcpReview {
    GovernanceAuditPublicMcpReview {
        status: public_text(&review.status, 80),
        gate_status: review
            .gate_status
            .as_deref()
            .map(|status| public_text(status, 80)),
        files_reviewed: review.files_reviewed,
        errors: review.errors,
        warnings: review.warnings,
        violation_count: review.violation_count,
    }
}

fn finding_counts(findings: &[GovernanceFinding]) -> GovernanceAuditPublicFindingCounts {
    let mut counts = GovernanceAuditPublicFindingCounts::default();
    for finding in findings {
        match finding.severity {
            GovernanceFindingSeverity::Info => counts.info += 1,
            GovernanceFindingSeverity::Warning => counts.warnings += 1,
            GovernanceFindingSeverity::Error => counts.errors += 1,
        }
    }
    counts
}

fn public_finding(finding: &GovernanceFinding) -> GovernanceAuditPublicFinding {
    GovernanceAuditPublicFinding {
        severity: finding.severity.clone(),
        category: public_text(&finding.category, 120),
        code: public_text(&finding.code, 120),
        message: public_text(&finding.message, 240),
        next_action: public_text(&finding.next_action, 240),
    }
}

fn next_actions(findings: &[GovernanceFinding]) -> Vec<String> {
    let mut actions = Vec::new();
    for finding in findings {
        let action = public_text(&finding.next_action, 240);
        if !actions.contains(&action) {
            actions.push(action);
        }
    }
    actions
}
