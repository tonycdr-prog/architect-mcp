use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack_github::{fetch_issue, fetch_pr, public_text};

#[derive(Debug, Clone)]
pub struct LaunchStackOptions {
    pub json: bool,
    pub repo: Option<String>,
    pub prs: Vec<u64>,
    pub blockers: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStackReport {
    pub schema_version: u8,
    pub result: LaunchJudgeResult,
    pub repository: Option<String>,
    pub pull_requests: Vec<LaunchStackPullRequest>,
    pub blocker_issues: Vec<LaunchStackIssue>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchStackItemStatus {
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStackPullRequest {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub is_draft: bool,
    pub merge_state_status: String,
    pub checks: LaunchStackCheckSummary,
    pub status: LaunchStackItemStatus,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStackCheckSummary {
    pub total: usize,
    pub passed: usize,
    pub pending: usize,
    pub failed: usize,
    pub pending_names: Vec<String>,
    pub failed_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStackIssue {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: String,
    pub status: LaunchStackItemStatus,
    pub next_action: Option<String>,
}

pub fn run_launch_stack(workspace: &Path, options: LaunchStackOptions) -> Result<()> {
    let report = build_launch_stack_report(workspace, &options);
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }
    if report.result == LaunchJudgeResult::NoGo {
        anyhow::bail!("launch stack result is no-go");
    }
    Ok(())
}

pub fn build_launch_stack_report(
    workspace: &Path,
    options: &LaunchStackOptions,
) -> LaunchStackReport {
    let mut prs = Vec::new();
    let mut blockers = Vec::new();
    let mut findings = Vec::new();

    if options.prs.is_empty() && options.blockers.is_empty() {
        findings.push("no PRs or blocker issues were supplied for launch-stack".to_string());
    }

    for number in &options.prs {
        match fetch_pr(workspace, options.repo.as_deref(), *number) {
            Ok(pr) => prs.push(pr),
            Err(error) => findings.push(error),
        }
    }
    for number in &options.blockers {
        match fetch_issue(workspace, options.repo.as_deref(), *number) {
            Ok(issue) => blockers.push(issue),
            Err(error) => findings.push(error),
        }
    }

    build_report_from_items(options.repo.clone(), prs, blockers, findings)
}

pub(crate) fn build_report_from_items(
    repository: Option<String>,
    pull_requests: Vec<LaunchStackPullRequest>,
    blocker_issues: Vec<LaunchStackIssue>,
    findings: Vec<String>,
) -> LaunchStackReport {
    let mut next_actions: Vec<String> = Vec::new();
    for pr in &pull_requests {
        if let Some(action) = &pr.next_action {
            next_actions.push(action.clone());
        }
    }
    for issue in &blocker_issues {
        if let Some(action) = &issue.next_action {
            next_actions.push(action.clone());
        }
    }
    if !findings.is_empty() {
        next_actions
            .push("fix launch-stack lookup errors before relying on this report".to_string());
    }

    let result = if !findings.is_empty()
        || pull_requests
            .iter()
            .any(|pr| pr.status == LaunchStackItemStatus::Failed)
        || blocker_issues
            .iter()
            .any(|issue| issue.status == LaunchStackItemStatus::Failed)
    {
        LaunchJudgeResult::NoGo
    } else if pull_requests
        .iter()
        .any(|pr| pr.status == LaunchStackItemStatus::Warning)
        || blocker_issues
            .iter()
            .any(|issue| issue.status == LaunchStackItemStatus::Warning)
    {
        LaunchJudgeResult::ConditionalGo
    } else {
        LaunchJudgeResult::Go
    };

    LaunchStackReport {
        schema_version: 1,
        result,
        repository: repository.map(|repo| public_text(&repo, 160)),
        pull_requests,
        blocker_issues,
        findings: findings
            .iter()
            .map(|finding| public_text(finding, 320))
            .collect(),
        next_actions,
    }
}

fn print_text_report(report: &LaunchStackReport) {
    println!("architect-mcp-tui launch stack: {:?}", report.result);
    for pr in &report.pull_requests {
        println!(
            "- PR #{}: {:?} draft={} merge={} checks passed={} pending={} failed={}",
            pr.number,
            pr.status,
            pr.is_draft,
            pr.merge_state_status,
            pr.checks.passed,
            pr.checks.pending,
            pr.checks.failed
        );
    }
    for issue in &report.blocker_issues {
        println!(
            "- issue #{}: {:?} state={}",
            issue.number, issue.status, issue.state
        );
    }
    for finding in &report.findings {
        println!("- finding: {finding}");
    }
    if !report.next_actions.is_empty() {
        println!("next actions:");
        for action in &report.next_actions {
            println!("- {action}");
        }
    }
}
