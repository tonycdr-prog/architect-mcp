use std::collections::{BTreeMap, HashSet};
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
    pub waived_blockers: Vec<String>,
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
    Waived,
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
    pub waiver_reason: Option<String>,
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
    let (waivers, waiver_findings) = parse_waivers(&options.waived_blockers);
    findings.extend(waiver_findings);

    if options.prs.is_empty() && options.blockers.is_empty() {
        findings.push("no PRs or blocker issues were supplied for launch-stack".to_string());
    }

    for number in &options.prs {
        match fetch_pr(workspace, options.repo.as_deref(), *number) {
            Ok(pr) => prs.push(pr),
            Err(error) => findings.push(error),
        }
    }
    let mut blocker_numbers = HashSet::new();
    for number in &options.blockers {
        blocker_numbers.insert(*number);
        match fetch_issue(workspace, options.repo.as_deref(), *number) {
            Ok(issue) => blockers.push(issue),
            Err(error) => findings.push(error),
        }
    }
    for number in waivers.keys() {
        if !blocker_numbers.contains(number) {
            findings.push(format!(
                "waiver supplied for issue #{number}, but that issue was not supplied as a blocker"
            ));
        }
    }

    build_report_from_items_with_waivers(options.repo.clone(), prs, blockers, findings, waivers)
}

#[cfg(test)]
pub(crate) fn build_report_from_items(
    repository: Option<String>,
    pull_requests: Vec<LaunchStackPullRequest>,
    blocker_issues: Vec<LaunchStackIssue>,
    findings: Vec<String>,
) -> LaunchStackReport {
    build_report_from_items_with_waivers(
        repository,
        pull_requests,
        blocker_issues,
        findings,
        BTreeMap::new(),
    )
}

pub(crate) fn build_report_from_items_with_waivers(
    repository: Option<String>,
    pull_requests: Vec<LaunchStackPullRequest>,
    mut blocker_issues: Vec<LaunchStackIssue>,
    findings: Vec<String>,
    waivers: BTreeMap<u64, String>,
) -> LaunchStackReport {
    apply_waivers(&mut blocker_issues, &waivers);
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

fn apply_waivers(blocker_issues: &mut [LaunchStackIssue], waivers: &BTreeMap<u64, String>) {
    for issue in blocker_issues {
        if issue.status == LaunchStackItemStatus::Warning
            && issue.state == "OPEN"
            && let Some(reason) = waivers.get(&issue.number)
        {
            issue.status = LaunchStackItemStatus::Waived;
            issue.waiver_reason = Some(reason.clone());
            issue.next_action = None;
        }
    }
}

pub(crate) fn parse_waivers(raw_waivers: &[String]) -> (BTreeMap<u64, String>, Vec<String>) {
    let mut waivers = BTreeMap::new();
    let mut findings = Vec::new();
    for raw in raw_waivers {
        match parse_waiver(raw) {
            Ok((number, reason)) => {
                if waivers.insert(number, reason).is_some() {
                    findings.push(format!("duplicate waiver supplied for issue #{number}"));
                }
            }
            Err(error) => findings.push(error),
        }
    }
    (waivers, findings)
}

fn parse_waiver(raw: &str) -> Result<(u64, String), String> {
    let Some((number, reason)) = raw.split_once('=').or_else(|| raw.split_once(':')) else {
        return Err(
            "blocker waiver must use ISSUE=reason or ISSUE:reason with a public reason".to_string(),
        );
    };
    let number = number
        .trim()
        .trim_start_matches('#')
        .parse::<u64>()
        .map_err(|_| "blocker waiver issue number must be numeric".to_string())?;
    let reason = public_text(reason, 240);
    if reason.is_empty() {
        return Err(format!(
            "blocker waiver for issue #{number} needs a public reason"
        ));
    }
    Ok((number, reason))
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
        print!(
            "- issue #{}: {:?} state={}",
            issue.number, issue.status, issue.state
        );
        if let Some(reason) = &issue.waiver_reason {
            print!(" waiver=\"{reason}\"");
        }
        println!();
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
