use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::config::ConfigCommand;
use crate::foundry_smoke::FoundrySmokeOptions;
use crate::foundry_smoke_retention::FoundrySmokeRetentionDecision;
use crate::smoke::SmokeOptions;

#[derive(Debug, Parser)]
#[command(name = "architect-mcp-tui")]
#[command(about = "Ratatui agent client for architect-mcp work-gate workflows.")]
pub struct Cli {
    #[arg(long, global = true, env = "ARCHITECT_MCP_TUI_CONFIG")]
    pub config: Option<PathBuf>,
    #[arg(long, global = true, env = "ARCHITECT_MCP_WORKSPACE")]
    pub workspace: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a scriptable headless prompt through the work gate and one adapter.
    Run {
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value = "codex")]
        adapter: String,
        #[arg(long)]
        jsonl: bool,
        #[arg(long, default_value_t = 1)]
        concurrency: usize,
        #[arg(long)]
        execute: bool,
    },
    /// Serve an Agent Client Protocol endpoint over stdio.
    Acp {
        #[arg(long)]
        stdio: bool,
    },
    /// Manage architect-mcp-tui configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Run a secret-safe terminal QA smoke report.
    Smoke {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = SmokeOptions::DEFAULT_PROMPT)]
        prompt: String,
        #[arg(long)]
        skip_gate: bool,
    },
    /// Generate public-safe terminal evidence for launch-judge input.
    TerminalEvidence {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long, default_value = SmokeOptions::DEFAULT_PROMPT)]
        prompt: String,
        #[arg(long)]
        skip_gate: bool,
        #[arg(long)]
        platform: Option<String>,
        #[arg(
            long,
            value_name = "local-terminal|vm-or-cloud-terminal|container|hosted-ci|unknown"
        )]
        environment: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long, value_name = "YYYY-MM-DD")]
        collected_at: Option<String>,
        #[arg(long, value_name = "URL")]
        issue_url: Option<String>,
    },
    /// Run a scripted interactive command-palette walkthrough in a throwaway workspace.
    Walkthrough {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        keep_workspace: bool,
    },
    /// Run a local-only real-adapter promotion smoke in a disposable git workspace.
    PromotionSmoke {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "codex")]
        adapter: String,
        #[arg(long)]
        keep_workspace: bool,
        #[arg(long, default_value_t = 600)]
        timeout_seconds: u64,
    },
    /// Run a repo-foundry smoke; live GitHub creation requires explicit confirmation.
    FoundrySmoke {
        #[command(flatten)]
        options: FoundrySmokeCliOptions,
    },
    /// Run a read-only governance and drift audit for the current workspace.
    GovernanceAudit {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        public_summary: bool,
        #[arg(long)]
        skip_mcp: bool,
        #[arg(long, default_value_t = 1000, value_parser = parse_max_files)]
        max_files: usize,
    },
    /// Combine smoke, governance, release, and external-evidence gates into a launch judge report.
    LaunchJudge {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        public_summary: bool,
        #[arg(long)]
        skip_mcp: bool,
        #[arg(long)]
        skip_smoke: bool,
        #[arg(long)]
        run_release_check: bool,
        #[arg(long)]
        require_clean_git: bool,
        #[arg(long, default_value_t = 1000)]
        max_files: usize,
        #[arg(long)]
        terminal_evidence: Vec<PathBuf>,
    },
    /// Summarize a PR stack and external blockers into a public-safe launch decision.
    LaunchStack {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        merge_plan: bool,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long = "stack-from-pr")]
        stack_from_pr: Option<u64>,
        #[arg(long = "pr")]
        prs: Vec<u64>,
        #[arg(long = "blocker")]
        blockers: Vec<u64>,
        #[arg(long = "waive-blocker", value_name = "ISSUE=REASON")]
        waived_blockers: Vec<String>,
        #[arg(long = "required-check")]
        required_checks: Vec<String>,
    },
    /// Combine launch-stack and public terminal-evidence issue state into one readiness report.
    LaunchReadiness {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        public_summary: bool,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long = "stack-from-pr")]
        stack_from_pr: Option<u64>,
        #[arg(long = "pr")]
        prs: Vec<u64>,
        #[arg(long = "blocker")]
        blockers: Vec<u64>,
        #[arg(long = "waive-blocker", value_name = "ISSUE=REASON")]
        waived_blockers: Vec<String>,
        #[arg(long = "required-check")]
        required_checks: Vec<String>,
        #[arg(long)]
        terminal_evidence_issue: Option<u64>,
        #[arg(long = "waive-terminal-evidence", value_name = "ISSUE=REASON")]
        terminal_evidence_waivers: Vec<String>,
    },
    /// Emit one public-safe release evidence index from launch and governance summaries.
    EvidenceIndex {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        markdown_output: Option<PathBuf>,
        #[arg(long)]
        require_go: bool,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long = "stack-from-pr")]
        stack_from_pr: Option<u64>,
        #[arg(long = "pr")]
        prs: Vec<u64>,
        #[arg(long = "blocker")]
        blockers: Vec<u64>,
        #[arg(long = "waive-blocker", value_name = "ISSUE=REASON")]
        waived_blockers: Vec<String>,
        #[arg(long = "required-check")]
        required_checks: Vec<String>,
        #[arg(long)]
        terminal_evidence_issue: Option<u64>,
        #[arg(long = "waive-terminal-evidence", value_name = "ISSUE=REASON")]
        terminal_evidence_waivers: Vec<String>,
        #[arg(long)]
        skip_mcp: bool,
        #[arg(long, default_value_t = 1000)]
        max_files: usize,
    },
    /// Collect public-safe terminal evidence JSON from a GitHub issue.
    CollectTerminalEvidence {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        issue: u64,
    },
}

fn parse_max_files(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "max-files must be an integer from 1 to 5000".to_string())?;
    if (1..=5000).contains(&parsed) {
        Ok(parsed)
    } else {
        Err("max-files must be from 1 to 5000".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_audit_max_files_matches_mcp_schema_range() {
        assert!(
            Cli::try_parse_from(["architect-mcp-tui", "governance-audit", "--max-files", "0"])
                .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "architect-mcp-tui",
                "governance-audit",
                "--max-files",
                "5001"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "architect-mcp-tui",
                "governance-audit",
                "--max-files",
                "5000"
            ])
            .is_ok()
        );
    }
}

#[derive(Debug, Args)]
pub struct FoundrySmokeCliOptions {
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub public_summary: bool,
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub execute: bool,
    #[arg(long)]
    pub confirm_private_repo_mutation: bool,
    #[arg(long)]
    pub keep_workspace: bool,
    #[arg(long, value_enum)]
    pub retention_decision: Option<FoundrySmokeRetentionDecision>,
    #[arg(long)]
    pub retention_reason: Option<String>,
}

impl From<FoundrySmokeCliOptions> for FoundrySmokeOptions {
    fn from(options: FoundrySmokeCliOptions) -> Self {
        Self {
            json: options.json,
            public_summary: options.public_summary,
            owner: options.owner,
            repo: options.repo,
            execute: options.execute,
            confirm_private_repo_mutation: options.confirm_private_repo_mutation,
            keep_workspace: options.keep_workspace,
            retention_decision: options.retention_decision,
            retention_reason: options.retention_reason,
        }
    }
}
