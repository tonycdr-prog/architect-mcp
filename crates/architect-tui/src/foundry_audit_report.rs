use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::foundry_audit_report_values::{
    actionability_summary, constitution_summary, decisions, evidence_summary, forge_summary,
    ledger_summary, public_safety_summary,
};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FoundryAuditStatus {
    Passed,
    PassedWithWarnings,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditReport {
    pub schema_version: u8,
    pub status: FoundryAuditStatus,
    pub workspace: String,
    pub read_only: bool,
    pub mutation_boundary: String,
    pub approval_required_before_mutation: bool,
    pub server_writes_performed: u64,
    pub mcp: FoundryAuditMcpSummary,
    pub constitution: FoundryAuditConstitutionSummary,
    pub evidence: FoundryAuditEvidenceSummary,
    pub actionability: FoundryAuditActionabilitySummary,
    pub ledger: FoundryAuditLedgerSummary,
    pub forge: FoundryAuditForgeSummary,
    pub decisions: Vec<FoundryAuditDecision>,
    pub public_safety: FoundryAuditPublicSafety,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditMcpSummary {
    pub tools_called: Vec<String>,
    pub files_reviewed: Option<u64>,
    pub scan_truncated: bool,
    pub review_gate_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditConstitutionSummary {
    pub files_reviewed: Option<u64>,
    pub hard_signals: u64,
    pub advisory_signals: u64,
    pub warnings: u64,
    pub pr_templates: u64,
    pub ci_workflows: u64,
    pub repo_shape: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditEvidenceSummary {
    pub total_evidence: u64,
    pub redacted: u64,
    pub omitted_raw_payloads: u64,
    pub suppression_candidates: u64,
    pub coverage_caveats: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditActionabilitySummary {
    pub total_findings: u64,
    pub pr_preview_candidates: u64,
    pub ask_human: u64,
    pub exception_candidates: u64,
    pub no_op_candidates: u64,
    pub public_safety_holds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditLedgerSummary {
    pub total_entries: u64,
    pub by_route: BTreeMap<String, u64>,
    pub approval_required: u64,
    pub human_required: u64,
    pub public_safety_holds: u64,
    pub server_writes_performed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditForgeSummary {
    pub previews_generated: u64,
    pub pull_request_previews: u64,
    pub architect_issue_previews: u64,
    pub exception_records: u64,
    pub no_op_records: u64,
    pub human_questions: u64,
    pub server_writes_performed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditDecision {
    pub id: String,
    pub route: String,
    pub score: u64,
    pub evidence_count: usize,
    pub risk: String,
    pub approval_state: String,
    pub next_action: String,
    pub preview_kind: Option<String>,
    pub preview_title: Option<String>,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FoundryAuditPublicSafety {
    pub raw_payloads_included: bool,
    pub raw_repo_content_included: bool,
    pub local_paths_included: bool,
    pub token_values_included: bool,
    pub mutation_allowed: bool,
}

pub(crate) fn report_from_mcp_values(
    workspace: String,
    review: Value,
    constitution: Value,
    inventory: Value,
    actionability: Value,
    ledger: Value,
    forge: Value,
) -> FoundryAuditReport {
    let mcp = FoundryAuditMcpSummary {
        tools_called: vec![
            "review_local_workspace".to_string(),
            "derive_local_repo_constitution".to_string(),
            "normalize_foundry_evidence".to_string(),
            "score_foundry_actionability".to_string(),
            "route_foundry_decisions".to_string(),
            "forge_foundry_previews".to_string(),
        ],
        files_reviewed: review.get("filesReviewed").and_then(Value::as_u64),
        scan_truncated: review
            .pointer("/scan/truncated")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        review_gate_status: review
            .pointer("/report/gate/status")
            .and_then(Value::as_str)
            .map(|status| public_text(status, 80)),
    };
    let constitution_summary = constitution_summary(&constitution);
    let evidence = evidence_summary(&inventory);
    let actionability_summary = actionability_summary(&actionability);
    let ledger_summary = ledger_summary(&ledger);
    let forge_summary = forge_summary(&forge);
    let decisions = decisions(&ledger, &forge);
    let public_safety = public_safety_summary([&inventory, &ledger, &forge]);
    let server_writes_performed =
        ledger_summary.server_writes_performed + forge_summary.server_writes_performed;
    let status = if server_writes_performed > 0
        || public_safety.raw_payloads_included
        || public_safety.raw_repo_content_included
        || public_safety.local_paths_included
        || public_safety.token_values_included
        || public_safety.mutation_allowed
    {
        FoundryAuditStatus::Failed
    } else if mcp.scan_truncated
        || ledger_summary.human_required > 0
        || ledger_summary.public_safety_holds > 0
        || evidence.redacted > 0
        || actionability_summary.public_safety_holds > 0
    {
        FoundryAuditStatus::PassedWithWarnings
    } else {
        FoundryAuditStatus::Passed
    };

    FoundryAuditReport {
        schema_version: 1,
        status,
        workspace,
        read_only: true,
        mutation_boundary: "preview_only_no_repository_mutation".to_string(),
        approval_required_before_mutation: true,
        server_writes_performed,
        mcp,
        constitution: constitution_summary,
        evidence,
        actionability: actionability_summary,
        ledger: ledger_summary,
        forge: forge_summary,
        decisions,
        public_safety,
        error: None,
    }
}

pub(crate) fn failed_report(
    workspace: String,
    tools_called: Vec<String>,
    error: String,
) -> FoundryAuditReport {
    FoundryAuditReport {
        schema_version: 1,
        status: FoundryAuditStatus::Failed,
        workspace,
        read_only: true,
        mutation_boundary: "preview_only_no_repository_mutation".to_string(),
        approval_required_before_mutation: true,
        server_writes_performed: 0,
        mcp: FoundryAuditMcpSummary {
            tools_called,
            ..FoundryAuditMcpSummary::default()
        },
        constitution: FoundryAuditConstitutionSummary::default(),
        evidence: FoundryAuditEvidenceSummary::default(),
        actionability: FoundryAuditActionabilitySummary::default(),
        ledger: FoundryAuditLedgerSummary::default(),
        forge: FoundryAuditForgeSummary::default(),
        decisions: Vec::new(),
        public_safety: FoundryAuditPublicSafety::default(),
        error: Some(public_text(&error, 240)),
    }
}

pub(crate) fn print_text_report(report: &FoundryAuditReport) {
    println!("architect-mcp-tui foundry audit: {:?}", report.status);
    println!("read-only: {}", report.read_only);
    println!("mutation boundary: {}", report.mutation_boundary);
    println!("files reviewed: {}", display_opt(report.mcp.files_reviewed));
    println!(
        "constitution: hard={} advisory={} warnings={} pr_templates={} ci_workflows={}",
        report.constitution.hard_signals,
        report.constitution.advisory_signals,
        report.constitution.warnings,
        report.constitution.pr_templates,
        report.constitution.ci_workflows
    );
    println!(
        "ledger: entries={} approval_required={} human_required={} server_writes={}",
        report.ledger.total_entries,
        report.ledger.approval_required,
        report.ledger.human_required,
        report.ledger.server_writes_performed
    );
    println!(
        "forge previews: total={} pr={} architect_issue={} exception={} no_op={} human={}",
        report.forge.previews_generated,
        report.forge.pull_request_previews,
        report.forge.architect_issue_previews,
        report.forge.exception_records,
        report.forge.no_op_records,
        report.forge.human_questions
    );
    for decision in &report.decisions {
        println!(
            "- {} score={} evidence={} risk={} preview={} next={}",
            decision.route,
            decision.score,
            decision.evidence_count,
            decision.risk,
            decision.preview_kind.as_deref().unwrap_or("none"),
            decision.next_action
        );
    }
}

fn display_opt(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
