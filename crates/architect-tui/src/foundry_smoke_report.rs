use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::foundry::repo_target;
use crate::foundry_smoke::FoundrySmokeOptions;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FoundrySmokeStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokeReport {
    pub schema_version: u8,
    pub status: FoundrySmokeStatus,
    pub mode: String,
    pub target: String,
    pub workspace: PathBuf,
    pub staged_repo: Option<PathBuf>,
    pub staged_artifacts: usize,
    pub execution_status: Option<String>,
    pub github: Option<FoundryGithubVerification>,
    pub commands: Vec<FoundrySmokeCommandReport>,
    pub error: Option<String>,
    pub retention: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokeCommandReport {
    pub command: String,
    pub ok: bool,
    pub transcript: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundryGithubVerification {
    pub repo_url: String,
    pub repo_visibility: String,
    pub is_private: bool,
    pub draft_pr_url: Option<String>,
    pub draft_pr_number: Option<u64>,
}

pub(crate) fn failed_report(
    options: &FoundrySmokeOptions,
    error: &anyhow::Error,
) -> FoundrySmokeReport {
    let repo = options
        .repo
        .clone()
        .unwrap_or_else(|| "unallocated-foundry-smoke-repo".to_string());
    FoundrySmokeReport {
        schema_version: 1,
        status: FoundrySmokeStatus::Failed,
        mode: if options.execute { "live" } else { "dry_run" }.to_string(),
        target: repo_target(Some(&options.owner), &repo),
        workspace: PathBuf::new(),
        staged_repo: None,
        staged_artifacts: 0,
        execution_status: None,
        github: None,
        commands: Vec::new(),
        error: Some(error.to_string()),
        retention: "no GitHub repo created".to_string(),
    }
}

pub(crate) fn print_text_report(report: &FoundrySmokeReport) {
    println!("architect-mcp-tui foundry smoke: {:?}", report.status);
    if let Some(error) = &report.error {
        println!("error: {error}");
    }
    println!("mode: {}", report.mode);
    println!("target: {}", report.target);
    println!("workspace: {}", report.workspace.display());
    if let Some(staged_repo) = &report.staged_repo {
        println!("staged repo: {}", staged_repo.display());
    }
    if let Some(github) = &report.github {
        println!("repo: {}", github.repo_url);
        if let Some(pr_url) = &github.draft_pr_url {
            println!("draft PR: {pr_url}");
        }
    }
    println!("retention: {}", report.retention);
}
