use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::arena::{rank_arena_candidates, rank_arena_records};
use crate::headless::HeadlessRunOptions;
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_arena_evidence::{
    arena_input_for, candidate_line, candidate_record_from_events, file_list,
};
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};
use crate::session::SessionPhase;

impl InteractiveWorkflowEngine {
    pub(crate) async fn arena_run(&mut self, adapters: Vec<String>) -> Result<WorkflowUpdate> {
        if adapters.len() < 2 {
            anyhow::bail!("arena run requires at least two adapters");
        }
        if !self.orchestrator.config.agents.worktree_isolation {
            anyhow::bail!("arena run requires isolated worktree execution");
        }
        let session = self.active()?.clone();
        if !session.gates.contains_key("review_proposed_file_plan") {
            anyhow::bail!("review files before arena run");
        }
        if session.phase != SessionPhase::FilePlanReviewed {
            anyhow::bail!("arena run is only available after review files");
        }
        if !session.execution_approved {
            anyhow::bail!("approve arena execution before arena run");
        }
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
            session.clear_adapter_run_evidence();
            session.arena_candidates = records;
            session.phase = SessionPhase::ReviewRequired;
            session.clear_execution_approval();
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

    pub(crate) fn arena_select(&mut self, adapter: &str) -> Result<WorkflowUpdate> {
        let adapter = adapter.trim();
        if adapter.is_empty() {
            anyhow::bail!("arena select requires an adapter name");
        }
        let candidate = self
            .active()?
            .arena_candidates
            .iter()
            .find(|candidate| candidate.adapter == adapter)
            .cloned()
            .with_context(|| format!("arena candidate '{adapter}' was not recorded"))?;
        if candidate.crashed || !candidate.adapter_run_issues.is_empty() {
            let issues = if candidate.adapter_run_issues.is_empty() {
                "adapter crashed or timed out".to_string()
            } else {
                candidate.adapter_run_issues.join("; ")
            };
            anyhow::bail!("arena candidate '{adapter}' has blocking issues: {issues}");
        }
        let worktree = candidate
            .worktree
            .clone()
            .context("arena candidate worktree evidence missing")?;
        if candidate.changed_files.is_empty() {
            anyhow::bail!("arena candidate '{adapter}' has no changed-file evidence");
        }
        let selected_files = file_list(&candidate);
        let session = self.update_active(|session| {
            session.clear_adapter_run_evidence();
            session.adapter = candidate.adapter.clone();
            session.worktree = Some(PathBuf::from(worktree));
            session.diff_stat = candidate.diff_stat.clone();
            session.changed_files = candidate.changed_files.clone();
            session.adapter_crashed = candidate.crashed;
            session.adapter_run_issues = candidate.adapter_run_issues.clone();
            for gate in [
                "review_implementation_against_contract",
                "review_repo_structure",
            ] {
                if let Some(result) = candidate.review_gates.get(gate) {
                    session.set_gate(gate, result.clone());
                }
            }
            session.phase = SessionPhase::ReviewRequired;
            session.clear_execution_approval();
        })?;
        Ok(update(
            vec![
                format!("arena candidate selected: {adapter}"),
                format!("selected files: {selected_files}"),
                "record verification, run final/session review, then approve promote before promotion".to_string(),
            ],
            inspector_for(session),
            Some(session.clone()),
        ))
    }
}
