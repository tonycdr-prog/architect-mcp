use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::config::TuiConfig;
use crate::evidence_index_markdown::render_markdown;
use crate::evidence_index_report::{
    EvidenceIndexReport, build_evidence_index_report_from_public_summaries_at,
};
use crate::governance_audit::{GovernanceAuditOptions, build_governance_audit_report};
use crate::governance_audit_public_summary::build_public_summary as build_governance_public_summary;
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness::{LaunchReadinessOptions, build_launch_readiness_report};
use crate::launch_readiness_public_summary::build_public_summary as build_launch_readiness_public_summary;

#[derive(Debug, Clone)]
pub struct EvidenceIndexOptions {
    pub json: bool,
    pub markdown: bool,
    pub markdown_output: Option<PathBuf>,
    pub require_go: bool,
    pub repo: Option<String>,
    pub stack_from_pr: Option<u64>,
    pub prs: Vec<u64>,
    pub blockers: Vec<u64>,
    pub waived_blockers: Vec<String>,
    pub terminal_evidence_issue: Option<u64>,
    pub terminal_evidence_waivers: Vec<String>,
    pub skip_mcp: bool,
    pub max_files: usize,
}

pub async fn run_evidence_index(
    workspace: PathBuf,
    config: TuiConfig,
    options: EvidenceIndexOptions,
) -> Result<()> {
    validate_output_mode(options.json, options.markdown)?;
    let report = build_evidence_index_report(workspace.clone(), config, &options).await;
    let markdown = if options.markdown || options.markdown_output.is_some() {
        Some(render_markdown(&report))
    } else {
        None
    };
    if let Some(markdown_output) = &options.markdown_output {
        write_markdown_output(
            &workspace,
            markdown_output,
            markdown.as_deref().unwrap_or(""),
        )?;
    }
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if options.markdown {
        println!("{}", markdown.as_deref().unwrap_or(""));
    } else {
        print_text_report(&report);
    }
    enforce_evidence_index_result(&report.result, options.require_go)?;
    Ok(())
}

pub(crate) fn enforce_evidence_index_result(
    result: &LaunchJudgeResult,
    require_go: bool,
) -> Result<()> {
    match result {
        LaunchJudgeResult::Go => Ok(()),
        LaunchJudgeResult::ConditionalGo if require_go => {
            anyhow::bail!("evidence index result is conditional-go; --require-go requires go")
        }
        LaunchJudgeResult::ConditionalGo => Ok(()),
        LaunchJudgeResult::NoGo => anyhow::bail!("evidence index result is no-go"),
    }
}

pub(crate) fn write_markdown_output(
    workspace: &Path,
    target: &Path,
    markdown: &str,
) -> Result<PathBuf> {
    let output = workspace_safe_output_path(workspace, target)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
        ensure_parent_stays_in_workspace(workspace, parent)?;
    }
    fs::write(&output, markdown)?;
    Ok(output)
}

pub(crate) fn validate_output_mode(json: bool, markdown: bool) -> Result<()> {
    if json && markdown {
        anyhow::bail!("choose only one evidence-index output mode: --json or --markdown");
    }
    Ok(())
}

pub async fn build_evidence_index_report(
    workspace: PathBuf,
    config: TuiConfig,
    options: &EvidenceIndexOptions,
) -> EvidenceIndexReport {
    let launch_readiness = build_launch_readiness_report(
        &workspace,
        &LaunchReadinessOptions {
            json: true,
            public_summary: true,
            repo: options.repo.clone(),
            stack_from_pr: options.stack_from_pr,
            prs: options.prs.clone(),
            blockers: options.blockers.clone(),
            waived_blockers: options.waived_blockers.clone(),
            terminal_evidence_issue: options.terminal_evidence_issue,
            terminal_evidence_waivers: options.terminal_evidence_waivers.clone(),
        },
    );
    let governance_audit = build_governance_audit_report(
        workspace,
        config,
        &GovernanceAuditOptions {
            json: true,
            public_summary: true,
            skip_mcp: options.skip_mcp,
            max_files: options.max_files,
        },
    )
    .await;

    build_evidence_index_report_from_public_summaries_at(
        build_launch_readiness_public_summary(&launch_readiness),
        build_governance_public_summary(&governance_audit),
        generated_at_unix_seconds(),
    )
}

fn generated_at_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn print_text_report(report: &EvidenceIndexReport) {
    println!("Evidence index: {:?}", report.result);
    println!("Read-only: {}", report.read_only);
    for section in &report.sections {
        println!(
            "- {}: {:?} ({})",
            section.name, section.result, section.summary
        );
    }
    if !report.next_actions.is_empty() {
        println!("Next actions:");
        for action in &report.next_actions {
            println!("- {action}");
        }
    }
}

fn workspace_safe_output_path(workspace: &Path, target: &Path) -> Result<PathBuf> {
    if target.as_os_str().is_empty() {
        anyhow::bail!("markdown output path must not be empty");
    }
    if target.is_absolute() {
        anyhow::bail!("markdown output path must be relative to the workspace");
    }

    let mut safe = PathBuf::new();
    for component in target.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("markdown output path must stay inside the workspace");
            }
        }
    }
    if safe.as_os_str().is_empty() {
        anyhow::bail!("markdown output path must name a file");
    }
    Ok(workspace.join(safe))
}

fn ensure_parent_stays_in_workspace(workspace: &Path, parent: &Path) -> Result<()> {
    let workspace = workspace.canonicalize()?;
    let parent = parent.canonicalize()?;
    if !parent.starts_with(&workspace) {
        anyhow::bail!("markdown output parent must stay inside the workspace");
    }
    Ok(())
}
