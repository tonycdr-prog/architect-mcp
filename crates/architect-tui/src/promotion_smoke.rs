use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::adapter::probe_adapter_health;
use crate::config::TuiConfig;
use crate::interactive::InteractiveWorkflowEngine;
use crate::mcp::ArchitectMcpBridge;
use crate::orchestrator::Orchestrator;
pub use crate::promotion_smoke_report::{
    PromotionSmokeCommandReport, PromotionSmokeReport, PromotionSmokeStatus,
    PromotionSmokeVerificationReport,
};
use crate::promotion_smoke_report::{failed_report, print_text_report, session_changed_files};
use crate::promotion_smoke_script::{finish_commands, setup_commands};
use crate::promotion_smoke_verification::run_required_verification;
use crate::promotion_smoke_workspace::{create_smoke_workspace, init_git_workspace};
use crate::session::{ApprovalStatus, SessionPhase};

#[derive(Debug, Clone)]
pub struct PromotionSmokeOptions {
    pub json: bool,
    pub adapter: String,
    pub keep_workspace: bool,
    pub timeout_seconds: u64,
}

pub async fn run_promotion_smoke(
    source_workspace: PathBuf,
    config: TuiConfig,
    options: PromotionSmokeOptions,
) -> Result<()> {
    let report = build_promotion_smoke_report(source_workspace, config, &options).await;
    match &report {
        Ok(report) if options.json => println!("{}", serde_json::to_string_pretty(report)?),
        Ok(report) => print_text_report(report),
        Err(error) if options.json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&failed_report(&options.adapter, error))?
            );
        }
        Err(error) => print_text_report(&failed_report(&options.adapter, error)),
    }

    let report = report?;
    if report.status == PromotionSmokeStatus::Failed {
        anyhow::bail!("promotion smoke failed");
    }
    Ok(())
}

pub async fn build_promotion_smoke_report(
    source_workspace: PathBuf,
    config: TuiConfig,
    options: &PromotionSmokeOptions,
) -> Result<PromotionSmokeReport> {
    let workspace = create_smoke_workspace()?;
    init_git_workspace(&workspace)?;
    let smoke_config = config_for_promotion_smoke(&source_workspace, config, options)?;
    ensure_adapter_ready(&smoke_config, &options.adapter)?;

    let mut engine =
        InteractiveWorkflowEngine::new(Orchestrator::new(workspace.clone(), smoke_config));
    let mut commands = Vec::new();
    let mut verification = Vec::new();
    let mut error = None;

    for command in setup_commands(&options.adapter) {
        if let Err(err) = apply_reported_command(&mut engine, &mut commands, &command).await {
            error = Some(err.to_string());
            break;
        }
    }

    if error.is_none() {
        match run_required_verification(&mut engine, &mut commands).await {
            Ok(records) => verification = records,
            Err(err) => error = Some(err.to_string()),
        }
    }

    if error.is_none() {
        for command in finish_commands(&engine, &verification)? {
            if let Err(err) = apply_reported_command(&mut engine, &mut commands, &command).await {
                error = Some(err.to_string());
                break;
            }
        }
    }

    let report = build_report(
        workspace.clone(),
        options,
        commands,
        verification,
        engine.active().ok(),
        error,
    );

    if !options.keep_workspace && report.status == PromotionSmokeStatus::Passed {
        let _ = fs::remove_dir_all(&workspace);
    }

    Ok(report)
}

pub(crate) async fn apply_reported_command(
    engine: &mut InteractiveWorkflowEngine,
    reports: &mut Vec<PromotionSmokeCommandReport>,
    command: &str,
) -> Result<()> {
    match engine.apply_input(command).await {
        Ok(update) => {
            reports.push(PromotionSmokeCommandReport {
                command: command.to_string(),
                ok: true,
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
            });
            Ok(())
        }
        Err(err) => {
            let message = err.to_string();
            reports.push(PromotionSmokeCommandReport {
                command: command.to_string(),
                ok: false,
                phase: None,
                approval: None,
                changed_files: Vec::new(),
                error: Some(message.clone()),
            });
            Err(anyhow::anyhow!(message))
        }
    }
}

fn config_for_promotion_smoke(
    source_workspace: &Path,
    mut config: TuiConfig,
    options: &PromotionSmokeOptions,
) -> Result<TuiConfig> {
    let spec = ArchitectMcpBridge::new(source_workspace, config.clone()).process_spec();
    config.architect_mcp.command = Some(spec.command);
    config.architect_mcp.args = spec.args;
    config.architect_mcp.tool_surface = spec.tool_surface;
    config.agents.default_adapter = options.adapter.clone();
    config.agents.default_timeout_seconds = options.timeout_seconds;
    config.validate()?;
    Ok(config)
}

fn ensure_adapter_ready(config: &TuiConfig, adapter_name: &str) -> Result<()> {
    let health = config
        .adapters
        .get(adapter_name)
        .map(|adapter| probe_adapter_health(adapter_name, adapter))
        .with_context(|| format!("adapter '{adapter_name}' is not configured"))?;
    if !health.ready {
        anyhow::bail!("adapter '{adapter_name}' is not ready: {}", health.detail);
    }
    Ok(())
}

fn build_report(
    workspace: PathBuf,
    options: &PromotionSmokeOptions,
    commands: Vec<PromotionSmokeCommandReport>,
    verification: Vec<PromotionSmokeVerificationReport>,
    active: Option<&crate::session::TuiSession>,
    mut error: Option<String>,
) -> PromotionSmokeReport {
    let changed_files = active.map(session_changed_files).unwrap_or_default();
    let final_phase = active.map(|session| session.phase.clone());
    let final_approval = active.map(|session| session.approval_status.clone());
    let promoted_files = if final_approval == Some(ApprovalStatus::Promoted) {
        changed_files
    } else {
        Vec::new()
    };
    let status = promotion_status(&promoted_files, &final_phase, &final_approval, &mut error);
    PromotionSmokeReport {
        schema_version: 1,
        status,
        adapter: options.adapter.clone(),
        workspace,
        commands,
        verification,
        promoted_files,
        final_phase,
        final_approval,
        error,
    }
}

fn promotion_status(
    promoted_files: &[String],
    final_phase: &Option<SessionPhase>,
    final_approval: &Option<ApprovalStatus>,
    error: &mut Option<String>,
) -> PromotionSmokeStatus {
    if error.is_none()
        && *final_phase == Some(SessionPhase::Complete)
        && *final_approval == Some(ApprovalStatus::Promoted)
        && promoted_files.len() == 1
        && promoted_files[0] == "docs/codex-adapter-smoke.md"
    {
        return PromotionSmokeStatus::Passed;
    }
    if error.is_none() {
        *error = Some("promotion smoke must promote only docs/codex-adapter-smoke.md".to_string());
    }
    PromotionSmokeStatus::Failed
}
