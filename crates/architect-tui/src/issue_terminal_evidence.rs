use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

pub(crate) use crate::issue_terminal_evidence_source::extract_json_blocks;
#[cfg(test)]
use crate::issue_terminal_evidence_source::issue_content_from_value;
use crate::issue_terminal_evidence_source::{IssueContent, fetch_issue_content, public_text};
use crate::launch_judge_evidence::{
    build_terminal_evidence_summary, parse_terminal_evidence_value,
};
use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeResult, LaunchJudgeTerminalEvidenceSummary,
};
use crate::terminal_evidence::TerminalEvidenceFile;

#[derive(Debug, Clone)]
pub struct IssueTerminalEvidenceOptions {
    pub json: bool,
    pub repo: Option<String>,
    pub issue: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueTerminalEvidenceReport {
    pub schema_version: u8,
    pub result: LaunchJudgeResult,
    pub repository: Option<String>,
    pub issue: IssueTerminalEvidenceIssue,
    pub extracted_blocks: Vec<IssueTerminalEvidenceBlock>,
    pub terminal_evidence: LaunchJudgeTerminalEvidenceSummary,
    pub merged_evidence: Option<TerminalEvidenceFile>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueTerminalEvidenceIssue {
    pub number: u64,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueTerminalEvidenceBlockStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueTerminalEvidenceBlock {
    pub source: String,
    pub status: IssueTerminalEvidenceBlockStatus,
    pub report_count: usize,
    pub issues: Vec<String>,
}

pub fn run_issue_terminal_evidence(
    workspace: &Path,
    options: IssueTerminalEvidenceOptions,
) -> Result<()> {
    let report = build_issue_terminal_evidence_report(workspace, &options);
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }
    if report.result == LaunchJudgeResult::NoGo {
        anyhow::bail!("terminal evidence collection result is no-go");
    }
    Ok(())
}

pub fn build_issue_terminal_evidence_report(
    workspace: &Path,
    options: &IssueTerminalEvidenceOptions,
) -> IssueTerminalEvidenceReport {
    match fetch_issue_content(workspace, options.repo.as_deref(), options.issue) {
        Ok(issue) => build_report_from_issue(options.repo.clone(), issue),
        Err(error) => {
            let check = LaunchJudgeCheck {
                name: "external terminal evidence".to_string(),
                status: LaunchJudgeCheckStatus::Failed,
                detail: "GitHub issue terminal evidence could not be collected".to_string(),
                next_action: Some("inspect GitHub CLI auth and issue access".to_string()),
            };
            build_report_from_summary(
                options.repo.clone(),
                IssueTerminalEvidenceIssue {
                    number: options.issue,
                    title: String::new(),
                    url: String::new(),
                },
                Vec::new(),
                LaunchJudgeTerminalEvidenceSummary {
                    supplied: false,
                    source_path: None,
                    reports: Vec::new(),
                    issues: vec![error],
                },
                check,
            )
        }
    }
}

#[cfg(test)]
pub(crate) fn build_issue_terminal_evidence_report_from_value(
    repository: Option<String>,
    fallback_issue: u64,
    value: &Value,
) -> IssueTerminalEvidenceReport {
    match issue_content_from_value(fallback_issue, value) {
        Ok(issue) => build_report_from_issue(repository, issue),
        Err(error) => {
            let check = LaunchJudgeCheck {
                name: "external terminal evidence".to_string(),
                status: LaunchJudgeCheckStatus::Failed,
                detail: "GitHub issue terminal evidence could not be parsed".to_string(),
                next_action: Some("inspect GitHub issue JSON shape".to_string()),
            };
            build_report_from_summary(
                repository,
                IssueTerminalEvidenceIssue {
                    number: fallback_issue,
                    title: String::new(),
                    url: String::new(),
                },
                Vec::new(),
                LaunchJudgeTerminalEvidenceSummary {
                    supplied: false,
                    source_path: None,
                    reports: Vec::new(),
                    issues: vec![error],
                },
                check,
            )
        }
    }
}

fn build_report_from_issue(
    repository: Option<String>,
    issue: IssueContent,
) -> IssueTerminalEvidenceReport {
    let mut blocks = Vec::new();
    let mut reports = Vec::new();
    let mut findings = Vec::new();
    let mut hard_failure = false;

    for body in issue.bodies() {
        for (index, block) in extract_json_blocks(&body.body).iter().enumerate() {
            let source = format!("{} block {}", body.source, index + 1);
            match serde_json::from_str::<Value>(block) {
                Ok(value) => match parse_terminal_evidence_value(value, &source) {
                    Ok(mut envelope) => {
                        let mut issues = Vec::new();
                        if let Some(version) = envelope.schema_version
                            && version != 1
                        {
                            issues.push(format!(
                                "{source}: terminal evidence schemaVersion must be 1"
                            ));
                        }
                        let report_count = envelope.reports.len();
                        reports.append(&mut envelope.reports);
                        if !issues.is_empty() {
                            findings.extend(issues.clone());
                        }
                        blocks.push(IssueTerminalEvidenceBlock {
                            source,
                            status: IssueTerminalEvidenceBlockStatus::Accepted,
                            report_count,
                            issues,
                        });
                    }
                    Err(issues) => {
                        hard_failure = true;
                        findings.extend(issues.clone());
                        blocks.push(IssueTerminalEvidenceBlock {
                            source,
                            status: IssueTerminalEvidenceBlockStatus::Rejected,
                            report_count: 0,
                            issues,
                        });
                    }
                },
                Err(error) => {
                    hard_failure = true;
                    let issues = vec![format!("{source}: JSON could not be parsed: {error}")];
                    findings.extend(issues.clone());
                    blocks.push(IssueTerminalEvidenceBlock {
                        source,
                        status: IssueTerminalEvidenceBlockStatus::Rejected,
                        report_count: 0,
                        issues,
                    });
                }
            }
        }
    }

    if blocks.is_empty() {
        findings.push(format!(
            "issue #{} does not contain terminal-evidence JSON blocks",
            issue.number()
        ));
    }

    let source_path = if blocks.is_empty() {
        None
    } else {
        Some(format!("issue #{} comments", issue.number()))
    };
    let (summary, check) =
        build_terminal_evidence_summary(source_path, reports, findings.clone(), hard_failure);
    build_report_from_summary(
        repository,
        IssueTerminalEvidenceIssue {
            number: issue.number(),
            title: public_text(issue.title(), 180),
            url: public_text(issue.url(), 240),
        },
        blocks,
        summary,
        check,
    )
}

fn build_report_from_summary(
    repository: Option<String>,
    issue: IssueTerminalEvidenceIssue,
    blocks: Vec<IssueTerminalEvidenceBlock>,
    terminal_evidence: LaunchJudgeTerminalEvidenceSummary,
    check: LaunchJudgeCheck,
) -> IssueTerminalEvidenceReport {
    let result = match check.status {
        LaunchJudgeCheckStatus::Failed => LaunchJudgeResult::NoGo,
        LaunchJudgeCheckStatus::Warning | LaunchJudgeCheckStatus::Skipped => {
            LaunchJudgeResult::ConditionalGo
        }
        LaunchJudgeCheckStatus::Passed => LaunchJudgeResult::Go,
    };
    let mut next_actions = check.next_action.into_iter().collect::<Vec<_>>();
    if blocks.is_empty() {
        next_actions.push(
            "ask Linux and Windows testers to post output from architect-mcp-tui terminal-evidence --markdown".to_string(),
        );
    }
    let merged_evidence = if terminal_evidence.reports.is_empty() {
        None
    } else {
        Some(TerminalEvidenceFile {
            schema_version: 1,
            reports: terminal_evidence.reports.clone(),
        })
    };
    IssueTerminalEvidenceReport {
        schema_version: 1,
        result,
        repository: repository.map(|repo| public_text(&repo, 160)),
        issue,
        extracted_blocks: blocks,
        findings: terminal_evidence.issues.clone(),
        terminal_evidence,
        merged_evidence,
        next_actions,
    }
}

fn print_text_report(report: &IssueTerminalEvidenceReport) {
    println!(
        "architect-mcp-tui issue terminal evidence: {:?}",
        report.result
    );
    println!("- issue #{}: {}", report.issue.number, report.issue.title);
    println!("- extracted blocks: {}", report.extracted_blocks.len());
    println!("- reports: {}", report.terminal_evidence.reports.len());
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
