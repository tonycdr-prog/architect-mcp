use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncWriteExt, stdout};
use uuid::Uuid;

use crate::headless_adapter::run_ready_adapter;
use crate::headless_events::{JsonlEvent, emit, emit_complete, emit_skip, emit_workflow};
use crate::headless_support::{
    likely_files, pre_edit_args, proposed_file_plan, string_array, verification_checks,
};
use crate::mcp::{McpToolOutcome, StdioMcpClient};
use crate::orchestrator::Orchestrator;

#[derive(Debug, Clone)]
pub struct HeadlessRunOptions {
    pub prompt: String,
    pub adapter: String,
    pub jsonl: bool,
    pub concurrency: usize,
    pub execute: bool,
}

impl Orchestrator {
    pub async fn run_headless(&self, options: HeadlessRunOptions) -> Result<()> {
        let mut output = stdout();
        self.run_headless_to_writer(options, &mut output).await
    }

    pub async fn run_headless_to_writer<W: AsyncWriteExt + Unpin>(
        &self,
        options: HeadlessRunOptions,
        output: &mut W,
    ) -> Result<()> {
        let session_id = Uuid::new_v4().to_string();
        let workspace_display = self.workspace.display().to_string();
        let workflow = self.new_app_workflow(&options.prompt);

        emit(
            output,
            options.jsonl,
            &JsonlEvent::SessionStarted {
                session_id: &session_id,
                workspace: &workspace_display,
                adapter: &options.adapter,
                concurrency: options.concurrency,
            },
        )
        .await?;
        emit_workflow(output, options.jsonl, &workflow.steps).await?;

        let mut client = StdioMcpClient::connect(&self.bridge)?;
        let _tools = client.list_tools()?;
        let Some(grill_value) = grill(&mut client, output, &options).await? else {
            emit_complete(output, options.jsonl, &session_id, "mcp_error").await?;
            return Ok(());
        };

        if intake_blocked(&grill_value, output, options.jsonl, &session_id).await? {
            return Ok(());
        }
        if gate_reviews(
            &mut client,
            output,
            options.jsonl,
            &workflow.idea,
            &grill_value,
        )
        .await?
        .is_none()
        {
            emit_complete(output, options.jsonl, &session_id, "mcp_error").await?;
            return Ok(());
        }
        if !options.execute {
            emit_skip(
                output,
                options.jsonl,
                "gate-only mode complete; pass --execute to run the selected adapter",
            )
            .await?;
            emit_complete(output, options.jsonl, &session_id, "approval_required").await?;
            return Ok(());
        }
        if run_ready_adapter(self, &options, output, &session_id, &workflow.idea).await? {
            emit_complete(output, options.jsonl, &session_id, "review_required").await?;
        }
        Ok(())
    }
}

async fn grill<W: AsyncWriteExt + Unpin>(
    client: &mut StdioMcpClient,
    output: &mut W,
    options: &HeadlessRunOptions,
) -> Result<Option<Value>> {
    call_gate(
        client,
        output,
        options.jsonl,
        "grill_me",
        "clarify task before edits",
        json!({
            "brief": { "idea": options.prompt.clone() },
            "includeContract": true,
            "includeArtifacts": true
        }),
    )
    .await
}

async fn gate_reviews<W: AsyncWriteExt + Unpin>(
    client: &mut StdioMcpClient,
    output: &mut W,
    jsonl: bool,
    idea: &str,
    grill_value: &Value,
) -> Result<Option<()>> {
    let build_plan = grill_value
        .get("buildPlan")
        .cloned()
        .unwrap_or_else(|| json!({ "archetype": "custom-app", "slices": [] }));
    let contract = grill_value.get("contract").cloned();
    let verification = verification_checks(grill_value, &build_plan);
    let mut args = pre_edit_args(idea, grill_value, &verification);
    let likely = likely_files(&build_plan);
    if !likely.is_empty() {
        args["likelyFiles"] = json!(likely);
    }
    if !verification.is_empty() {
        args["verificationChecks"] = json!(verification.clone());
    }

    if call_gate(
        client,
        output,
        jsonl,
        "create_pre_edit_contract",
        "freeze goals, constraints, risks, and evidence",
        args,
    )
    .await?
    .is_none()
    {
        return Ok(None);
    }
    let mut build_args = json!({ "plan": build_plan, "allowedChecks": verification });
    if let Some(contract) = contract {
        build_args["contract"] = contract;
    }
    if call_gate(
        client,
        output,
        jsonl,
        "review_build_plan",
        "constrain workflow slices before agent execution",
        build_args,
    )
    .await?
    .is_none()
    {
        return Ok(None);
    }
    if call_gate(
        client,
        output,
        jsonl,
        "review_proposed_file_plan",
        "approve touched paths and artifact hygiene",
        json!({ "plan": proposed_file_plan(grill_value) }),
    )
    .await?
    .is_none()
    {
        return Ok(None);
    }
    Ok(Some(()))
}

async fn intake_blocked<W: AsyncWriteExt + Unpin>(
    grill_value: &Value,
    output: &mut W,
    jsonl: bool,
    session_id: &str,
) -> Result<bool> {
    let blockers = string_array(grill_value, "blockers");
    let ready = grill_value
        .get("ready")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if ready && blockers.is_empty() {
        return Ok(false);
    }
    let challenges = grill_value
        .get("challenges")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let next_question = grill_value
        .get("nextQuestion")
        .cloned()
        .unwrap_or(Value::Null);
    emit(
        output,
        jsonl,
        &JsonlEvent::ApprovalRequired {
            reason: "grill_me returned intake blockers or an incomplete brief",
            blockers: &blockers,
            challenges: &challenges,
            next_question: &next_question,
        },
    )
    .await?;
    emit_complete(output, jsonl, session_id, "approval_required").await?;
    Ok(true)
}

async fn call_gate<W: AsyncWriteExt + Unpin>(
    client: &mut StdioMcpClient,
    output: &mut W,
    jsonl: bool,
    name: &str,
    purpose: &str,
    args: Value,
) -> Result<Option<Value>> {
    emit(output, jsonl, &JsonlEvent::McpCall { name, purpose }).await?;
    match client.call_tool(name, args)? {
        McpToolOutcome::Ok { value } => {
            emit(
                output,
                jsonl,
                &JsonlEvent::McpResult {
                    name,
                    result: &value,
                },
            )
            .await?;
            Ok(Some(value))
        }
        McpToolOutcome::Error { error } => {
            emit(
                output,
                jsonl,
                &JsonlEvent::McpError {
                    name,
                    error: &error,
                },
            )
            .await?;
            Ok(None)
        }
    }
}
