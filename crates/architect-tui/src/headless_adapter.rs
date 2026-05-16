use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::adapter::{AgentEvent, probe_adapter_health, run_adapter_pty};
use crate::adapter_review::{collect_diff_evidence, review_adapter_work};
use crate::headless::GateReviewState;
use crate::headless::HeadlessRunOptions;
use crate::headless_events::{JsonlEvent, emit, emit_complete, emit_skip};
use crate::mcp::StdioMcpClient;
use crate::orchestrator::Orchestrator;

pub(crate) async fn run_ready_adapter<W: AsyncWriteExt + Unpin>(
    orchestrator: &Orchestrator,
    options: &HeadlessRunOptions,
    output: &mut W,
    session_id: &str,
    prompt: &str,
    client: &mut StdioMcpClient,
    gate_state: &GateReviewState,
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
    let mut adapter = adapter.clone();
    let Some(workspace) =
        prepare_adapter_workspace(orchestrator, options, session_id, output).await?
    else {
        emit_complete(output, options.jsonl, session_id, "approval_required").await?;
        return Ok(false);
    };
    adapter.working_directory = Some(workspace.display().to_string());
    let events = run_adapter_pty(
        &adapter,
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
    let evidence = collect_diff_evidence(&workspace)?;
    let worktree_display = workspace.display().to_string();
    emit(
        output,
        options.jsonl,
        &JsonlEvent::DiffEvidence {
            worktree: &worktree_display,
            changed_files: &evidence.changed_files,
            diff_stat: &evidence.diff_stat,
        },
    )
    .await?;
    review_adapter_work(client, output, options, gate_state, &evidence).await?;
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

async fn prepare_adapter_workspace<W: AsyncWriteExt + Unpin>(
    orchestrator: &Orchestrator,
    options: &HeadlessRunOptions,
    session_id: &str,
    output: &mut W,
) -> Result<Option<std::path::PathBuf>> {
    if !orchestrator.config.agents.worktree_isolation {
        if orchestrator
            .config
            .agents
            .shared_workspace_requires_confirmation
        {
            emit_skip(
                output,
                options.jsonl,
                "shared-workspace execution requires explicit interactive confirmation",
            )
            .await?;
            return Ok(None);
        }
        return Ok(Some(orchestrator.workspace.clone()));
    }
    ensure_git_repo(&orchestrator.workspace)?;
    let worktree = orchestrator.worktree_path_for_agent(session_id, &options.adapter);
    if let Some(parent) = worktree.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(&orchestrator.workspace)
        .args(["worktree", "add", "--detach"])
        .arg(&worktree)
        .arg("HEAD")
        .output()
        .context("failed to create isolated adapter worktree")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "git worktree add failed with status {}: {}",
            output.status,
            stderr.trim()
        );
    }
    Ok(Some(worktree))
}

fn ensure_git_repo(workspace: &std::path::Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("failed to check git workspace")?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "true" {
        anyhow::bail!("adapter execution requires a git workspace for isolated worktrees");
    }
    Ok(())
}
