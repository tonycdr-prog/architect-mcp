use anyhow::Result;

use crate::approval::{promote_approved_changes, promotion_status_lines};
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};
use crate::session::SessionPhase;
use crate::verification::ensure_verification_passed;

impl InteractiveWorkflowEngine {
    pub(crate) fn approve(&mut self, reason: &str) -> Result<WorkflowUpdate> {
        let phase = self.active()?.phase.clone();
        let has_worktree = self.active()?.worktree.is_some();
        match (phase, has_worktree) {
            (SessionPhase::FilePlanReviewed, false) => {
                let session = self.update_active(|session| session.approve_execution(reason))?;
                Ok(update(
                    vec![format!("adapter execution approved: {reason}")],
                    inspector_for(session),
                    Some(session.clone()),
                ))
            }
            (SessionPhase::ReviewRequired, true) => {
                anyhow::bail!(
                    "run final review and session review after passed verification before promotion approval"
                )
            }
            (SessionPhase::Complete, true) => {
                ensure_verification_passed(self.active()?)?;
                if !self.active()?.adapter_run_issues.is_empty() {
                    anyhow::bail!(
                        "adapter run has blocking issues; rerun adapter successfully or use override <reason>"
                    );
                }
                let session = self.update_active(|session| session.approve(reason))?;
                Ok(update(
                    vec![format!("changes approved for promotion: {reason}")],
                    inspector_for(session),
                    Some(session.clone()),
                ))
            }
            _ => anyhow::bail!(
                "approval is available after file review for adapter execution, or after adapter review for promotion"
            ),
        }
    }

    pub(crate) fn reject(&mut self, reason: &str) -> Result<WorkflowUpdate> {
        let session = self.update_active(|session| session.reject(reason))?;
        Ok(update(
            vec![format!("changes rejected: {reason}")],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn override_approval(&mut self, reason: &str) -> Result<WorkflowUpdate> {
        let session = self.update_active(|session| session.override_approval(reason))?;
        Ok(update(
            vec![format!("promotion override recorded: {reason}")],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn promote(&mut self) -> Result<WorkflowUpdate> {
        let mut session = self.active()?.clone();
        let promoted = promote_approved_changes(&mut session, &self.orchestrator.workspace)?;
        self.store.save(&mut session)?;
        self.active = Some(session);
        let session = self.active()?;
        Ok(update(
            vec![format!("promoted {} changed file(s)", promoted.len())],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn promotion_status(&self) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        Ok(update(
            promotion_status_lines(session, &self.orchestrator.workspace),
            inspector_for(session),
            Some(session.clone()),
        ))
    }
}
