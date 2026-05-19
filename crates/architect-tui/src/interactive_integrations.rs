use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_integrations_support::{
    apply_args, apply_lines, ensure_install_review_passed, ensure_recommended, plan_lines,
    recommendation_lines, recommendation_request, review_lines,
};
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};

impl InteractiveWorkflowEngine {
    pub(crate) async fn integrations_recommend(&mut self, context: &str) -> Result<WorkflowUpdate> {
        let args = json!({ "request": recommendation_request(self.active()?, context) });
        let result = self
            .call_tool(
                "recommend_mcp_servers",
                "recommend MCP servers only after clarified provider and boundary needs",
                args,
            )
            .await?;
        let session = self.update_active(|session| session.set_mcp_recommendation(result))?;
        Ok(update(
            recommendation_lines(session.mcp_recommendation.as_ref()),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn integrations_plan(
        &mut self,
        server_id: &str,
        target_client: Option<&str>,
    ) -> Result<WorkflowUpdate> {
        let server_id = server_id.trim();
        if server_id.is_empty() {
            anyhow::bail!("integrations plan requires a server id");
        }
        ensure_recommended(self.active()?, server_id)?;
        let request = json!({
            "serverId": server_id,
            "targetClient": target_client.unwrap_or("generic-json"),
            "hostedMode": false
        });
        let result = self
            .call_tool(
                "create_mcp_install_plan",
                "create dry-run MCP install plan after recommendation",
                json!({ "request": request }),
            )
            .await?;
        let session = self.update_active(|session| session.set_mcp_install_plan(result))?;
        Ok(update(
            plan_lines(session.mcp_install_plan.as_ref()),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn integrations_review(&mut self) -> Result<WorkflowUpdate> {
        let plan = self
            .current_mcp_install_plan()
            .await
            .context("create an MCP install plan before review")?;
        let result = self
            .call_tool(
                "review_mcp_install_plan",
                "review MCP install plan before any apply step",
                json!({ "request": { "plan": plan, "writeFiles": false, "explicitApproval": false } }),
            )
            .await?;
        let session = self.update_active(|session| session.set_mcp_install_review(result))?;
        Ok(update(
            review_lines(session.mcp_install_review.as_ref()),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn integrations_apply_dry_run(
        &mut self,
        target_path: Option<&str>,
    ) -> Result<WorkflowUpdate> {
        ensure_install_review_passed(self.active()?)?;
        let plan = self.current_mcp_install_plan().await?;
        let result = self
            .call_tool(
                "apply_mcp_install_plan",
                "dry-run MCP config merge without writing files",
                apply_args(plan, target_path, false, false),
            )
            .await?;
        let session = self.update_active(|session| session.set_mcp_install_apply_result(result))?;
        Ok(update(
            apply_lines(session.gates.get("apply_mcp_install_plan"), false),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) fn integrations_approve(&mut self, reason: &str) -> Result<WorkflowUpdate> {
        ensure_install_review_passed(self.active()?)?;
        let reason = reason.trim();
        if reason.is_empty() {
            anyhow::bail!("integrations approve requires a reason");
        }
        let session = self.update_active(|session| session.approve_mcp_install(reason))?;
        Ok(update(
            vec![format!("MCP install approved: {reason}")],
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    pub(crate) async fn integrations_write(
        &mut self,
        target_path: Option<&str>,
    ) -> Result<WorkflowUpdate> {
        ensure_install_review_passed(self.active()?)?;
        if !self.active()?.mcp_install_approved {
            anyhow::bail!("approve MCP install before writing config");
        }
        let plan = self.current_mcp_install_plan().await?;
        let result = self
            .call_tool(
                "apply_mcp_install_plan",
                "write reviewed MCP config after explicit TUI approval",
                apply_args(plan, target_path, true, true),
            )
            .await?;
        let status = result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        if status != "written" {
            anyhow::bail!("MCP install write did not complete: {status}");
        }
        let session = self.update_active(|session| {
            session.set_mcp_install_apply_result(result);
            session.clear_mcp_install_approval();
        })?;
        Ok(update(
            apply_lines(session.gates.get("apply_mcp_install_plan"), true),
            inspector_for(session),
            Some(session.clone()),
        ))
    }

    async fn current_mcp_install_plan(&mut self) -> Result<Value> {
        let plan = self
            .active()?
            .mcp_install_plan
            .clone()
            .context("create an MCP install plan first")?;
        let server_id = plan
            .get("serverId")
            .and_then(Value::as_str)
            .context("stored MCP install plan is missing serverId")?;
        let target_client = plan
            .get("targetClient")
            .and_then(Value::as_str)
            .unwrap_or("generic-json");
        let hosted_mode = plan
            .get("hostedMode")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let hydrated = self
            .call_tool(
                "create_mcp_install_plan",
                "recreate MCP install plan from stored metadata before review/apply/write",
                json!({
                    "request": {
                        "serverId": server_id,
                        "targetClient": target_client,
                        "hostedMode": hosted_mode,
                    }
                }),
            )
            .await?;
        let hydrated_package = hydrated.get("packagePin").and_then(Value::as_str);
        let stored_package = plan.get("packagePin").and_then(Value::as_str);
        if let (Some(stored), Some(hydrated)) = (stored_package, hydrated_package)
            && stored != hydrated
        {
            anyhow::bail!("stored MCP install plan is stale; rerun integrations plan");
        }
        Ok(hydrated)
    }
}
