use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::adapter::{AgentEvent, probe_adapter_health, run_adapter_process, run_adapter_pty};
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
    let prompt = gated_adapter_prompt(prompt, gate_state);
    let run_options = crate::adapter::PtyRunOptions {
        adapter_name: options.adapter.clone(),
        prompt,
        timeout: Duration::from_secs(orchestrator.config.agents.default_timeout_seconds),
    };
    let events = if adapter.pty {
        run_adapter_pty(&adapter, run_options)
    } else {
        run_adapter_process(&adapter, run_options)
    }
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
    reset_existing_adapter_worktree(orchestrator, &worktree)?;
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

fn reset_existing_adapter_worktree(orchestrator: &Orchestrator, worktree: &Path) -> Result<()> {
    if fs::symlink_metadata(worktree).is_err() {
        return Ok(());
    }
    ensure_managed_worktree_path(orchestrator, worktree)?;
    let output = Command::new("git")
        .arg("-C")
        .arg(&orchestrator.workspace)
        .args(["worktree", "remove", "--force"])
        .arg(worktree)
        .output()
        .context("failed to remove existing isolated adapter worktree")?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_lower = stderr.to_ascii_lowercase();
    if stderr_lower.contains("not a working tree") {
        remove_managed_path(worktree)
            .with_context(|| format!("failed to remove stale {}", worktree.display()))?;
        return Ok(());
    }
    anyhow::bail!(
        "git worktree remove failed with status {}: {}",
        output.status,
        stderr.trim()
    );
}

fn ensure_managed_worktree_path(orchestrator: &Orchestrator, worktree: &Path) -> Result<()> {
    let managed_root = orchestrator
        .workspace
        .join(".architect-mcp")
        .join("worktrees");
    if !worktree.starts_with(&managed_root) {
        anyhow::bail!(
            "refusing to reset unmanaged adapter worktree path {}",
            worktree.display()
        );
    }
    Ok(())
}

fn remove_managed_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
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

fn gated_adapter_prompt(prompt: &str, gate_state: &GateReviewState) -> String {
    let contract = gate_state
        .pre_edit
        .get("contract")
        .unwrap_or(&gate_state.pre_edit);
    let contract_json =
        serde_json::to_string_pretty(contract).unwrap_or_else(|_| contract.to_string());
    let verification = verification_list(&gate_state.verification);
    format!(
        "architect-mcp approved this adapter execution after the local work gate.\n\n\
Original request:\n{prompt}\n\n\
Approved pre-edit contract JSON:\n```json\n{contract_json}\n```\n\n\
Required verification checks:\n{verification}\n\n\
Execution rules:\n\
- Work only inside the current isolated adapter worktree.\n\
- Do not commit, push, merge, promote, publish, or edit files outside this worktree.\n\
- Implement the smallest useful slice that satisfies the approved contract.\n\
- Leave changes uncommitted for architect-mcp review and human promotion.\n\
- In the final response, state changed files, verification run or not run, assumptions, and remaining work."
    )
}

fn verification_list(checks: &[String]) -> String {
    if checks.is_empty() {
        return "- No explicit checks supplied by the gate; report verification honestly."
            .to_string();
    }
    checks
        .iter()
        .map(|check| format!("- {check}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gated_adapter_prompt_includes_contract_and_execution_rules() {
        let gate_state = GateReviewState {
            pre_edit: json!({
                "contract": {
                    "name": "Local Notes",
                    "allowedFiles": ["src/features/notes/**"]
                }
            }),
            verification: vec!["npm test".to_string(), "npm run build".to_string()],
        };

        let prompt = gated_adapter_prompt("Build notes CRUD", &gate_state);

        assert!(prompt.contains("Original request:\nBuild notes CRUD"));
        assert!(prompt.contains("\"name\": \"Local Notes\""));
        assert!(prompt.contains("- npm test"));
        assert!(prompt.contains("- npm run build"));
        assert!(prompt.contains("Do not commit, push, merge, promote, publish"));
        assert!(prompt.contains("Leave changes uncommitted"));
    }
}
