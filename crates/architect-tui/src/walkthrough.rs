use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;
use uuid::Uuid;

use crate::config::TuiConfig;
use crate::interactive::InteractiveWorkflowEngine;
use crate::orchestrator::Orchestrator;
use crate::session::{ApprovalStatus, SessionPhase};
use crate::walkthrough_script::{config_for_walkthrough, walkthrough_commands};

#[derive(Debug, Clone)]
pub struct WalkthroughOptions {
    pub json: bool,
    pub keep_workspace: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WalkthroughStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalkthroughReport {
    pub schema_version: u8,
    pub status: WalkthroughStatus,
    pub workspace: PathBuf,
    pub commands: Vec<WalkthroughCommandReport>,
    pub promoted_files: Vec<String>,
    pub final_phase: Option<SessionPhase>,
    pub final_approval: Option<ApprovalStatus>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalkthroughCommandReport {
    pub command: String,
    pub ok: bool,
    pub transcript: Vec<String>,
    pub phase: Option<SessionPhase>,
    pub approval: Option<ApprovalStatus>,
    pub changed_files: Vec<String>,
    pub error: Option<String>,
}

pub async fn run_walkthrough(
    source_workspace: PathBuf,
    config: TuiConfig,
    options: WalkthroughOptions,
) -> Result<()> {
    let report = build_walkthrough_report(source_workspace, config, options.keep_workspace).await;
    match &report {
        Ok(report) if options.json => println!("{}", serde_json::to_string_pretty(report)?),
        Ok(report) => print_text_report(report),
        Err(error) => {
            let failed = WalkthroughReport {
                schema_version: 1,
                status: WalkthroughStatus::Failed,
                workspace: PathBuf::new(),
                commands: Vec::new(),
                promoted_files: Vec::new(),
                final_phase: None,
                final_approval: None,
                error: Some(error.to_string()),
            };
            if options.json {
                println!("{}", serde_json::to_string_pretty(&failed)?);
            } else {
                print_text_report(&failed);
            }
        }
    }

    let report = report?;
    if report.status == WalkthroughStatus::Failed {
        anyhow::bail!("interactive walkthrough failed");
    }
    Ok(())
}

async fn build_walkthrough_report(
    source_workspace: PathBuf,
    config: TuiConfig,
    keep_workspace: bool,
) -> Result<WalkthroughReport> {
    let workspace = create_walkthrough_workspace()?;
    init_git_workspace(&workspace)?;
    let walkthrough_config = config_for_walkthrough(&source_workspace, config);
    walkthrough_config.validate()?;
    let mut engine =
        InteractiveWorkflowEngine::new(Orchestrator::new(workspace.clone(), walkthrough_config));
    let mut commands = Vec::new();
    let mut error = None;

    for command in walkthrough_commands() {
        let result = engine.apply_input(command).await;
        let report = match result {
            Ok(update) => WalkthroughCommandReport {
                command: command.to_string(),
                ok: true,
                transcript: update.transcript,
                phase: update.session.as_ref().map(|session| session.phase.clone()),
                approval: update
                    .session
                    .as_ref()
                    .map(|session| session.approval_status.clone()),
                changed_files: update
                    .session
                    .as_ref()
                    .map(session_changed_files)
                    .unwrap_or_default(),
                error: None,
            },
            Err(err) => {
                let message = err.to_string();
                error = Some(message.clone());
                WalkthroughCommandReport {
                    command: command.to_string(),
                    ok: false,
                    transcript: Vec::new(),
                    phase: None,
                    approval: None,
                    changed_files: Vec::new(),
                    error: Some(message),
                }
            }
        };
        commands.push(report);
        if error.is_some() {
            break;
        }
    }

    let active = engine.active().ok();
    let changed_files = active.map(session_changed_files).unwrap_or_default();
    let final_phase = active.map(|session| session.phase.clone());
    let final_approval = active.map(|session| session.approval_status.clone());
    let promoted_files = if final_approval == Some(ApprovalStatus::Promoted) {
        changed_files
    } else {
        Vec::new()
    };
    let status = if error.is_none()
        && final_phase == Some(SessionPhase::Complete)
        && final_approval == Some(ApprovalStatus::Promoted)
        && !promoted_files.is_empty()
    {
        WalkthroughStatus::Passed
    } else {
        if error.is_none() {
            error = Some("walkthrough ended without promoted changed-file evidence".to_string());
        }
        WalkthroughStatus::Failed
    };

    let report = WalkthroughReport {
        schema_version: 1,
        status,
        workspace: workspace.clone(),
        commands,
        promoted_files,
        final_phase,
        final_approval,
        error,
    };

    if !keep_workspace && report.status == WalkthroughStatus::Passed {
        let _ = fs::remove_dir_all(&workspace);
    }

    Ok(report)
}

fn create_walkthrough_workspace() -> Result<PathBuf> {
    let workspace =
        std::env::temp_dir().join(format!("architect-mcp-tui-walkthrough-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)
        .with_context(|| format!("failed to create {}", workspace.display()))?;
    Ok(workspace)
}

fn init_git_workspace(workspace: &Path) -> Result<()> {
    run_git(workspace, ["init"])?;
    run_git(
        workspace,
        ["config", "user.email", "walkthrough@example.com"],
    )?;
    run_git(
        workspace,
        ["config", "user.name", "architect-mcp walkthrough"],
    )?;
    fs::write(
        workspace.join("README.md"),
        "architect-mcp TUI walkthrough\n",
    )?;
    run_git(workspace, ["add", "README.md"])?;
    run_git(workspace, ["commit", "-m", "init"])?;
    Ok(())
}

fn run_git<const N: usize>(workspace: &Path, args: [&str; N]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn session_changed_files(session: &crate::session::TuiSession) -> Vec<String> {
    session
        .changed_files
        .iter()
        .filter_map(|file| file.get("path").and_then(|value| value.as_str()))
        .map(ToString::to_string)
        .collect()
}

fn print_text_report(report: &WalkthroughReport) {
    println!("architect-mcp-tui walkthrough: {:?}", report.status);
    if let Some(error) = &report.error {
        println!("error: {error}");
    }
    println!("workspace: {}", report.workspace.display());
    for command in &report.commands {
        println!(
            "- {}: {}",
            command.command,
            if command.ok { "ok" } else { "failed" }
        );
        if let Some(error) = &command.error {
            println!("  error: {error}");
        }
    }
    if !report.promoted_files.is_empty() {
        println!("promoted files: {}", report.promoted_files.join(", "));
    }
}
