use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::session::{ApprovalStatus, TuiSession};

pub(crate) const PROMOTION_REVIEW_GATES: &[&str] = &[
    "review_implementation_against_contract",
    "review_repo_structure",
    "review_agent_final_response",
    "review_agent_session",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromotionReceipt {
    pub promoted_at: u64,
    pub decision: String,
    pub reason: Option<String>,
    pub promoted_files: Vec<String>,
    pub changed_files: Vec<Value>,
    pub review_gates: BTreeMap<String, PromotionReviewGateReceipt>,
    pub verification: BTreeMap<String, String>,
    pub adapter_run_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromotionReviewGateReceipt {
    pub present: bool,
    pub status: Option<String>,
    pub valid: Option<bool>,
    pub errors: Option<u64>,
    pub warnings: Option<u64>,
}

pub(crate) fn build_promotion_receipt(
    session: &TuiSession,
    promoted_files: &[PathBuf],
    promoted_at: u64,
) -> PromotionReceipt {
    let review_gates = PROMOTION_REVIEW_GATES
        .iter()
        .map(|gate| {
            (
                (*gate).to_string(),
                review_gate_receipt(session.gates.get(*gate)),
            )
        })
        .collect();

    PromotionReceipt {
        promoted_at,
        decision: promotion_decision(&session.approval_status).to_string(),
        reason: session.approval_reason.clone(),
        promoted_files: promoted_files
            .iter()
            .map(|path| normalized_path(path))
            .collect(),
        changed_files: session.changed_files.clone(),
        review_gates,
        verification: session.verification.clone(),
        adapter_run_issues: session.adapter_run_issues.clone(),
    }
}

pub fn promotion_receipt_lines(session: &TuiSession) -> Vec<String> {
    let Some(receipt) = &session.promotion_receipt else {
        return vec![
            "promotion receipt: missing".to_string(),
            "next: promote approved changes before inspecting a receipt".to_string(),
        ];
    };
    let mut lines = vec![
        "promotion receipt: recorded".to_string(),
        format!("decision: {}", receipt.decision),
        format!(
            "reason: {}",
            receipt.reason.as_deref().unwrap_or("not recorded")
        ),
        format!("promoted files: {}", receipt.promoted_files.len()),
    ];
    lines.extend(
        receipt
            .promoted_files
            .iter()
            .take(8)
            .map(|path| format!("  file: {path}")),
    );
    if receipt.promoted_files.len() > 8 {
        lines.push(format!(
            "  ... {} more file(s)",
            receipt.promoted_files.len() - 8
        ));
    }
    lines.push(format!(
        "changed-file evidence: {}",
        receipt.changed_files.len()
    ));
    lines.push("review gates:".to_string());
    lines.extend(
        receipt
            .review_gates
            .iter()
            .map(|(name, gate)| format!("  {name}: {}", compact_gate_state(gate))),
    );
    lines.push("verification:".to_string());
    if receipt.verification.is_empty() {
        lines.push("  none recorded".to_string());
    } else {
        lines.extend(
            receipt
                .verification
                .iter()
                .map(|(check, status)| format!("  {check}={status}")),
        );
    }
    lines.push(format!(
        "adapter issues: {}",
        receipt.adapter_run_issues.len()
    ));
    lines
}

fn compact_gate_state(gate: &PromotionReviewGateReceipt) -> String {
    if !gate.present {
        return "missing".to_string();
    }
    let mut parts = vec!["present".to_string()];
    if let Some(status) = &gate.status {
        parts.push(format!("status={status}"));
    }
    if let Some(valid) = gate.valid {
        parts.push(format!("valid={valid}"));
    }
    if let Some(errors) = gate.errors {
        parts.push(format!("errors={errors}"));
    }
    if let Some(warnings) = gate.warnings {
        parts.push(format!("warnings={warnings}"));
    }
    parts.join(" ")
}

fn promotion_decision(status: &ApprovalStatus) -> &'static str {
    match status {
        ApprovalStatus::Approved => "approved",
        ApprovalStatus::Override => "override",
        ApprovalStatus::Promoted => "promoted",
        ApprovalStatus::Rejected => "rejected",
        ApprovalStatus::Pending => "pending",
    }
}

fn review_gate_receipt(review: Option<&Value>) -> PromotionReviewGateReceipt {
    PromotionReviewGateReceipt {
        present: review.is_some(),
        status: review.and_then(review_status),
        valid: review.and_then(|value| value.get("valid").and_then(Value::as_bool)),
        errors: review.and_then(review_errors),
        warnings: review.and_then(review_warnings),
    }
}

fn review_status(review: &Value) -> Option<String> {
    review
        .get("status")
        .or_else(|| review.pointer("/report/gate/status"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn review_errors(review: &Value) -> Option<u64> {
    review
        .pointer("/summary/errors")
        .or_else(|| review.pointer("/report/summary/errors"))
        .and_then(Value::as_u64)
}

fn review_warnings(review: &Value) -> Option<u64> {
    review
        .pointer("/summary/warnings")
        .or_else(|| review.pointer("/report/summary/warnings"))
        .and_then(Value::as_u64)
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
