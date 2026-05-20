use serde_json::Value;

use crate::foundry_audit_report::{
    FoundryAuditActionabilitySummary, FoundryAuditConstitutionSummary, FoundryAuditDecision,
    FoundryAuditEvidenceSummary, FoundryAuditForgeSummary, FoundryAuditLedgerSummary,
    FoundryAuditPublicSafety,
};
use crate::launch_stack_github::public_text;

pub(crate) fn constitution_summary(value: &Value) -> FoundryAuditConstitutionSummary {
    let constitution = value.get("constitution").unwrap_or(value);
    FoundryAuditConstitutionSummary {
        files_reviewed: value.get("filesReviewed").and_then(Value::as_u64),
        hard_signals: value_u64(constitution, "/summary/hardSignals"),
        advisory_signals: value_u64(constitution, "/summary/advisorySignals"),
        warnings: value_u64(constitution, "/summary/warnings"),
        pr_templates: array_len(constitution, "/pullRequests/templates"),
        ci_workflows: array_len(constitution, "/ci/workflows"),
        repo_shape: constitution
            .pointer("/summary/repoShape")
            .and_then(Value::as_str)
            .map(|shape| public_text(shape, 80)),
    }
}

pub(crate) fn evidence_summary(value: &Value) -> FoundryAuditEvidenceSummary {
    let summary = value.get("summary").unwrap_or(value);
    FoundryAuditEvidenceSummary {
        total_evidence: field_u64(summary, "totalEvidence"),
        redacted: field_u64(summary, "redacted"),
        omitted_raw_payloads: field_u64(summary, "omittedRawPayloads"),
        suppression_candidates: field_u64(summary, "suppressionCandidates"),
        coverage_caveats: field_u64(summary, "coverageCaveats"),
    }
}

pub(crate) fn actionability_summary(value: &Value) -> FoundryAuditActionabilitySummary {
    let summary = value.get("summary").unwrap_or(value);
    FoundryAuditActionabilitySummary {
        total_findings: field_u64(summary, "totalFindings"),
        pr_preview_candidates: field_u64(summary, "prPreviewCandidates"),
        ask_human: field_u64(summary, "askHuman"),
        exception_candidates: field_u64(summary, "exceptionCandidates"),
        no_op_candidates: field_u64(summary, "noOpCandidates"),
        public_safety_holds: field_u64(summary, "publicSafetyHolds"),
    }
}

pub(crate) fn ledger_summary(value: &Value) -> FoundryAuditLedgerSummary {
    let summary = value.get("summary").unwrap_or(value);
    FoundryAuditLedgerSummary {
        total_entries: field_u64(summary, "totalEntries"),
        by_route: summary
            .get("byRoute")
            .and_then(Value::as_object)
            .map(|routes| {
                routes
                    .iter()
                    .filter_map(|(route, count)| {
                        count.as_u64().map(|count| (public_text(route, 80), count))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        approval_required: field_u64(summary, "approvalRequired"),
        human_required: field_u64(summary, "humanRequired"),
        public_safety_holds: field_u64(summary, "publicSafetyHolds"),
        server_writes_performed: field_u64(summary, "serverWritesPerformed"),
    }
}

pub(crate) fn forge_summary(value: &Value) -> FoundryAuditForgeSummary {
    let summary = value.get("summary").unwrap_or(value);
    FoundryAuditForgeSummary {
        previews_generated: field_u64(summary, "previewsGenerated"),
        pull_request_previews: field_u64(summary, "pullRequestPreviews"),
        architect_issue_previews: field_u64(summary, "architectIssuePreviews"),
        exception_records: field_u64(summary, "exceptionRecords"),
        no_op_records: field_u64(summary, "noOpRecords"),
        human_questions: field_u64(summary, "humanQuestions"),
        server_writes_performed: field_u64(summary, "serverWritesPerformed"),
    }
}

pub(crate) fn decisions(ledger: &Value, forge: &Value) -> Vec<FoundryAuditDecision> {
    let previews = forge
        .get("previews")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    ledger
        .get("entries")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .map(|entry| decision(entry, &previews))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn public_safety_summary(values: [&Value; 3]) -> FoundryAuditPublicSafety {
    let mut safety = FoundryAuditPublicSafety::default();
    for value in values {
        let Some(public_safety) = value.get("publicSafety") else {
            safety.raw_payloads_included = true;
            safety.raw_repo_content_included = true;
            safety.local_paths_included = true;
            safety.token_values_included = true;
            safety.mutation_allowed = true;
            continue;
        };
        safety.raw_payloads_included |= bool_field_aliases(public_safety, &["rawPayloadsIncluded"]);
        safety.raw_repo_content_included |= bool_field_aliases(
            public_safety,
            &["rawRepoContentIncluded", "rawContentIncluded"],
        );
        safety.local_paths_included |= bool_field_aliases(public_safety, &["localPathsIncluded"]);
        safety.token_values_included |= bool_field_aliases(public_safety, &["tokenValuesIncluded"]);
        safety.mutation_allowed |= bool_field_aliases(public_safety, &["mutationAllowed"]);
    }
    safety
}

fn decision(entry: &Value, previews: &[Value]) -> FoundryAuditDecision {
    let id = entry.get("id").and_then(Value::as_str).unwrap_or("unknown");
    let preview = previews.iter().find(|preview| {
        preview
            .get("sourceDecisionId")
            .and_then(Value::as_str)
            .is_some_and(|source| source == id)
    });
    FoundryAuditDecision {
        id: public_text(id, 120),
        route: public_text(
            entry
                .get("route")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            80,
        ),
        score: entry
            .pointer("/source/score")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        evidence_count: entry
            .get("evidenceIds")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
        risk: public_text(
            entry
                .get("redactionState")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            80,
        ),
        approval_state: public_text(
            entry
                .get("approvalState")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            80,
        ),
        next_action: public_text(
            entry
                .get("nextAction")
                .and_then(Value::as_str)
                .unwrap_or("review foundry decision"),
            240,
        ),
        preview_kind: preview
            .and_then(|preview| preview.get("kind"))
            .and_then(Value::as_str)
            .map(|kind| public_text(kind, 80)),
        preview_title: preview
            .and_then(|preview| preview.get("title"))
            .and_then(Value::as_str)
            .map(|title| public_text(title, 160)),
        warning_count: preview
            .and_then(|preview| preview.get("warnings"))
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
    }
}

fn value_u64(value: &Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(Value::as_u64).unwrap_or(0)
}

fn array_len(value: &Value, pointer: &str) -> u64 {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(|items| items.len() as u64)
        .unwrap_or(0)
}

fn field_u64(value: &Value, name: &str) -> u64 {
    value.get(name).and_then(Value::as_u64).unwrap_or(0)
}

fn bool_field_aliases(value: &Value, names: &[&str]) -> bool {
    let mut saw_alias = false;
    let mut included = false;
    for name in names {
        match value.get(name) {
            Some(Value::Bool(current)) => {
                saw_alias = true;
                included |= *current;
            }
            Some(_) => return true,
            None => continue,
        }
    }
    if saw_alias { included } else { true }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::public_safety_summary;

    #[test]
    fn public_safety_fails_closed_when_public_safety_block_missing() {
        let safety = public_safety_summary([
            &json!({"summary": {}}),
            &json!({"publicSafety": {"rawPayloadsIncluded": false, "rawRepoContentIncluded": false, "localPathsIncluded": false, "tokenValuesIncluded": false, "mutationAllowed": false}}),
            &json!({"publicSafety": {"rawPayloadsIncluded": false, "rawRepoContentIncluded": false, "localPathsIncluded": false, "tokenValuesIncluded": false, "mutationAllowed": false}}),
        ]);

        assert!(safety.raw_payloads_included);
        assert!(safety.raw_repo_content_included);
        assert!(safety.local_paths_included);
        assert!(safety.token_values_included);
        assert!(safety.mutation_allowed);
    }

    #[test]
    fn public_safety_fails_closed_when_flags_are_missing_or_malformed() {
        let safety = public_safety_summary([
            &json!({"publicSafety": {"rawPayloadsIncluded": false, "rawRepoContentIncluded": false, "localPathsIncluded": false, "tokenValuesIncluded": false, "mutationAllowed": false}}),
            &json!({"publicSafety": {"rawPayloadsIncluded": "nope", "rawRepoContentIncluded": false, "localPathsIncluded": false, "tokenValuesIncluded": false, "mutationAllowed": false}}),
            &json!({"publicSafety": {"rawRepoContentIncluded": false, "localPathsIncluded": false, "tokenValuesIncluded": false, "mutationAllowed": false}}),
        ]);

        assert!(safety.raw_payloads_included);
        assert!(!safety.raw_repo_content_included);
        assert!(!safety.local_paths_included);
        assert!(!safety.token_values_included);
        assert!(!safety.mutation_allowed);
    }
}
