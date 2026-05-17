use anyhow::{Result, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FoundrySmokeRetentionDecision {
    Retain,
    DeleteLater,
    ManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokeRetentionDecisionRecord {
    pub decision: String,
    pub reason: String,
    pub deletion_performed: bool,
}

impl FoundrySmokeRetentionDecision {
    pub(crate) fn public_id(self) -> &'static str {
        match self {
            Self::Retain => "retained_for_evidence",
            Self::DeleteLater => "delete_later_requested",
            Self::ManualReview => "manual_review_needed",
        }
    }

    fn retention_text(self, reason: &str) -> String {
        match self {
            Self::Retain => format!("retained for maintainer evidence: {reason}"),
            Self::DeleteLater => format!(
                "delete-later requested after review: {reason}; no repository deletion was performed by foundry-smoke"
            ),
            Self::ManualReview => format!(
                "manual retention review needed: {reason}; no repository deletion was performed by foundry-smoke"
            ),
        }
    }
}

pub(crate) fn retention_from_options(
    execute: bool,
    decision: Option<FoundrySmokeRetentionDecision>,
    reason: Option<&str>,
) -> Result<(String, Option<FoundrySmokeRetentionDecisionRecord>)> {
    match (decision, reason) {
        (None, Some(reason)) if !normalize_reason(reason).is_empty() => {
            bail!("--retention-reason requires --retention-decision")
        }
        (None, _) => Ok((default_retention_text(execute), None)),
        (Some(decision), Some(reason)) => {
            let reason = normalize_reason(reason);
            if reason.is_empty() {
                bail!("--retention-reason must be non-empty when --retention-decision is set");
            }
            let retention = decision.retention_text(&reason);
            Ok((
                retention,
                Some(FoundrySmokeRetentionDecisionRecord {
                    decision: decision.public_id().to_string(),
                    reason,
                    deletion_performed: false,
                }),
            ))
        }
        (Some(_), None) => {
            bail!("--retention-decision requires --retention-reason")
        }
    }
}

fn default_retention_text(execute: bool) -> String {
    if execute {
        "private GitHub repo retained for maintainer evidence; delete manually when no longer needed"
            .to_string()
    } else {
        "no GitHub repo created".to_string()
    }
}

fn normalize_reason(reason: &str) -> String {
    reason.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_decision_requires_reason() {
        let error = retention_from_options(true, Some(FoundrySmokeRetentionDecision::Retain), None)
            .expect_err("blank decision should fail");

        assert!(
            error
                .to_string()
                .contains("--retention-decision requires --retention-reason")
        );
    }

    #[test]
    fn retention_reason_requires_decision() {
        let error = retention_from_options(true, None, Some("keep it"))
            .expect_err("orphan reason should fail");

        assert!(
            error
                .to_string()
                .contains("--retention-reason requires --retention-decision")
        );
    }

    #[test]
    fn retention_decision_rejects_blank_reason() {
        let error = retention_from_options(
            true,
            Some(FoundrySmokeRetentionDecision::ManualReview),
            Some(" \n \t "),
        )
        .expect_err("blank reason should fail");

        assert!(
            error
                .to_string()
                .contains("--retention-reason must be non-empty")
        );
    }

    #[test]
    fn retention_decision_records_no_deletion_performed() {
        let (text, record) = retention_from_options(
            true,
            Some(FoundrySmokeRetentionDecision::DeleteLater),
            Some("delete after launch review"),
        )
        .expect("valid decision");
        let record = record.expect("decision record");

        assert!(text.contains("delete-later requested"));
        assert_eq!(record.decision, "delete_later_requested");
        assert_eq!(record.reason, "delete after launch review");
        assert!(!record.deletion_performed);
    }
}
