use std::path::PathBuf;

use anyhow::Result;

use crate::cli_foundry::FoundryAuditCliOptions;
use crate::config::TuiConfig;
use crate::foundry_audit_mcp::run_mcp_foundry_audit;
use crate::foundry_audit_public_summary::build_public_summary;
use crate::foundry_audit_report::print_text_report;
pub use crate::foundry_audit_report::{FoundryAuditReport, FoundryAuditStatus};

#[derive(Debug, Clone)]
pub struct FoundryAuditOptions {
    pub json: bool,
    pub public_summary: bool,
    pub max_files: usize,
    pub mcp_workspace: Option<PathBuf>,
}

pub async fn run_foundry_audit(
    workspace: PathBuf,
    config: TuiConfig,
    options: FoundryAuditOptions,
) -> Result<()> {
    if options.json && options.public_summary {
        anyhow::bail!("--json and --public-summary are mutually exclusive");
    }
    let report = build_foundry_audit_report(workspace, config, &options).await;
    if options.public_summary {
        println!(
            "{}",
            serde_json::to_string_pretty(&build_public_summary(&report))?
        );
    } else if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }
    if report.status == FoundryAuditStatus::Failed {
        anyhow::bail!("foundry audit failed");
    }
    Ok(())
}

pub async fn run_foundry_audit_cli(
    workspace: PathBuf,
    config: TuiConfig,
    options: FoundryAuditCliOptions,
) -> Result<()> {
    let audit_workspace = options
        .repo_path
        .as_ref()
        .map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                workspace.join(path)
            }
        })
        .unwrap_or_else(|| workspace.clone());
    run_foundry_audit(
        audit_workspace,
        config,
        FoundryAuditOptions {
            json: options.json,
            public_summary: options.public_summary,
            max_files: options.max_files,
            mcp_workspace: options.repo_path.map(|_| workspace),
        },
    )
    .await
}

pub async fn build_foundry_audit_report(
    workspace: PathBuf,
    config: TuiConfig,
    options: &FoundryAuditOptions,
) -> FoundryAuditReport {
    let mcp_workspace = options.mcp_workspace.as_deref().unwrap_or(&workspace);
    run_mcp_foundry_audit(&workspace, mcp_workspace, config, options.max_files)
}

pub fn foundry_audit_ledger_lines(report: &FoundryAuditReport) -> Vec<String> {
    let mut lines = vec![
        format!("foundry audit: {:?}", report.status),
        format!("read-only: {}", report.read_only),
        format!(
            "mutation boundary: {}; approval required before mutation: {}",
            report.mutation_boundary, report.approval_required_before_mutation
        ),
        format!(
            "ledger decisions: {} evidence={} server_writes={}",
            report.ledger.total_entries,
            report.evidence.total_evidence,
            report.server_writes_performed
        ),
        format!(
            "forge previews: total={} pr={} architect_issue={} exception={} no_op={} human={}",
            report.forge.previews_generated,
            report.forge.pull_request_previews,
            report.forge.architect_issue_previews,
            report.forge.exception_records,
            report.forge.no_op_records,
            report.forge.human_questions
        ),
    ];
    for decision in &report.decisions {
        lines.push(format!(
            "- {} score={} evidence={} risk={} approval={} preview={} next={}",
            decision.route,
            decision.score,
            decision.evidence_count,
            decision.risk,
            decision.approval_state,
            decision.preview_kind.as_deref().unwrap_or("none"),
            decision.next_action
        ));
    }
    if report.decisions.is_empty() {
        lines.push("ledger: no decisions emitted by the MCP chain".to_string());
    }
    if let Some(error) = &report.error {
        lines.push(format!("error: {error}"));
    }
    lines
}
