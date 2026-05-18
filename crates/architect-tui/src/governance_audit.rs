use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::TuiConfig;
use crate::governance_audit_mcp::run_mcp_repo_review;
use crate::governance_audit_public_summary::build_public_summary;
use crate::governance_audit_report::{
    GovernanceAuditReport, GovernanceAuditStatus, GovernanceFinding, GovernanceMcpReview,
    print_text_report,
};
use crate::governance_audit_support::{
    categories, deterministic_gates, error, package_scripts, read_to_string, smoke_evidence,
    status_for_findings, warning,
};
use crate::governance_memory::memory_proposals;
use crate::governance_secret_scan::{governance_config_files, looks_secret_like};

#[derive(Debug, Clone)]
pub struct GovernanceAuditOptions {
    pub json: bool,
    pub public_summary: bool,
    pub skip_mcp: bool,
    pub max_files: usize,
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
    let mut findings = static_findings(&workspace);
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

    let deterministic_gates = deterministic_gates(&workspace);
    let smoke_evidence = smoke_evidence(&workspace);
    let memory_proposals = memory_proposals(&workspace);
    let categories = categories(&findings);
    let status = status_for_findings(&findings);

    GovernanceAuditReport {
        schema_version: 1,
        status,
        workspace: workspace.display().to_string(),
        read_only: true,
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

fn static_findings(workspace: &Path) -> Vec<GovernanceFinding> {
    let mut findings = Vec::new();
    required_file(&mut findings, workspace, "AGENTS.md", "agent-instructions");
    required_file(
        &mut findings,
        workspace,
        "docs/architecture-contract.md",
        "architecture-contract",
    );
    required_file(&mut findings, workspace, "docs/build-plan.md", "build-plan");
    required_file(&mut findings, workspace, "README.md", "public-docs");
    required_file(&mut findings, workspace, "llms.txt", "public-docs");
    required_file(&mut findings, workspace, ".env.example", "environment");
    required_file(
        &mut findings,
        workspace,
        "package-lock.json",
        "dependencies",
    );
    required_file(&mut findings, workspace, "Cargo.lock", "dependencies");
    required_file(
        &mut findings,
        workspace,
        ".github/dependabot.yml",
        "dependencies",
    );
    required_file(&mut findings, workspace, ".github/workflows/ci.yml", "ci");

    check_package_scripts(&mut findings, workspace);
    check_workflows(&mut findings, workspace);
    check_memory_policy(&mut findings, workspace);
    check_secret_files(&mut findings, workspace);
    findings
}

fn required_file(
    findings: &mut Vec<GovernanceFinding>,
    workspace: &Path,
    relative: &str,
    category: &str,
) {
    let path = workspace.join(relative);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() && metadata.len() > 0 => {}
        _ => findings.push(error(
            category,
            "GOV_REQUIRED_FILE_MISSING",
            &format!("{relative} is missing or empty."),
            relative,
            "Restore the required governance artifact before release.",
        )),
    }
}

fn check_package_scripts(findings: &mut Vec<GovernanceFinding>, workspace: &Path) {
    let scripts = package_scripts(workspace).unwrap_or_default();
    for script in [
        "release:check",
        "rust:check",
        "typecheck",
        "test",
        "build",
        "docs:build",
        "tui:live-qa",
    ] {
        if !scripts.contains_key(script) {
            findings.push(error(
                "release-readiness",
                "GOV_PACKAGE_SCRIPT_MISSING",
                &format!("package.json is missing the {script} script."),
                "package.json",
                "Add the script or update the audit policy with the replacement command.",
            ));
        }
    }

    if scripts
        .get("release:check")
        .is_some_and(|command| !command.contains("check:v10"))
    {
        findings.push(warning(
            "release-readiness",
            "GOV_RELEASE_GATE_WEAK",
            "release:check does not mention the staged readiness gate.",
            "package.json scripts.release:check",
            "Keep npm run release:check as the clean-checkout release gate.",
        ));
    }
}

fn check_workflows(findings: &mut Vec<GovernanceFinding>, workspace: &Path) {
    let ci = read_to_string(workspace, ".github/workflows/ci.yml");
    for command in [
        "npm run rust:check",
        "npm run typecheck",
        "npm test",
        "npm run build",
    ] {
        if !ci.contains(command) {
            findings.push(warning(
                "ci",
                "GOV_CI_CHECK_MISSING",
                &format!("CI workflow does not mention {command}."),
                ".github/workflows/ci.yml",
                "Keep deterministic CI checks visible in the verify workflow.",
            ));
        }
    }

    let publish = read_to_string(workspace, ".github/workflows/npm-publish.yml");
    if !publish.contains("npm run release:check") {
        findings.push(error(
            "release-readiness",
            "GOV_RELEASE_GATE_NOT_ENFORCED",
            "The npm publish workflow does not run npm run release:check.",
            ".github/workflows/npm-publish.yml",
            "Run the clean-checkout release gate before package publication.",
        ));
    }

    let live_qa = read_to_string(workspace, ".github/workflows/tui-live-qa.yml");
    if !live_qa.contains("npm run tui:live-qa") {
        findings.push(warning(
            "qa",
            "GOV_TUI_LIVE_QA_MISSING",
            "TUI live QA workflow does not run npm run tui:live-qa.",
            ".github/workflows/tui-live-qa.yml",
            "Keep smoke evidence separate from deterministic release gates.",
        ));
    }
}

fn check_memory_policy(findings: &mut Vec<GovernanceFinding>, workspace: &Path) {
    let agents = read_to_string(workspace, "AGENTS.md").to_ascii_lowercase();
    for phrase in [
        "do not store secrets",
        "raw conversation logs",
        "transient task",
    ] {
        if !agents.contains(phrase) {
            findings.push(warning(
                "memory",
                "GOV_MEMORY_POLICY_GAP",
                &format!("AGENTS.md does not mention '{phrase}'."),
                "AGENTS.md",
                "Keep the memory policy explicit about durable context and prohibited data.",
            ));
        }
    }
}

fn check_secret_files(findings: &mut Vec<GovernanceFinding>, workspace: &Path) {
    for path in governance_config_files(workspace) {
        let relative = path
            .strip_prefix(workspace)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        let content = fs::read_to_string(&path).unwrap_or_default();
        if looks_secret_like(&content) {
            findings.push(error(
                "security",
                "GOV_SECRET_SHAPED_CONFIG",
                &format!("{relative} contains secret-shaped text."),
                &relative,
                "Remove real credentials and use environment-variable placeholders.",
            ));
        }
    }
}
