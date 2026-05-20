use std::path::PathBuf;

use clap::Args;

use crate::cli::parse_max_files;
use crate::foundry_smoke::FoundrySmokeOptions;
use crate::foundry_smoke_retention::FoundrySmokeRetentionDecision;

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

#[derive(Debug, Args)]
pub struct FoundryAuditCliOptions {
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub public_summary: bool,
    #[arg(long)]
    pub repo_path: Option<PathBuf>,
    #[arg(long, default_value_t = 1000, value_parser = parse_max_files)]
    pub max_files: usize,
}
