pub use crate::interactive_commands::{WorkflowCommand, parse_workflow_command};
pub use crate::interactive_update::WorkflowUpdate;
use crate::interactive_update::{help_update, inspector_for, update};
use crate::orchestrator::Orchestrator;
use crate::session::{SessionPhase, SessionStore, TuiSession};
use anyhow::Result;
use serde_json::json;
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
        let session = self.update_active(|session| {
            session
                .verification
                .insert(check.to_string(), status.to_string());
        })?;
        Ok(update(
            vec![format!("verification recorded: {check}={status}")],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    async fn final_review(&mut self, response: &str) -> Result<WorkflowUpdate> {
        let checks = self
            .active()?
            .verification
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let result = self
            .call_tool(
                "review_agent_final_response",
                "check final answer evidence and gaps",
                json!({ "request": { "response": response, "requiredChecks": checks } }),
            )
            .await?;
        let session = self.update_active(|session| {
            session.set_gate("review_agent_final_response", result);
        })?;
        Ok(update(
            vec!["final response reviewed".to_string()],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    async fn session_review(&mut self) -> Result<WorkflowUpdate> {
        let result = self
            .call_tool(
                "review_agent_session",
                "review whole session for missed gates",
                json!({ "request": self.active()?.gates.clone() }),
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
}
