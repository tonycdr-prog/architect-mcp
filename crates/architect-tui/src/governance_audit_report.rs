use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceAuditStatus {
    Passed,
    PassedWithWarnings,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceFindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditReport {
    pub schema_version: u8,
    pub status: GovernanceAuditStatus,
    pub workspace: String,
    pub read_only: bool,
    pub categories: Vec<GovernanceAuditCategory>,
    pub deterministic_gates: Vec<GovernanceGateEvidence>,
    pub smoke_evidence: Vec<GovernanceGateEvidence>,
    pub memory_proposals: Vec<GovernanceMemoryProposal>,
    pub mcp_review: Option<GovernanceMcpReview>,
    pub findings: Vec<GovernanceFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditCategory {
    pub name: String,
    pub status: GovernanceAuditStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceFinding {
    pub severity: GovernanceFindingSeverity,
    pub category: String,
    pub code: String,
    pub message: String,
    pub evidence: String,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceGateEvidence {
    pub name: String,
    pub kind: String,
    pub command: String,
    pub required_for_release: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceMemoryProposal {
    pub scope: String,
    pub source: String,
    pub text: String,
    pub safe_to_store: bool,
    pub review_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceMcpReview {
    pub status: String,
    pub gate_status: Option<String>,
    pub files_reviewed: Option<u64>,
    pub errors: Option<u64>,
    pub warnings: Option<u64>,
    pub violation_count: Option<usize>,
    pub detail: String,
}

pub(crate) fn print_text_report(report: &GovernanceAuditReport) {
    println!("architect-mcp-tui governance audit: {:?}", report.status);
    println!("workspace: {}", report.workspace);
    println!("read-only: {}", report.read_only);
    for category in &report.categories {
        println!(
            "- {}: {:?} - {}",
            category.name, category.status, category.summary
        );
    }
    if let Some(mcp_review) = &report.mcp_review {
        println!(
            "mcp review: {} gate={}",
            mcp_review.status,
            mcp_review.gate_status.as_deref().unwrap_or("unknown")
        );
    }
    for finding in &report.findings {
        println!(
            "- {:?} {}: {}",
            finding.severity, finding.code, finding.message
        );
    }
}
