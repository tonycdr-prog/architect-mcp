use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::Value;
use uuid::Uuid;

use crate::config::TuiConfig;
use crate::foundry::repo_target;
pub use crate::foundry_smoke_report::{
    FoundryGithubVerification, FoundrySmokeCommandReport, FoundrySmokeReport, FoundrySmokeStatus,
};
use crate::foundry_smoke_report::{failed_report, print_text_report};
use crate::foundry_smoke_script::foundry_smoke_commands;
use crate::interactive::InteractiveWorkflowEngine;
use crate::mcp::ArchitectMcpBridge;
use crate::orchestrator::Orchestrator;
use crate::promotion_smoke_workspace::init_git_workspace;

#[derive(Debug, Clone)]
pub struct FoundrySmokeOptions {
    pub json: bool,
    pub owner: String,
    pub repo: Option<String>,
    pub execute: bool,
    pub confirm_private_repo_mutation: bool,
    pub keep_workspace: bool,
}

pub async fn run_foundry_smoke(
    source_workspace: PathBuf,
    config: TuiConfig,
    options: FoundrySmokeOptions,
) -> Result<()> {
    let report = build_foundry_smoke_report(source_workspace, config, &options).await;
    match &report {
        Ok(report) if options.json => println!("{}", serde_json::to_string_pretty(report)?),
        Ok(report) => print_text_report(report),
        Err(error) if options.json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&failed_report(&options, error))?
            );
        }
        Err(error) => print_text_report(&failed_report(&options, error)),
    }

    let report = report?;
    if report.status == FoundrySmokeStatus::Failed {
        anyhow::bail!("foundry smoke failed");
    }
    Ok(())
}

pub async fn build_foundry_smoke_report(
    source_workspace: PathBuf,
    config: TuiConfig,
    options: &FoundrySmokeOptions,
) -> Result<FoundrySmokeReport> {
    let repo = options.repo.clone().unwrap_or_else(default_repo_name);
    let target = repo_target(Some(&options.owner), &repo);
    if options.execute && !options.confirm_private_repo_mutation {
        anyhow::bail!("live foundry smoke requires --confirm-private-repo-mutation");
    }
    if options.execute {
        ensure_repo_absent(&target)?;
    }

    let workspace = create_foundry_workspace()?;
    init_git_workspace(&workspace)?;
    let smoke_config = config_for_foundry_smoke(&source_workspace, config)?;
    let mut engine =
        InteractiveWorkflowEngine::new(Orchestrator::new(workspace.clone(), smoke_config));
    let mut commands = Vec::new();
    let mut error = None;

    for command in foundry_smoke_commands(&repo, &options.owner, options.execute) {
        if let Err(err) = apply_command(&mut engine, &mut commands, &command).await {
            error = Some(err.to_string());
            break;
        }
    }

    let active = engine.active().ok();
    let staged_repo = active
        .and_then(|session| session.foundry_stage.as_ref())
        .map(|stage| stage.path.clone());
    let staged_artifacts = active
        .and_then(|session| session.foundry_stage.as_ref())
        .map(|stage| stage.artifacts_written)
        .unwrap_or_default();
    let execution_status = active
        .and_then(|session| session.foundry_execution.as_ref())
        .map(|execution| execution.status.clone());
    let github = if error.is_none() && options.execute {
        match verify_github_target(&target) {
            Ok(verification) => Some(verification),
            Err(err) => {
                error = Some(err.to_string());
                None
            }
        }
    } else {
        None
    };
    let status = foundry_status(
        options.execute,
        staged_repo.as_deref(),
        execution_status.as_deref(),
        github.as_ref(),
        &mut error,
    );
    let report = FoundrySmokeReport {
        schema_version: 1,
        status,
        mode: if options.execute { "live" } else { "dry_run" }.to_string(),
        target,
        workspace: workspace.clone(),
        staged_repo,
        staged_artifacts,
        execution_status,
        github,
        commands,
        error,
        retention: if options.execute {
            "private GitHub repo retained for maintainer evidence; delete manually when no longer needed"
                .to_string()
        } else {
            "no GitHub repo created".to_string()
        },
    };

    if !options.keep_workspace && report.status == FoundrySmokeStatus::Passed && !options.execute {
        let _ = fs::remove_dir_all(&workspace);
    }

    Ok(report)
}

fn config_for_foundry_smoke(source_workspace: &Path, mut config: TuiConfig) -> Result<TuiConfig> {
    let spec = ArchitectMcpBridge::new(source_workspace, config.clone()).process_spec();
    config.architect_mcp.command = Some(spec.command);
    config.architect_mcp.args = spec.args;
    config.architect_mcp.tool_surface = spec.tool_surface;
    config.validate()?;
    Ok(config)
}

async fn apply_command(
    engine: &mut InteractiveWorkflowEngine,
    reports: &mut Vec<FoundrySmokeCommandReport>,
    command: &str,
) -> Result<()> {
    match engine.apply_input(command).await {
        Ok(update) => {
            reports.push(FoundrySmokeCommandReport {
                command: command.to_string(),
                ok: true,
                transcript: update.transcript,
                error: None,
            });
            Ok(())
        }
        Err(err) => {
            let message = err.to_string();
            reports.push(FoundrySmokeCommandReport {
                command: command.to_string(),
                ok: false,
                transcript: Vec::new(),
                error: Some(message.clone()),
            });
            Err(anyhow::anyhow!(message))
        }
    }
}

pub(crate) fn foundry_status(
    execute: bool,
    staged_repo: Option<&Path>,
    execution_status: Option<&str>,
    github: Option<&FoundryGithubVerification>,
    error: &mut Option<String>,
) -> FoundrySmokeStatus {
    if staged_repo.is_some()
        && !execute
        && execution_status.is_none()
        && github.is_none()
        && error.is_none()
    {
        return FoundrySmokeStatus::Passed;
    }
    if staged_repo.is_some()
        && execute
        && execution_status == Some("passed")
        && github.is_some_and(foundry_github_verification_passed)
        && error.is_none()
    {
        return FoundrySmokeStatus::Passed;
    }
    if error.is_none() {
        *error =
            Some("foundry smoke did not produce required staging/execution evidence".to_string());
    }
    FoundrySmokeStatus::Failed
}

fn verify_github_target(target: &str) -> Result<FoundryGithubVerification> {
    let repo = run_json_command(
        Command::new("gh")
            .args(["repo", "view", target, "--json"])
            .arg("nameWithOwner,visibility,isPrivate,url"),
    )?;
    let prs = run_json_command(
        Command::new("gh")
            .args(["pr", "list", "-R", target, "--head", "architect/bootstrap"])
            .args(["--json", "number,url,isDraft,state", "--limit", "1"]),
    )?;
    let pr = prs.as_array().and_then(|items| items.first());
    Ok(FoundryGithubVerification {
        repo_url: string_field(&repo, "url")?,
        repo_visibility: string_field(&repo, "visibility")?,
        is_private: repo
            .get("isPrivate")
            .and_then(Value::as_bool)
            .unwrap_or_default(),
        draft_pr_url: pr
            .and_then(|value| value.get("url"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        draft_pr_number: pr
            .and_then(|value| value.get("number"))
            .and_then(Value::as_u64),
        draft_pr_is_draft: pr
            .and_then(|value| value.get("isDraft"))
            .and_then(Value::as_bool),
        draft_pr_state: pr
            .and_then(|value| value.get("state"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn foundry_github_verification_passed(verification: &FoundryGithubVerification) -> bool {
    verification.is_private
        && verification.draft_pr_url.is_some()
        && verification.draft_pr_number.is_some()
        && verification.draft_pr_is_draft == Some(true)
        && verification.draft_pr_state.as_deref() == Some("OPEN")
}

fn ensure_repo_absent(target: &str) -> Result<()> {
    let output = Command::new("gh")
        .args(["repo", "view", target])
        .output()
        .with_context(|| format!("failed to check whether {target} already exists"))?;
    if output.status.success() {
        anyhow::bail!("refusing to overwrite existing GitHub repository {target}");
    }
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if stderr.contains("could not resolve")
        || stderr.contains("not found")
        || stderr.contains("not exist")
    {
        return Ok(());
    }
    anyhow::bail!("could not confirm {target} is absent: {}", stderr.trim())
}

fn run_json_command(command: &mut Command) -> Result<Value> {
    let output = command.output().context("failed to run gh command")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    serde_json::from_slice(&output.stdout).context("failed to parse gh JSON output")
}

fn string_field(value: &Value, field: &str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .with_context(|| format!("gh response omitted {field}"))
}

fn create_foundry_workspace() -> Result<PathBuf> {
    let workspace =
        std::env::temp_dir().join(format!("architect-mcp-tui-foundry-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)
        .with_context(|| format!("failed to create {}", workspace.display()))?;
    Ok(workspace)
}

fn default_repo_name() -> String {
    let id = Uuid::new_v4().simple().to_string();
    format!("architect-mcp-foundry-smoke-{}", &id[..8])
}
