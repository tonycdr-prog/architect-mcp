use anyhow::Result;
use serde_json::{Value, json};

use crate::headless::HeadlessRunOptions;
use crate::headless_support::{
    likely_files_from_gate, pre_edit_args, proposed_file_plan, verification_checks,
};
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_support::apply_run_evidence;
use crate::interactive_update::{WorkflowUpdate, gate_line, inspector_for, update};
use crate::session::SessionPhase;

impl InteractiveWorkflowEngine {
    pub(crate) fn resume(&mut self, id: &str) -> Result<WorkflowUpdate> {
        let session = self.store.load(id)?;
        self.active = Some(session.clone());
        Ok(update(
            vec![
                format!("session resumed: {}", session.id),
                format!("phase: {:?}", session.phase),
            ],
            inspector_for(&session),
            Some(session),
        ))
    }

    pub(crate) fn answer(&mut self, key: &str, value: &str) -> Result<WorkflowUpdate> {
        let session = self.update_active(|session| session.set_answer(key, value))?;
        Ok(update(
            vec![format!("answer recorded: {key}")],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn grill(&mut self) -> Result<WorkflowUpdate> {
        let brief = self.active()?.brief.clone();
        let result = self
            .call_tool(
                "grill_me",
                "clarify task before edits",
                json!({ "brief": brief, "includeContract": true, "includeArtifacts": true }),
            )
            .await?;
        let ready = result
            .get("ready")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let session = self.update_active(|session| {
            session.set_gate("grill_me", result);
            session.phase = if ready {
                SessionPhase::IntakeReady
            } else {
                SessionPhase::IntakeBlocked
            };
        })?;
        let mut lines = vec![gate_line("grill_me", ready)];
        lines.extend(grill_feedback(session.gates.get("grill_me")));
        Ok(update(lines, inspector_for(session), Some(session.clone())))
    }

    pub(crate) async fn create_contract(&mut self) -> Result<WorkflowUpdate> {
        self.require_intake_ready()?;
        let (idea, grill, verification) = self.gate_inputs()?;
        let build_plan = grill.get("buildPlan").cloned().unwrap_or_else(|| json!({}));
        let mut args = pre_edit_args(&idea, &grill, &verification);
        let likely = likely_files_from_gate(&grill, &build_plan);
        if !likely.is_empty() {
            args["likelyFiles"] = json!(likely);
        }
        let result = self
            .call_tool(
                "create_pre_edit_contract",
                "freeze goals, constraints, risks, and evidence",
                args,
            )
            .await?;
        let session = self.update_active(|session| {
            session.set_required_verification(verification);
            session.set_gate("create_pre_edit_contract", result);
            session.phase = SessionPhase::ContractReady;
        })?;
        Ok(update(
            vec!["contract ready".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn review_plan(&mut self) -> Result<WorkflowUpdate> {
        self.require_gate(
            "create_pre_edit_contract",
            "create contract before review plan",
        )?;
        let grill = self.grill_gate()?;
        let plan = grill.get("buildPlan").cloned().unwrap_or_else(|| json!({}));
        let verification = verification_checks(&grill, &plan);
        let result = self
            .call_tool(
                "review_build_plan",
                "constrain workflow slices before agent execution",
                json!({ "plan": plan, "allowedChecks": verification }),
            )
            .await?;
        let session = self.update_active(|session| {
            session.set_required_verification(verification);
            session.set_gate("review_build_plan", result);
            session.phase = SessionPhase::PlanReviewed;
        })?;
        Ok(update(
            vec!["plan reviewed".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn review_files(&mut self) -> Result<WorkflowUpdate> {
        self.require_gate("review_build_plan", "review plan before review files")?;
        let grill = self.grill_gate()?;
        let result = self
            .call_tool(
                "review_proposed_file_plan",
                "approve touched paths and artifact hygiene",
                json!({ "plan": proposed_file_plan(&grill) }),
            )
            .await?;
        let session = self.update_active(|session| {
            session.set_gate("review_proposed_file_plan", result);
            session.phase = SessionPhase::FilePlanReviewed;
            session.clear_execution_approval();
            session.clear_arena_candidates();
        })?;
        Ok(update(
            vec!["file plan reviewed".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn run_adapter(&mut self) -> Result<WorkflowUpdate> {
        self.require_gate(
            "review_proposed_file_plan",
            "review files before run adapter",
        )?;
        let session = self.active()?.clone();
        if session.phase != SessionPhase::FilePlanReviewed {
            anyhow::bail!("run adapter is only available after review files");
        }
        if !session.execution_approved {
            anyhow::bail!("approve adapter execution before run adapter");
        }
        let pre_edit = session
            .gates
            .get("create_pre_edit_contract")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("create contract before run adapter"))?;
        let (_, _, verification) = self.gate_inputs()?;
        let gate_state = crate::headless::GateReviewState {
            pre_edit,
            verification: verification.clone(),
        };
        let mut output = Vec::new();
        let executed = self
            .orchestrator
            .run_adapter_against_gate_state(
                HeadlessRunOptions {
                    prompt: session.prompt.clone(),
                    adapter: session.adapter.clone(),
                    jsonl: true,
                    concurrency: 1,
                    execute: true,
                },
                &mut output,
                &session.id,
                &session.prompt,
                &gate_state,
            )
            .await?;
        let events = String::from_utf8_lossy(&output).to_string();
        let session = self.update_active(|session| {
            session.clear_adapter_run_evidence();
            session.set_required_verification(verification);
            apply_run_evidence(session, &events);
            session.phase = if executed {
                SessionPhase::ReviewRequired
            } else {
                SessionPhase::FilePlanReviewed
            };
            if executed {
                session.clear_execution_approval();
            }
        })?;
        let summary = if executed {
            "adapter run completed; review evidence recorded"
        } else {
            "adapter did not run; approval or adapter readiness required"
        };
        Ok(update(
            vec![summary.to_string(), events],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    fn require_intake_ready(&self) -> Result<()> {
        let grill = self.grill_gate()?;
        let ready = grill.get("ready").and_then(Value::as_bool).unwrap_or(false);
        let blockers = grill
            .get("blockers")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .any(|item| item.as_str().is_some_and(|s| !s.is_empty()))
            })
            .unwrap_or(false);
        if !ready || blockers {
            anyhow::bail!(
                "grill_me must be ready before this command; answer blockers and rerun grill"
            );
        }
        Ok(())
    }

    fn require_gate(&self, name: &str, message: &str) -> Result<()> {
        if !self.active()?.gates.contains_key(name) {
            anyhow::bail!("{message}");
        }
        Ok(())
    }
}

fn grill_feedback(grill: Option<&Value>) -> Vec<String> {
    let Some(grill) = grill else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    if let Some(blockers) = grill.get("blockers").and_then(Value::as_array) {
        for blocker in blockers
            .iter()
            .filter_map(Value::as_str)
            .filter(|blocker| !blocker.is_empty())
            .take(5)
        {
            lines.push(format!("blocker: {blocker}"));
        }
    }
    if let Some(question) = grill
        .get("nextQuestion")
        .and_then(|value| value.get("question"))
        .and_then(Value::as_str)
    {
        lines.push(format!("next question: {question}"));
    }
    if let Some(answer) = grill
        .get("nextQuestion")
        .and_then(|value| value.get("recommendedAnswer"))
        .and_then(Value::as_str)
    {
        lines.push(format!("recommended answer: {answer}"));
    }
    lines
}
