use anyhow::Result;

use crate::foundry::{
    build_repo_foundry_plan, foundry_plan_lines, foundry_status_lines,
    render_foundry_create_preview,
};
use crate::foundry_execution::{ensure_staged_repo_exists, execute_repo_foundry_plan};
use crate::foundry_stage::stage_repo_foundry_plan;
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};

impl InteractiveWorkflowEngine {
    pub(crate) fn foundry_plan(
        &mut self,
        repo_name: &str,
        owner: Option<&str>,
    ) -> Result<WorkflowUpdate> {
        self.require_file_plan_for_foundry()?;
        let plan = build_repo_foundry_plan(self.active()?, repo_name, owner)?;
        let transcript = foundry_plan_lines(&plan);
        let session = self.update_active(|session| {
            session.foundry_plan = Some(plan);
            session.foundry_stage = None;
            session.foundry_execution = None;
            session.foundry_approved = false;
            session.foundry_approval_reason = None;
        })?;
        Ok(update(
            transcript,
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn foundry_status(&self) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        Ok(update(
            foundry_status_lines(session),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn foundry_approve(&mut self, reason: &str) -> Result<WorkflowUpdate> {
        self.require_file_plan_for_foundry()?;
        if reason.trim().is_empty() {
            anyhow::bail!("foundry approve requires a reason");
        }
        if self.active()?.foundry_plan.is_none() {
            anyhow::bail!("run foundry plan before foundry approve");
        }
        let session = self.update_active(|session| {
            session.foundry_approved = true;
            session.foundry_approval_reason = Some(reason.trim().to_string());
        })?;
        Ok(update(
            vec![format!("foundry approved: {}", reason.trim())],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn foundry_stage(&mut self) -> Result<WorkflowUpdate> {
        self.require_file_plan_for_foundry()?;
        if self.active()?.foundry_plan.is_none() {
            anyhow::bail!("run foundry plan before foundry stage");
        }
        if !self.active()?.foundry_approved {
            anyhow::bail!("approve local repo staging before foundry stage");
        }
        let stage = stage_repo_foundry_plan(&self.orchestrator.workspace, self.active()?)?;
        let session = self.update_active(|session| {
            session.foundry_stage = Some(stage.clone());
            session.foundry_execution = None;
            session.foundry_approved = false;
            session.foundry_approval_reason = None;
        })?;
        Ok(update(
            vec![
                format!("foundry staged repo: {}", stage.path.display()),
                format!("staged artifacts: {}", stage.artifacts_written),
                "next: foundry approve <reason>, then foundry create --execute".to_string(),
            ],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn foundry_create(&mut self, execute: bool) -> Result<WorkflowUpdate> {
        self.require_file_plan_for_foundry()?;
        let session = self.active()?;
        let Some(plan) = &session.foundry_plan else {
            anyhow::bail!("run foundry plan before foundry create");
        };
        if !session.foundry_approved {
            anyhow::bail!("approve repo creation before foundry create");
        }
        if execute {
            let Some(stage) = &session.foundry_stage else {
                anyhow::bail!("run foundry stage before foundry create --execute");
            };
            ensure_staged_repo_exists(stage)?;
            let execution = execute_repo_foundry_plan(plan, stage)?;
            let status = execution.status.clone();
            let command_count = execution.commands.len();
            let session = self.update_active(|session| {
                session.foundry_execution = Some(execution);
                session.foundry_approved = false;
                session.foundry_approval_reason = None;
            })?;
            return Ok(update(
                vec![
                    format!("foundry github execution: {status}"),
                    format!("commands run: {command_count}"),
                    "foundry approval consumed".to_string(),
                ],
                inspector_for(session),
                Some(session.clone()),
            ));
        }
        Ok(update(
            render_foundry_create_preview(plan),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    fn require_file_plan_for_foundry(&self) -> Result<()> {
        if !self
            .active()?
            .gates
            .contains_key("review_proposed_file_plan")
        {
            anyhow::bail!("review files before foundry plan");
        }
        Ok(())
    }
}
