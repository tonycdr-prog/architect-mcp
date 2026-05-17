use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack_discovery::{LaunchStackDiscovery, resolve_launch_stack_pr_numbers};
use crate::launch_stack_github::{fetch_issue, fetch_pr, public_text};
use crate::launch_stack_merge_plan::print_launch_stack_merge_plan;
use crate::launch_stack_output::print_launch_stack_text_report;
use crate::launch_stack_waivers::apply_waivers;

pub(crate) use crate::launch_stack_waivers::parse_waivers;

#[derive(Debug, Clone)]
pub struct LaunchStackOptions {
    pub json: bool,
    pub merge_plan: bool,
    pub repo: Option<String>,
    pub stack_from_pr: Option<u64>,
    pub prs: Vec<u64>,
    pub blockers: Vec<u64>,
    pub waived_blockers: Vec<String>,
    pub required_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStackReport {
    pub schema_version: u8,
    pub result: LaunchJudgeResult,
    pub repository: Option<String>,
    pub stack_discovery: Option<LaunchStackDiscovery>,
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
    pub review_decision: Option<String>,
    pub merge_state_status: String,
    pub mergeable: Option<String>,
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
    pub names: Vec<String>,
    pub pending_names: Vec<String>,
    pub failed_names: Vec<String>,
    pub missing_required_names: Vec<String>,
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
    if options.json && options.merge_plan {
        anyhow::bail!("--json and --merge-plan are mutually exclusive");
    }
    let report = build_launch_stack_report(workspace, &options);
    if options.merge_plan {
        print_launch_stack_merge_plan(&report);
    } else if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_launch_stack_text_report(&report);
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

    if options.stack_from_pr.is_none() && options.prs.is_empty() && options.blockers.is_empty() {
        findings.push("no PRs or blocker issues were supplied for launch-stack".to_string());
    }

    let (pr_numbers, stack_discovery, discovery_findings) = resolve_launch_stack_pr_numbers(
        workspace,
        options.repo.as_deref(),
        options.stack_from_pr,
        &options.prs,
    );
    findings.extend(discovery_findings);
    for number in pr_numbers {
        match fetch_pr(
            workspace,
            options.repo.as_deref(),
            number,
            &options.required_checks,
        ) {
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

    build_report_from_items_with_waivers_and_discovery(
        options.repo.clone(),
        prs,
        blockers,
        findings,
        waivers,
        stack_discovery,
    )
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

#[cfg(test)]
pub(crate) fn build_report_from_items_with_waivers(
    repository: Option<String>,
    pull_requests: Vec<LaunchStackPullRequest>,
    blocker_issues: Vec<LaunchStackIssue>,
    findings: Vec<String>,
    waivers: BTreeMap<u64, String>,
) -> LaunchStackReport {
    build_report_from_items_with_waivers_and_discovery(
        repository,
        pull_requests,
        blocker_issues,
        findings,
        waivers,
        None,
    )
}

fn build_report_from_items_with_waivers_and_discovery(
    repository: Option<String>,
    pull_requests: Vec<LaunchStackPullRequest>,
    mut blocker_issues: Vec<LaunchStackIssue>,
    findings: Vec<String>,
    waivers: BTreeMap<u64, String>,
    stack_discovery: Option<LaunchStackDiscovery>,
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
        stack_discovery,
        pull_requests,
        blocker_issues,
        findings: findings
            .iter()
            .map(|finding| public_text(finding, 320))
            .collect(),
        next_actions,
    }
}
