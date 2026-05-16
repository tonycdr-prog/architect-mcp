use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::adapter::AgentEvent;
use crate::mcp::McpResponseError;
use crate::orchestrator::WorkflowStep;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum JsonlEvent<'a> {
    SessionStarted {
        session_id: &'a str,
        workspace: &'a str,
        adapter: &'a str,
        concurrency: usize,
    },
    McpCall {
        name: &'a str,
        purpose: &'a str,
    },
    McpResult {
        name: &'a str,
        result: &'a Value,
    },
    McpError {
        name: &'a str,
        error: &'a McpResponseError,
    },
    ApprovalRequired {
        reason: &'a str,
        blockers: &'a [String],
        challenges: &'a Value,
        next_question: &'a Value,
    },
    AdapterSkipped {
        reason: &'a str,
    },
    AgentEvent {
        event: &'a AgentEvent,
    },
    DiffEvidence {
        worktree: &'a str,
        changed_files: &'a [Value],
        diff_stat: &'a str,
    },
    WorkflowStep {
        name: &'a str,
        gate: &'a str,
        approval_required: bool,
    },
    Complete {
        session_id: &'a str,
        status: &'a str,
    },
}

pub(crate) async fn emit_workflow<W: AsyncWriteExt + Unpin>(
    output: &mut W,
    jsonl: bool,
    steps: &[WorkflowStep],
) -> Result<()> {
    for step in steps {
        emit(
            output,
            jsonl,
            &JsonlEvent::WorkflowStep {
                name: &step.name,
                gate: &step.gate,
                approval_required: step.approval_required,
            },
        )
        .await?;
    }
    Ok(())
}

pub(crate) async fn emit_skip<W: AsyncWriteExt + Unpin>(
    output: &mut W,
    jsonl: bool,
    reason: &str,
) -> Result<()> {
    emit(output, jsonl, &JsonlEvent::AdapterSkipped { reason }).await
}

pub(crate) async fn emit_complete<W: AsyncWriteExt + Unpin>(
    output: &mut W,
    jsonl: bool,
    session_id: &str,
    status: &str,
) -> Result<()> {
    emit(output, jsonl, &JsonlEvent::Complete { session_id, status }).await
}

pub(crate) async fn emit<W: AsyncWriteExt + Unpin, T: Serialize>(
    output: &mut W,
    jsonl: bool,
    event: &T,
) -> Result<()> {
    if jsonl {
        let line = serde_json::to_string(event)?;
        output.write_all(line.as_bytes()).await?;
        output.write_all(b"\n").await?;
    } else {
        let pretty = serde_json::to_string_pretty(event)?;
        output.write_all(pretty.as_bytes()).await?;
        output.write_all(b"\n").await?;
    }
    Ok(())
}
