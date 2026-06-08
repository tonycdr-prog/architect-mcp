use std::path::PathBuf;

use anyhow::Result;

use crate::config::TuiConfig;
use crate::governance_audit_mcp::run_mcp_repo_review;
use crate::governance_audit_public_summary::build_public_summary;
use crate::governance_audit_report::{
    GovernanceAuditReport, GovernanceAuditStatus, GovernanceFinding, GovernanceMcpReview,
    print_text_report,
};
use crate::governance_audit_static::static_findings;
use crate::governance_audit_support::{
    categories, deterministic_gates, smoke_evidence, status_for_findings, warning,
};
use crate::governance_memory::memory_proposals;
use crate::governance_profile::{GovernanceRepoProfile, resolve_governance_profile};

#[derive(Debug, Clone)]
pub struct GovernanceAuditOptions {
    pub json: bool,
    pub public_summary: bool,
    pub skip_mcp: bool,
    pub max_files: usize,
    pub profile: Option<GovernanceRepoProfile>,
}

pub async fn run_governance_audit(
    workspace: PathBuf,
    config: TuiConfig,
    options: GovernanceAuditOptions,
) -> Result<()> {
    let report = build_governance_audit_report(workspace, config, &options).await;
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
    if report.status == GovernanceAuditStatus::Failed {
        anyhow::bail!("governance audit failed");
    }
    Ok(())
}

pub async fn build_governance_audit_report(
    workspace: PathBuf,
    config: TuiConfig,
    options: &GovernanceAuditOptions,
) -> GovernanceAuditReport {
    let repo_profile = resolve_governance_profile(&workspace, options.profile);
    let mut findings = static_findings(&workspace, repo_profile.name);
    let mcp_review = if options.skip_mcp {
        Some(GovernanceMcpReview {
            status: "skipped".to_string(),
            gate_status: None,
            files_reviewed: None,
            scan_truncated: None,
            errors: None,
            warnings: None,
            violation_count: None,
            detail: "MCP review skipped by flag; deterministic file checks still ran".to_string(),
        })
    } else {
        Some(run_mcp_repo_review(&workspace, config, options.max_files))
    };

    if let Some(review) = &mcp_review {
        push_mcp_review_findings(&mut findings, review);
    }

    let deterministic_gates = deterministic_gates(&workspace, repo_profile.name);
    let smoke_evidence = smoke_evidence(&workspace);
    let memory_proposals = memory_proposals(&workspace);
    let categories = categories(&findings);
    let status = status_for_findings(&findings);

    GovernanceAuditReport {
        schema_version: 1,
        status,
        workspace: workspace.display().to_string(),
        read_only: true,
        repo_profile,
        categories,
        deterministic_gates,
        smoke_evidence,
        memory_proposals,
        mcp_review,
        findings,
    }
}

pub(crate) fn push_mcp_review_findings(
    findings: &mut Vec<GovernanceFinding>,
    review: &GovernanceMcpReview,
) {
    if review.status == "error" {
        findings.push(warning(
            "mcp-review",
            "GOV_MCP_REVIEW_UNAVAILABLE",
            "MCP repo review did not produce drift evidence.",
            &review.detail,
            "Run npm run build and retry governance-audit without --skip-mcp.",
        ));
    } else if matches!(review.gate_status.as_deref(), Some("fail" | "warn")) {
        findings.push(warning(
            "mcp-review",
            "GOV_MCP_REVIEW_NOT_CLEAN",
            "MCP repo review reported a non-clean repo-structure gate.",
            &review.detail,
            "Inspect the MCP review warnings or violations before treating the repo as healthy.",
        ));
    }
    if review.scan_truncated == Some(true) {
        findings.push(warning(
            "mcp-review",
            "GOV_MCP_REVIEW_TRUNCATED",
            "MCP repo review scanned only part of the workspace.",
            "review_local_workspace scan.truncated=true",
            "Increase --max-files or narrow the workspace before treating governance evidence as complete.",
        ));
    }
}
