use anyhow::{Context, Result};

use crate::approval::promote_approved_changes;
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};

impl InteractiveWorkflowEngine {
    pub(crate) fn approve(&mut self, reason: &str) -> Result<WorkflowUpdate> {
        let session = self.update_active(|session| session.approve(reason))?;
        Ok(update(
            vec![format!("changes approved: {reason}")],
            inspector_for(session),
            Some(session.clone()),
        ))
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
        let mut session = self.active.take().context("start with: new app <idea>")?;
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
}
