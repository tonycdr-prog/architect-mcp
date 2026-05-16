use anyhow::Result;
use serde_json::Value;

use crate::arena::{ArenaCandidateInput, ArenaReviewStatus, rank_arena_candidates};
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};
use crate::session::TuiSession;

impl InteractiveWorkflowEngine {
    pub(crate) fn arena_rank(&self) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        let candidates = self
            .orchestrator
            .config
            .adapters
            .keys()
            .map(|adapter| arena_input_for(session, adapter))
            .collect::<Vec<_>>();
        let ranked = rank_arena_candidates(candidates);
        let mut lines =
            vec!["arena ranking requires explicit approval before promotion".to_string()];
        lines.extend(ranked.iter().map(|candidate| {
            format!(
                "#{rank} {adapter}: {score} ({reasons})",
                rank = candidate.rank,
                adapter = candidate.adapter,
                score = candidate.score,
                reasons = candidate.reasons.join("; ")
            )
        }));
        Ok(update(lines, inspector_for(session), Some(session.clone())))
    }
}

fn arena_input_for(session: &TuiSession, adapter: &str) -> ArenaCandidateInput {
    if adapter != session.adapter {
        return ArenaCandidateInput {
            adapter: adapter.to_string(),
            review_status: ArenaReviewStatus::Unknown,
            verification_passed: false,
            diff_size: 0,
            contract_drift: false,
            crashed: false,
        };
    }
    ArenaCandidateInput {
        adapter: adapter.to_string(),
        review_status: review_status(session),
        verification_passed: verification_passed(session),
        diff_size: changed_line_count(session),
        contract_drift: contract_drift(session),
        crashed: false,
    }
}

fn review_status(session: &TuiSession) -> ArenaReviewStatus {
    let Some(review) = session.gates.get("review_implementation_against_contract") else {
        return ArenaReviewStatus::Unknown;
    };
    let text = review.to_string().to_lowercase();
    if text.contains("fail") || text.contains("blocker") {
        ArenaReviewStatus::Fail
    } else if text.contains("warn") || text.contains("risk") {
        ArenaReviewStatus::Warn
    } else {
        ArenaReviewStatus::Pass
    }
}

fn verification_passed(session: &TuiSession) -> bool {
    !session.verification.is_empty()
        && session
            .verification
            .values()
            .all(|status| matches!(status.as_str(), "pass" | "passed" | "ok" | "green"))
}

fn changed_line_count(session: &TuiSession) -> u64 {
    session
        .changed_files
        .iter()
        .filter_map(|file| file.get("lines").and_then(Value::as_u64))
        .sum()
}

fn contract_drift(session: &TuiSession) -> bool {
    session
        .gates
        .get("review_implementation_against_contract")
        .map(|review| review.to_string().to_lowercase().contains("drift"))
        .unwrap_or(false)
}
