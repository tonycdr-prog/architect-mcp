use std::path::PathBuf;
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
    let report = build_evidence_index_report(workspace, config, &options).await;
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if options.markdown {
        println!("{}", render_markdown(&report));
    } else {
        print_text_report(&report);
    }
    if report.result == LaunchJudgeResult::NoGo {
        anyhow::bail!("evidence index result is no-go");
    }
    Ok(())
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
