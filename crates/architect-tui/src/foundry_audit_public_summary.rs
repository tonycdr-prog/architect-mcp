use std::collections::BTreeMap;

use serde::Serialize;

use crate::foundry_audit_report::{
    FoundryAuditDecision, FoundryAuditPublicSafety, FoundryAuditReport, FoundryAuditStatus,
};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicSummary {
    pub schema_version: u8,
    pub status: FoundryAuditStatus,
    pub read_only: bool,
    pub mutation_boundary: String,
    pub approval_required_before_mutation: bool,
    pub server_writes_performed: u64,
    pub files_reviewed: Option<u64>,
    pub scan_truncated: bool,
    pub constitution: FoundryAuditPublicConstitution,
    pub evidence: FoundryAuditPublicEvidence,
    pub ledger: FoundryAuditPublicLedger,
    pub forge: FoundryAuditPublicForge,
    pub decisions: Vec<FoundryAuditPublicDecision>,
    pub public_safety: FoundryAuditPublicSafety,
    pub next_actions: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicConstitution {
    pub hard_signals: u64,
    pub advisory_signals: u64,
    pub warnings: u64,
    pub pr_templates: u64,
    pub ci_workflows: u64,
    pub repo_shape: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicEvidence {
    pub total_evidence: u64,
    pub redacted: u64,
    pub omitted_raw_payloads: u64,
    pub suppression_candidates: u64,
    pub coverage_caveats: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicLedger {
    pub total_entries: u64,
    pub by_route: BTreeMap<String, u64>,
    pub approval_required: u64,
    pub human_required: u64,
    pub public_safety_holds: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicForge {
    pub previews_generated: u64,
    pub pull_request_previews: u64,
    pub architect_issue_previews: u64,
    pub exception_records: u64,
    pub no_op_records: u64,
    pub human_questions: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicDecision {
    pub route: String,
    pub score: u64,
    pub evidence_count: usize,
    pub risk: String,
    pub approval_state: String,
    pub next_action: String,
    pub preview_kind: Option<String>,
    pub warning_count: usize,
}

pub(crate) fn build_public_summary(report: &FoundryAuditReport) -> FoundryAuditPublicSummary {
    FoundryAuditPublicSummary {
        schema_version: 1,
        status: report.status.clone(),
        read_only: report.read_only,
        mutation_boundary: audit_public_text(&report.mutation_boundary, 120),
        approval_required_before_mutation: report.approval_required_before_mutation,
        server_writes_performed: report.server_writes_performed,
        files_reviewed: report.mcp.files_reviewed,
        scan_truncated: report.mcp.scan_truncated,
        constitution: FoundryAuditPublicConstitution {
            hard_signals: report.constitution.hard_signals,
            advisory_signals: report.constitution.advisory_signals,
            warnings: report.constitution.warnings,
            pr_templates: report.constitution.pr_templates,
            ci_workflows: report.constitution.ci_workflows,
            repo_shape: report
                .constitution
                .repo_shape
                .as_deref()
                .map(|shape| audit_public_text(shape, 80)),
        },
        evidence: FoundryAuditPublicEvidence {
            total_evidence: report.evidence.total_evidence,
            redacted: report.evidence.redacted,
            omitted_raw_payloads: report.evidence.omitted_raw_payloads,
            suppression_candidates: report.evidence.suppression_candidates,
            coverage_caveats: report.evidence.coverage_caveats,
        },
        ledger: FoundryAuditPublicLedger {
            total_entries: report.ledger.total_entries,
            by_route: report.ledger.by_route.clone(),
            approval_required: report.ledger.approval_required,
            human_required: report.ledger.human_required,
            public_safety_holds: report.ledger.public_safety_holds,
        },
        forge: FoundryAuditPublicForge {
            previews_generated: report.forge.previews_generated,
            pull_request_previews: report.forge.pull_request_previews,
            architect_issue_previews: report.forge.architect_issue_previews,
            exception_records: report.forge.exception_records,
            no_op_records: report.forge.no_op_records,
            human_questions: report.forge.human_questions,
        },
        decisions: report.decisions.iter().map(public_decision).collect(),
        public_safety: report.public_safety.clone(),
        next_actions: public_next_actions(&report.decisions),
        error: report
            .error
            .as_deref()
            .map(|error| audit_public_text(error, 240)),
    }
}

fn public_decision(decision: &FoundryAuditDecision) -> FoundryAuditPublicDecision {
    FoundryAuditPublicDecision {
        route: audit_public_text(&decision.route, 80),
        score: decision.score,
        evidence_count: decision.evidence_count,
        risk: audit_public_text(&decision.risk, 80),
        approval_state: audit_public_text(&decision.approval_state, 80),
        next_action: audit_public_text(&decision.next_action, 240),
        preview_kind: decision
            .preview_kind
            .as_deref()
            .map(|kind| audit_public_text(kind, 80)),
        warning_count: decision.warning_count,
    }
}

fn public_next_actions(decisions: &[FoundryAuditDecision]) -> Vec<String> {
    let mut actions = Vec::new();
    for decision in decisions {
        let action = audit_public_text(&decision.next_action, 240);
        if !actions.contains(&action) {
            actions.push(action);
        }
    }
    if actions.is_empty() {
        actions.push("rerun foundry-audit after MCP evidence is available".to_string());
    }
    actions
}

fn audit_public_text(value: &str, max_len: usize) -> String {
    public_text(value, max_len)
        .replace("stdout:", "[omitted-raw-output]:")
        .replace("stderr:", "[omitted-raw-output]:")
        .replace("STDOUT:", "[omitted-raw-output]:")
        .replace("STDERR:", "[omitted-raw-output]:")
}
