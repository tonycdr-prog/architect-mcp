use anyhow::{Context, Result};
use serde_json::Value;

use crate::arena::{
    ArenaCandidateInput, ArenaCandidateRecord, ArenaReviewStatus, rank_arena_candidates,
    rank_arena_records,
};
use crate::headless::HeadlessRunOptions;
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};
use crate::session::{SessionPhase, TuiSession};

impl InteractiveWorkflowEngine {
    pub(crate) async fn arena_run(&mut self, adapters: Vec<String>) -> Result<WorkflowUpdate> {
        if adapters.is_empty() {
            anyhow::bail!("arena run requires at least one adapter");
        }
        let session = self.active()?.clone();
        let pre_edit = session
            .gates
            .get("create_pre_edit_contract")
            .cloned()
            .context("create contract before arena run")?;
        let (_, _, verification) = self.gate_inputs()?;
        let gate_state = crate::headless::GateReviewState {
            pre_edit,
            verification,
        };
        let mut records = Vec::new();
        let mut lines = vec![format!(
            "arena run: {} candidate(s), isolated worktrees, no auto-promotion",
            adapters.len()
        )];
        for adapter in adapters {
            let mut output = Vec::new();
            let result = self
                .orchestrator
                .run_adapter_against_gate_state(
                    HeadlessRunOptions {
                        prompt: session.prompt.clone(),
                        adapter: adapter.clone(),
                        jsonl: true,
                        concurrency: records.len() + 1,
                        execute: true,
                    },
                    &mut output,
                    &session.id,
                    &session.prompt,
                    &gate_state,
                )
                .await;
            let events = String::from_utf8_lossy(&output).to_string();
            let mut record = candidate_record_from_events(&adapter, &events);
            if let Err(error) = result {
                record.crashed = true;
                record.summary.push(format!("adapter error: {error}"));
            }
            lines.push(candidate_line(&record));
            records.push(record);
        }
        let session = self.update_active(|session| {
            session.arena_candidates = records;
            session.phase = SessionPhase::ReviewRequired;
        })?;
        lines.push("run arena rank to compare candidates before approval".to_string());
        Ok(update(lines, inspector_for(session), Some(session.clone())))
    }

    pub(crate) fn arena_rank(&self) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        let ranked = if session.arena_candidates.is_empty() {
            let candidates = self
                .orchestrator
                .config
                .adapters
                .keys()
                .map(|adapter| arena_input_for(session, adapter))
                .collect::<Vec<_>>();
            rank_arena_candidates(candidates)
        } else {
            rank_arena_records(&session.arena_candidates)
        };
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
        if !session.arena_candidates.is_empty() {
            lines.push("candidate evidence:".to_string());
            for candidate in &session.arena_candidates {
                lines.push(format!(
                    "- {}: files={} worktree={} crashed={}",
                    candidate.adapter,
                    file_list(candidate),
                    candidate.worktree.as_deref().unwrap_or("not recorded"),
                    candidate.crashed
                ));
            }
        }
        Ok(update(lines, inspector_for(session), Some(session.clone())))
    }
}

fn candidate_record_from_events(adapter: &str, events: &str) -> ArenaCandidateRecord {
    let mut temp = TuiSession::new("arena candidate", adapter);
    crate::interactive_support::apply_run_evidence(&mut temp, events);
    let mut summary = Vec::new();
    for line in events.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("approval_required") => {
                if let Some(reason) = event.get("reason").and_then(Value::as_str) {
                    summary.push(format!("approval required: {reason}"));
                }
            }
            Some("agent_event") => {
                if let Some(kind) = event
                    .get("event")
                    .and_then(|event| event.get("type"))
                    .and_then(Value::as_str)
                {
                    summary.push(format!("agent event: {kind}"));
                }
            }
            _ => {}
        }
    }
    ArenaCandidateRecord {
        adapter: adapter.to_string(),
        worktree: temp
            .worktree
            .as_ref()
            .map(|path| path.display().to_string()),
        review_status: review_status(&temp),
        verification_passed: verification_passed(&temp),
        diff_size: changed_line_count(&temp),
        contract_drift: contract_drift(&temp),
        crashed: temp.adapter_crashed,
        changed_files: temp.changed_files,
        summary,
    }
}

fn candidate_line(candidate: &ArenaCandidateRecord) -> String {
    format!(
        "candidate {}: review={:?} files={} worktree={}",
        candidate.adapter,
        candidate.review_status,
        file_list(candidate),
        candidate.worktree.as_deref().unwrap_or("not recorded")
    )
}

fn file_list(candidate: &ArenaCandidateRecord) -> String {
    let files = candidate
        .changed_files
        .iter()
        .filter_map(|file| file.get("path").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if files.is_empty() {
        "none".to_string()
    } else {
        files.join(",")
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
        crashed: session.adapter_crashed,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn candidate_record_extracts_diff_and_crash_evidence() {
        let events = [
            json!({
                "type": "diff_evidence",
                "worktree": ".architect-mcp/worktrees/session/shell",
                "changed_files": [{ "path": "docs/live-qa.md", "lines": 8 }],
                "diff_stat": "docs/live-qa.md | 8 ++++++++"
            })
            .to_string(),
            json!({
                "type": "agent_event",
                "event": { "type": "crashed", "message": "boom" }
            })
            .to_string(),
        ]
        .join("\n");
        let candidate = candidate_record_from_events("shell", &events);
        assert_eq!(candidate.adapter, "shell");
        assert_eq!(candidate.diff_size, 8);
        assert!(candidate.crashed);
        assert_eq!(candidate.changed_files[0]["path"], "docs/live-qa.md");
    }
}
