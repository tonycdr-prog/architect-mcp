use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

use crate::foundry::{
    build_repo_foundry_plan, foundry_plan_lines, foundry_status_lines,
    render_foundry_create_preview,
};
use crate::foundry_audit::{
    FoundryAuditOptions, build_foundry_audit_report, foundry_audit_ledger_lines,
};
use crate::foundry_execution::{ensure_staged_repo_exists, execute_repo_foundry_plan};
use crate::foundry_stage::stage_repo_foundry_plan;
use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{
    WorkflowUpdate, inspector_for, inspector_for_foundry_audit, update,
};

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

    pub(crate) async fn foundry_audit(
        &mut self,
        target_path: Option<&str>,
    ) -> Result<WorkflowUpdate> {
        let workspace = target_workspace(&self.orchestrator.workspace, target_path)?;
        let report = build_foundry_audit_report(
            workspace,
            self.orchestrator.config.clone(),
            &FoundryAuditOptions {
                json: false,
                public_summary: false,
                max_files: 1000,
                mcp_workspace: Some(self.orchestrator.workspace.clone()),
            },
        )
        .await;
        let transcript = foundry_audit_ledger_lines(&report);
        self.foundry_audit = Some(report);
        Ok(update(
            transcript,
            inspector_for_foundry_audit(self.active.as_ref(), self.foundry_audit.as_ref()),
            self.active.clone(),
        ))
    }

    pub(crate) fn foundry_ledger(&self) -> Result<WorkflowUpdate> {
        let Some(report) = &self.foundry_audit else {
            anyhow::bail!("run foundry audit before foundry ledger");
        };
        Ok(update(
            foundry_audit_ledger_lines(report),
            inspector_for_foundry_audit(self.active.as_ref(), Some(report)),
            self.active.clone(),
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
            ensure_staged_repo_exists(&self.orchestrator.workspace, &session.id, plan, stage)?;
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
        let session = self.active()?;
        let review = session.gates.get("review_proposed_file_plan");
        let Some(review) = review else {
            anyhow::bail!("review files before foundry plan");
        };
        if review_has_blockers(review) {
            anyhow::bail!("file-plan review must pass before foundry plan");
        }
        Ok(())
    }
}

fn target_workspace(
    default_workspace: &std::path::Path,
    target_path: Option<&str>,
) -> Result<PathBuf> {
    let Some(target_path) = target_path.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(default_workspace.to_path_buf());
    };
    let path = PathBuf::from(target_path);
    Ok(if path.is_absolute() {
        path
    } else {
        default_workspace.join(path)
    })
}

fn review_has_blockers(review: &Value) -> bool {
    status_blocks(review.get("status"))
        || status_blocks(review.pointer("/report/gate/status"))
        || review_errors(review.pointer("/summary/errors"))
        || review_errors(review.pointer("/report/summary/errors"))
        || review
            .get("violations")
            .and_then(Value::as_array)
            .is_some_and(|violations| violations.iter().any(violation_blocks))
}

fn status_blocks(status: Option<&Value>) -> bool {
    matches!(
        status.and_then(Value::as_str),
        Some("fail" | "failed" | "blocked" | "blocker")
    )
}

fn review_errors(errors: Option<&Value>) -> bool {
    errors
        .and_then(Value::as_u64)
        .is_some_and(|errors| errors > 0)
}

fn violation_blocks(violation: &Value) -> bool {
    matches!(
        violation.get("severity").and_then(Value::as_str),
        Some("error")
    ) || status_blocks(violation.get("status"))
}
