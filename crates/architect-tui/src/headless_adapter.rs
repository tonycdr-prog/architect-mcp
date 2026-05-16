use std::time::Duration;

use anyhow::Result;
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::adapter::{AgentEvent, probe_adapter_health, run_adapter_pty};
use crate::headless::HeadlessRunOptions;
use crate::headless_events::{JsonlEvent, emit, emit_complete, emit_skip};
use crate::orchestrator::Orchestrator;

pub(crate) async fn run_ready_adapter<W: AsyncWriteExt + Unpin>(
    orchestrator: &Orchestrator,
    options: &HeadlessRunOptions,
    output: &mut W,
    session_id: &str,
    prompt: &str,
) -> Result<bool> {
    let Some(adapter) = orchestrator.config.adapters.get(&options.adapter) else {
        emit_skip(output, options.jsonl, "selected adapter is not configured").await?;
        emit_complete(output, options.jsonl, session_id, "approval_required").await?;
        return Ok(false);
    };

    let health = probe_adapter_health(&options.adapter, adapter);
    if !health.ready {
        emit_adapter_not_ready(output, options, session_id, health.detail).await?;
        return Ok(false);
    }
    let events = run_adapter_pty(
        adapter,
        crate::adapter::PtyRunOptions {
            adapter_name: options.adapter.clone(),
            prompt: prompt.to_string(),
            timeout: Duration::from_secs(orchestrator.config.agents.default_timeout_seconds),
        },
    )
    .unwrap_or_else(|error| {
        vec![AgentEvent::Crashed {
            message: error.to_string(),
        }]
    });
    for event in &events {
        emit(output, options.jsonl, &JsonlEvent::AgentEvent { event }).await?;
    }
    Ok(true)
}

async fn emit_adapter_not_ready<W: AsyncWriteExt + Unpin>(
    output: &mut W,
    options: &HeadlessRunOptions,
    session_id: &str,
    detail: String,
) -> Result<()> {
    let blockers = vec![detail];
    let challenges = json!([]);
    let next_question = json!({
        "question": format!("Make adapter '{}' ready before execution.", options.adapter),
        "recommendedAnswer": "Install or authenticate the adapter, then rerun with --execute."
    });
    emit(
        output,
        options.jsonl,
        &JsonlEvent::ApprovalRequired {
            reason: "selected adapter is not ready",
            blockers: &blockers,
            challenges: &challenges,
            next_question: &next_question,
        },
    )
    .await?;
    emit_complete(output, options.jsonl, session_id, "approval_required").await
}
