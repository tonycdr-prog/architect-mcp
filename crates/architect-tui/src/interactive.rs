pub use crate::interactive_commands::{WorkflowCommand, parse_workflow_command};
pub use crate::interactive_update::WorkflowUpdate;
use crate::interactive_update::{help_update, inspector_for, update};
use crate::orchestrator::Orchestrator;
use crate::session::{SessionPhase, SessionStore, TuiSession};
use crate::verification::{
    ensure_known_verification_check, ensure_verification_passed, normalize_verification_status,
    required_checks, verification_records, verification_summary_lines,
};
use anyhow::Result;
use serde_json::{Map, Value, json};
pub struct InteractiveWorkflowEngine {
    pub(crate) orchestrator: Orchestrator,
    pub(crate) store: SessionStore,
    pub(crate) active: Option<TuiSession>,
}

impl InteractiveWorkflowEngine {
    pub fn new(orchestrator: Orchestrator) -> Self {
        let store = SessionStore::for_workspace(&orchestrator.workspace);
        Self {
            orchestrator,
            store,
            active: None,
        }
    }

    pub async fn apply_input(&mut self, input: &str) -> Result<WorkflowUpdate> {
        match parse_workflow_command(input) {
            WorkflowCommand::NewApp(idea) => self.new_app(idea),
            WorkflowCommand::Resume(id) => self.resume(&id),
            WorkflowCommand::Answer { key, value } => self.answer(&key, &value),
            WorkflowCommand::Grill => self.grill().await,
            WorkflowCommand::CreateContract => self.create_contract().await,
            WorkflowCommand::ReviewPlan => self.review_plan().await,
            WorkflowCommand::ReviewFiles => self.review_files().await,
            WorkflowCommand::RunAdapter => self.run_adapter().await,
            WorkflowCommand::Approve(reason) => self.approve(&reason),
            WorkflowCommand::Reject(reason) => self.reject(&reason),
            WorkflowCommand::Override(reason) => self.override_approval(&reason),
            WorkflowCommand::Promote => self.promote(),
            WorkflowCommand::DiffSummary => self.diff_summary(),
            WorkflowCommand::DiffFile(path) => self.diff_file(&path),
            WorkflowCommand::ArenaRun(adapters) => self.arena_run(adapters).await,
            WorkflowCommand::ArenaRank => self.arena_rank(),
            WorkflowCommand::VerificationStatus => self.verification_status(),
            WorkflowCommand::RecordVerification { check, status } => {
                self.record_verification(&check, &status)
            }
            WorkflowCommand::FinalReview(response) => self.final_review(&response).await,
            WorkflowCommand::SessionReview => self.session_review().await,
            WorkflowCommand::Cancel => self.cancel(),
            WorkflowCommand::Help => Ok(help_update()),
        }
    }

    fn new_app(&mut self, idea: String) -> Result<WorkflowUpdate> {
        let adapter = self.orchestrator.config.agents.default_adapter.clone();
        let mut session = TuiSession::new(idea.clone(), adapter);
        self.store.save(&mut session)?;
        self.active = Some(session.clone());
        Ok(update(
            vec![
                format!("new app: {idea}"),
                format!("session: {}", session.id),
            ],
            inspector_for(&session),
            Some(session),
        ))
    }
}
impl InteractiveWorkflowEngine {
    fn record_verification(&mut self, check: &str, status: &str) -> Result<WorkflowUpdate> {
        if check.trim().is_empty() {
            anyhow::bail!("verification check is required");
        }
        let normalized = normalize_verification_status(status)?;
        let session = self.active()?;
        let check = check.trim();
        ensure_known_verification_check(session, check)?;
        if session.worktree.is_none()
            || !matches!(
                session.phase,
                SessionPhase::ReviewRequired | SessionPhase::Complete
            )
        {
            anyhow::bail!("run adapter before recording verification evidence");
        }
        let session = self.update_active(|session| {
            session
                .verification
                .insert(check.to_string(), normalized.clone());
        })?;
        Ok(update(
            vec![format!("verification recorded: {check}={normalized}")],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    fn verification_status(&self) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        Ok(update(
            verification_summary_lines(session),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    async fn final_review(&mut self, response: &str) -> Result<WorkflowUpdate> {
        self.require_adapter_review_evidence()?;
        ensure_verification_passed(self.active()?)?;
        let checks = required_checks(self.active()?);
        let result = self
            .call_tool(
                "review_agent_final_response",
                "check final answer evidence and gaps",
                json!({ "request": { "response": response, "requiredChecks": checks } }),
            )
            .await?;
        let session = self.update_active(|session| {
            session.set_final_response(response.to_string());
            session.set_gate("review_agent_final_response", result);
        })?;
        Ok(update(
            vec!["final response reviewed".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    async fn session_review(&mut self) -> Result<WorkflowUpdate> {
        self.require_adapter_review_evidence()?;
        self.require_final_review_after_verification()?;
        let request = self.session_review_request()?;
        let result = self
            .call_tool(
                "review_agent_session",
                "review whole session for missed gates",
                json!({ "request": request }),
            )
            .await?;
        let session = self.update_active(|session| {
            session.set_gate("review_agent_session", result);
            session.phase = SessionPhase::Complete;
        })?;
        Ok(update(
            vec!["session reviewed".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    fn cancel(&mut self) -> Result<WorkflowUpdate> {
        let session = self.update_active(|session| {
            session.phase = SessionPhase::Cancelled;
        })?;
        Ok(update(
            vec!["session cancelled".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    fn require_adapter_review_evidence(&self) -> Result<()> {
        let session = self.active()?;
        if session.worktree.is_none()
            || !matches!(
                session.phase,
                SessionPhase::ReviewRequired | SessionPhase::Complete
            )
        {
            anyhow::bail!("run adapter before final/session review");
        }
        Ok(())
    }

    fn require_final_review_after_verification(&self) -> Result<()> {
        let session = self.active()?;
        ensure_verification_passed(session)?;
        if session
            .final_response
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
            || !session.gates.contains_key("review_agent_final_response")
        {
            anyhow::bail!("run final review after passed verification before session review");
        }
        Ok(())
    }

    fn session_review_request(&self) -> Result<Value> {
        let session = self.active()?;
        let mut request = Map::new();
        request.insert("request".to_string(), Value::String(session.prompt.clone()));
        request.insert(
            "changedFiles".to_string(),
            Value::Array(session.changed_files.clone()),
        );
        request.insert(
            "verification".to_string(),
            Value::Array(verification_records(session)),
        );
        if let Some(final_response) = &session.final_response {
            request.insert(
                "finalResponse".to_string(),
                Value::String(final_response.clone()),
            );
        }
        if let Some(pre_edit) = session.gates.get("create_pre_edit_contract") {
            if let Some(intent) = pre_edit.get("intent").cloned() {
                request.insert("intent".to_string(), intent);
            }
            if let Some(contract) = pre_edit.get("contract").cloned() {
                request.insert("contract".to_string(), contract);
            }
        }
        Ok(Value::Object(request))
    }
}
