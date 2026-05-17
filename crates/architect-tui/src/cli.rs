use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::ConfigCommand;
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
        #[arg(long, default_value = SmokeOptions::DEFAULT_PROMPT)]
        prompt: String,
        #[arg(long)]
        skip_gate: bool,
        #[arg(long)]
        platform: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        notes: Option<String>,
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
        #[arg(long)]
        json: bool,
        #[arg(long)]
        owner: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        confirm_private_repo_mutation: bool,
        #[arg(long)]
        keep_workspace: bool,
    },
    /// Run a read-only governance and drift audit for the current workspace.
    GovernanceAudit {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        public_summary: bool,
        #[arg(long)]
        skip_mcp: bool,
        #[arg(long, default_value_t = 1000)]
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
        repo: Option<String>,
        #[arg(long = "stack-from-pr")]
        stack_from_pr: Option<u64>,
        #[arg(long = "pr")]
        prs: Vec<u64>,
        #[arg(long = "blocker")]
        blockers: Vec<u64>,
        #[arg(long = "waive-blocker", value_name = "ISSUE=REASON")]
        waived_blockers: Vec<String>,
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
        repo: Option<String>,
        #[arg(long = "stack-from-pr")]
        stack_from_pr: Option<u64>,
        #[arg(long = "pr")]
        prs: Vec<u64>,
        #[arg(long = "blocker")]
        blockers: Vec<u64>,
        #[arg(long = "waive-blocker", value_name = "ISSUE=REASON")]
        waived_blockers: Vec<String>,
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
