use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::issue_terminal_evidence::{
    IssueTerminalEvidenceOptions, IssueTerminalEvidenceReport, build_issue_terminal_evidence_report,
};
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness_output::print_text_report;
use crate::launch_readiness_public_summary::build_public_summary;
use crate::launch_stack::{LaunchStackOptions, LaunchStackReport, build_launch_stack_report};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone)]
pub struct LaunchReadinessOptions {
    pub json: bool,
    pub public_summary: bool,
    pub repo: Option<String>,
    pub stack_from_pr: Option<u64>,
    pub prs: Vec<u64>,
    pub blockers: Vec<u64>,
    pub waived_blockers: Vec<String>,
    pub required_checks: Vec<String>,
    pub terminal_evidence_issue: Option<u64>,
    pub terminal_evidence_waivers: Vec<String>,
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
    pub terminal_evidence_waiver: Option<LaunchReadinessTerminalEvidenceWaiver>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessTerminalEvidenceWaiver {
    pub issue: u64,
    pub reason: String,
    pub applied: bool,
}

pub fn run_launch_readiness(workspace: &Path, options: LaunchReadinessOptions) -> Result<()> {
    let report = build_launch_readiness_report(workspace, &options);
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
            merge_plan: false,
            repo: options.repo.clone(),
            stack_from_pr: options.stack_from_pr,
            prs: options.prs.clone(),
            blockers: options.blockers.clone(),
            waived_blockers: options.waived_blockers.clone(),
            required_checks: options.required_checks.clone(),
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
    let (terminal_evidence_waiver, waiver_findings) = parse_terminal_evidence_waiver(
        options.terminal_evidence_issue,
        &options.terminal_evidence_waivers,
    );
    build_launch_readiness_report_from_reports_with_terminal_waiver(
        options.repo.clone(),
        stack,
        terminal_evidence,
        terminal_evidence_waiver,
        waiver_findings,
    )
}

#[cfg(test)]
pub(crate) fn build_launch_readiness_report_from_reports(
    repository: Option<String>,
    launch_stack: LaunchStackReport,
    terminal_evidence_issue: Option<IssueTerminalEvidenceReport>,
) -> LaunchReadinessReport {
    build_launch_readiness_report_from_reports_with_terminal_waiver(
        repository,
        launch_stack,
        terminal_evidence_issue,
        None,
        Vec::new(),
    )
}

pub(crate) fn build_launch_readiness_report_from_reports_with_terminal_waiver(
    repository: Option<String>,
    launch_stack: LaunchStackReport,
    terminal_evidence_issue: Option<IssueTerminalEvidenceReport>,
    mut terminal_evidence_waiver: Option<LaunchReadinessTerminalEvidenceWaiver>,
    terminal_evidence_waiver_findings: Vec<String>,
) -> LaunchReadinessReport {
    let mut findings = launch_stack.findings.clone();
    let mut next_actions = launch_stack.next_actions.clone();
    let mut result_inputs = vec![launch_stack.result.clone()];
    let has_valid_terminal_waiver =
        terminal_evidence_waiver.is_some() && terminal_evidence_waiver_findings.is_empty();

    if !terminal_evidence_waiver_findings.is_empty() {
        findings.extend(terminal_evidence_waiver_findings);
        next_actions.push(
            "fix terminal-evidence waiver arguments before relying on this report".to_string(),
        );
        result_inputs.push(LaunchJudgeResult::NoGo);
    }

    if let Some(evidence) = &terminal_evidence_issue {
        findings.extend(evidence.findings.clone());
        if evidence.result == LaunchJudgeResult::ConditionalGo && has_valid_terminal_waiver {
            if let Some(waiver) = &mut terminal_evidence_waiver {
                waiver.applied = true;
            }
            result_inputs.push(LaunchJudgeResult::Go);
        } else {
            next_actions.extend(evidence.next_actions.clone());
            result_inputs.push(evidence.result.clone());
        }
    } else {
        if has_valid_terminal_waiver {
            findings.push(
                "terminal-evidence waiver was supplied but no terminal-evidence issue was supplied"
                    .to_string(),
            );
            result_inputs.push(LaunchJudgeResult::NoGo);
        } else {
            result_inputs.push(LaunchJudgeResult::ConditionalGo);
            next_actions.push(
                "supply --terminal-evidence-issue with the public Linux/Windows QA issue before final launch go"
                    .to_string(),
            );
        }
    }

    dedupe(&mut findings);
    dedupe(&mut next_actions);

    LaunchReadinessReport {
        schema_version: 2,
        result: combine_results(&result_inputs),
        repository: repository.map(|repo| public_text(&repo, 160)),
        read_only: true,
        launch_stack,
        terminal_evidence_issue,
        terminal_evidence_waiver,
        findings,
        next_actions,
    }
}

pub(crate) fn parse_terminal_evidence_waiver(
    terminal_evidence_issue: Option<u64>,
    raw_waivers: &[String],
) -> (Option<LaunchReadinessTerminalEvidenceWaiver>, Vec<String>) {
    let mut waiver = None;
    let mut findings = Vec::new();

    for raw in raw_waivers {
        match parse_single_terminal_evidence_waiver(raw) {
            Ok((issue, reason)) => {
                if waiver.is_some() {
                    findings.push(format!(
                        "duplicate terminal-evidence waiver supplied for issue #{issue}"
                    ));
                } else {
                    waiver = Some(LaunchReadinessTerminalEvidenceWaiver {
                        issue,
                        reason,
                        applied: false,
                    });
                }
            }
            Err(error) => findings.push(error),
        }
    }

    if let Some(waiver) = &waiver {
        match terminal_evidence_issue {
            Some(issue) if issue == waiver.issue => {}
            Some(issue) => findings.push(format!(
                "terminal-evidence waiver supplied for issue #{}, but terminal evidence issue is #{issue}",
                waiver.issue
            )),
            None => findings.push(
                "terminal-evidence waiver supplied but --terminal-evidence-issue was not supplied"
                    .to_string(),
            ),
        }
    }

    (waiver, findings)
}

fn parse_single_terminal_evidence_waiver(raw: &str) -> Result<(u64, String), String> {
    let Some((issue, reason)) = raw.split_once('=').or_else(|| raw.split_once(':')) else {
        return Err(
            "terminal-evidence waiver must use ISSUE=reason or ISSUE:reason with a public reason"
                .to_string(),
        );
    };
    let issue = issue
        .trim()
        .trim_start_matches('#')
        .parse::<u64>()
        .map_err(|_| "terminal-evidence waiver issue number must be numeric".to_string())?;
    let reason = public_text(reason, 240);
    if reason.is_empty() {
        return Err(format!(
            "terminal-evidence waiver for issue #{issue} needs a public reason"
        ));
    }
    Ok((issue, reason))
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
