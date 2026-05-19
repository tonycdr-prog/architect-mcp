use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::issue_terminal_evidence::{
    IssueTerminalEvidenceOptions, IssueTerminalEvidenceReport, build_issue_terminal_evidence_report,
};
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_stack::{LaunchStackOptions, LaunchStackReport, build_launch_stack_report};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone)]
pub struct LaunchReadinessOptions {
    pub json: bool,
    pub repo: Option<String>,
    pub prs: Vec<u64>,
    pub blockers: Vec<u64>,
    pub waived_blockers: Vec<String>,
    pub terminal_evidence_issue: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessReport {
    pub schema_version: u8,
    pub result: LaunchJudgeResult,
    pub repository: Option<String>,
    pub read_only: bool,
    pub launch_stack: LaunchStackReport,
    pub terminal_evidence_issue: Option<IssueTerminalEvidenceReport>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

pub fn run_launch_readiness(workspace: &Path, options: LaunchReadinessOptions) -> Result<()> {
    let report = build_launch_readiness_report(workspace, &options);
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }
    if report.result == LaunchJudgeResult::NoGo {
        anyhow::bail!("launch readiness result is no-go");
    }
    Ok(())
}

pub fn build_launch_readiness_report(
    workspace: &Path,
    options: &LaunchReadinessOptions,
) -> LaunchReadinessReport {
    let stack = build_launch_stack_report(
        workspace,
        &LaunchStackOptions {
            json: true,
            repo: options.repo.clone(),
            prs: options.prs.clone(),
            blockers: options.blockers.clone(),
            waived_blockers: options.waived_blockers.clone(),
        },
    );
    let terminal_evidence = options.terminal_evidence_issue.map(|issue| {
        build_issue_terminal_evidence_report(
            workspace,
            &IssueTerminalEvidenceOptions {
                json: true,
                repo: options.repo.clone(),
                issue,
            },
        )
    });
    build_launch_readiness_report_from_reports(options.repo.clone(), stack, terminal_evidence)
}

pub(crate) fn build_launch_readiness_report_from_reports(
    repository: Option<String>,
    launch_stack: LaunchStackReport,
    terminal_evidence_issue: Option<IssueTerminalEvidenceReport>,
) -> LaunchReadinessReport {
    let mut findings = launch_stack.findings.clone();
    let mut next_actions = launch_stack.next_actions.clone();
    let mut result_inputs = vec![launch_stack.result.clone()];

    if let Some(evidence) = &terminal_evidence_issue {
        findings.extend(evidence.findings.clone());
        next_actions.extend(evidence.next_actions.clone());
        result_inputs.push(evidence.result.clone());
    } else {
        result_inputs.push(LaunchJudgeResult::ConditionalGo);
        next_actions.push(
            "supply --terminal-evidence-issue with the public Linux/Windows QA issue before final launch go"
                .to_string(),
        );
    }

    dedupe(&mut findings);
    dedupe(&mut next_actions);

    LaunchReadinessReport {
        schema_version: 1,
        result: combine_results(&result_inputs),
        repository: repository.map(|repo| public_text(&repo, 160)),
        read_only: true,
        launch_stack,
        terminal_evidence_issue,
        findings,
        next_actions,
    }
}

fn combine_results(results: &[LaunchJudgeResult]) -> LaunchJudgeResult {
    if results
        .iter()
        .any(|result| result == &LaunchJudgeResult::NoGo)
    {
        LaunchJudgeResult::NoGo
    } else if results
        .iter()
        .any(|result| result == &LaunchJudgeResult::ConditionalGo)
    {
        LaunchJudgeResult::ConditionalGo
    } else {
        LaunchJudgeResult::Go
    }
}

fn dedupe(values: &mut Vec<String>) {
    let mut seen = std::collections::BTreeSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

fn print_text_report(report: &LaunchReadinessReport) {
    println!("architect-mcp-tui launch readiness: {:?}", report.result);
    println!("- read-only: {}", report.read_only);
    println!("- stack: {:?}", report.launch_stack.result);
    match &report.terminal_evidence_issue {
        Some(evidence) => {
            println!("- terminal evidence issue: {:?}", evidence.result);
            println!(
                "- terminal evidence reports: {}",
                evidence.terminal_evidence.reports.len()
            );
        }
        None => {
            println!("- terminal evidence issue: not supplied");
        }
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
