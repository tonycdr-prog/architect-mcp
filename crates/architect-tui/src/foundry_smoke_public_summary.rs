use serde::Serialize;

use crate::foundry_smoke_report::{FoundrySmokeReport, FoundrySmokeStatus};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokePublicSummary {
    pub schema_version: u8,
    pub status: FoundrySmokeStatus,
    pub mode: String,
    pub public_safe: bool,
    pub target_redacted: bool,
    pub mutation: FoundrySmokeMutationSummary,
    pub staged_artifacts: usize,
    pub github: Option<FoundrySmokeGithubPublicSummary>,
    pub commands: FoundrySmokeCommandPublicSummary,
    pub retention: String,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
    pub omitted: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokeMutationSummary {
    pub github_mutation: bool,
    pub requires_execute: bool,
    pub requires_confirm_private_repo_mutation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokeGithubPublicSummary {
    pub private_repo_verified: bool,
    pub draft_pr_verified: bool,
    pub draft_pr_state: Option<String>,
    pub draft_pr_is_draft: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundrySmokeCommandPublicSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

pub(crate) fn print_foundry_smoke_public_summary(
    report: &FoundrySmokeReport,
) -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&build_foundry_smoke_public_summary(report))?
    );
    Ok(())
}

pub(crate) fn build_foundry_smoke_public_summary(
    report: &FoundrySmokeReport,
) -> FoundrySmokePublicSummary {
    let github = report.github.as_ref().map(|github| {
        let draft_pr_verified = github.draft_pr_number.is_some()
            && github.draft_pr_is_draft == Some(true)
            && github.draft_pr_state.as_deref() == Some("OPEN");
        FoundrySmokeGithubPublicSummary {
            private_repo_verified: github.is_private,
            draft_pr_verified,
            draft_pr_state: github
                .draft_pr_state
                .as_ref()
                .map(|state| public_text(state, 40)),
            draft_pr_is_draft: github.draft_pr_is_draft,
        }
    });
    let commands = FoundrySmokeCommandPublicSummary {
        total: report.commands.len(),
        passed: report.commands.iter().filter(|command| command.ok).count(),
        failed: report.commands.iter().filter(|command| !command.ok).count(),
    };
    let mut findings = Vec::new();
    if let Some(error) = &report.error {
        findings.push(public_error_text(report, error, 240));
    }
    if report.mode == "live" && github.is_none() {
        findings.push("live foundry smoke did not verify private GitHub repo evidence".to_string());
    }
    if report.status == FoundrySmokeStatus::Failed && findings.is_empty() {
        findings.push("foundry smoke failed without a public-safe error summary".to_string());
    }

    FoundrySmokePublicSummary {
        schema_version: 1,
        status: report.status.clone(),
        mode: public_report_text(report, &report.mode, 40),
        public_safe: true,
        target_redacted: true,
        mutation: FoundrySmokeMutationSummary {
            github_mutation: report.mode == "live",
            requires_execute: report.mode == "live",
            requires_confirm_private_repo_mutation: report.mode == "live",
        },
        staged_artifacts: report.staged_artifacts,
        github,
        commands,
        retention: public_report_text(report, &report.retention, 200),
        findings,
        next_actions: next_actions(report),
        omitted: vec![
            "target repository name and URL".to_string(),
            "workspace path".to_string(),
            "staged repository path".to_string(),
            "raw command strings".to_string(),
            "stdout and stderr tails".to_string(),
            "raw command transcripts".to_string(),
            "raw MCP payloads".to_string(),
        ],
    }
}

fn next_actions(report: &FoundrySmokeReport) -> Vec<String> {
    if report.status == FoundrySmokeStatus::Failed {
        return vec![
            "inspect local foundry-smoke JSON output before sharing public evidence".to_string(),
        ];
    }
    if report.mode == "dry_run" {
        return vec![
            "review staged repo artifacts locally".to_string(),
            "run live foundry-smoke only with --execute --confirm-private-repo-mutation when a private proof repo is intended".to_string(),
        ];
    }
    vec!["decide whether to retain or delete the private proof repo after review".to_string()]
}

fn public_report_text(report: &FoundrySmokeReport, value: &str, max_len: usize) -> String {
    let mut text = value.to_string();
    for secret in report_specific_values(report) {
        if !secret.is_empty() {
            text = text.replace(&secret, "[redacted-target]");
        }
    }
    public_text(&text, max_len)
}

fn public_error_text(report: &FoundrySmokeReport, value: &str, max_len: usize) -> String {
    let text = public_report_text(report, value, max_len);
    let lower = text.to_ascii_lowercase();
    if lower.contains("stdout") || lower.contains("stderr") || lower.contains("transcript") {
        return "foundry smoke failed; raw command output omitted".to_string();
    }
    if lower.contains("\"jsonrpc\"") || lower.contains("\"method\"") || lower.contains("tools/call")
    {
        return "foundry smoke failed during MCP exchange; raw payload omitted".to_string();
    }
    if lower.contains("gh repo ")
        || lower.contains("gh pr ")
        || lower.contains("foundry create")
        || lower.contains("foundry stage")
        || lower.contains("foundry plan")
    {
        return "foundry smoke failed; raw command text omitted".to_string();
    }
    text
}

fn report_specific_values(report: &FoundrySmokeReport) -> Vec<String> {
    let mut values = vec![report.target.clone()];
    if let Some(repo_name) = report
        .target
        .rsplit_once('/')
        .map(|(_, repo)| repo)
        .filter(|repo| repo.chars().count() > 3)
    {
        values.push(repo_name.to_string());
    }
    if let Some(github) = &report.github {
        values.push(github.repo_url.clone());
        if let Some(draft_pr_url) = &github.draft_pr_url {
            values.push(draft_pr_url.clone());
        }
    }
    values
}
