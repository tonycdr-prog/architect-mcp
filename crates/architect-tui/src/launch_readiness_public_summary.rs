use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness::{LaunchReadinessReport, LaunchReadinessTerminalEvidenceWaiver};
use crate::launch_stack::{LaunchStackItemStatus, LaunchStackReport};
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicSummary {
    pub schema_version: u8,
    pub generated_at_unix_seconds: u64,
    pub result: LaunchJudgeResult,
    pub repository: Option<String>,
    pub read_only: bool,
    pub launch_stack: LaunchReadinessPublicStack,
    pub terminal_evidence: LaunchReadinessPublicTerminalEvidence,
    pub terminal_evidence_waiver: Option<LaunchReadinessPublicTerminalEvidenceWaiver>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicStack {
    pub result: LaunchJudgeResult,
    pub from_pr: Option<u64>,
    pub stopped_at_base: Option<String>,
    pub pull_request_count: usize,
    pub pull_request_order: Vec<u64>,
    pub pull_request_status: LaunchReadinessPublicStatusCounts,
    pub missing_required_check_count: usize,
    pub missing_required_checks: Vec<LaunchReadinessPublicMissingRequiredCheck>,
    pub blocker_issues: Vec<LaunchReadinessPublicBlockerIssue>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicStatusCounts {
    pub passed: usize,
    pub waived: usize,
    pub warning: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicBlockerIssue {
    pub number: u64,
    pub state: String,
    pub status: LaunchStackItemStatus,
    pub waived: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicMissingRequiredCheck {
    pub pull_request: u64,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicTerminalEvidence {
    pub issue: Option<LaunchReadinessPublicTerminalEvidenceIssue>,
    pub report_count: usize,
    pub platforms: Vec<String>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicTerminalEvidenceIssue {
    pub number: u64,
    pub result: LaunchJudgeResult,
    pub extracted_block_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadinessPublicTerminalEvidenceWaiver {
    pub issue: u64,
    pub applied: bool,
    pub reason: String,
}

pub(crate) fn build_public_summary(report: &LaunchReadinessReport) -> LaunchReadinessPublicSummary {
    build_public_summary_at(report, generated_at_unix_seconds())
}

pub(crate) fn build_public_summary_at(
    report: &LaunchReadinessReport,
    generated_at_unix_seconds: u64,
) -> LaunchReadinessPublicSummary {
    LaunchReadinessPublicSummary {
        schema_version: 1,
        generated_at_unix_seconds,
        result: report.result.clone(),
        repository: report
            .repository
            .as_deref()
            .map(|repository| public_text(repository, 160)),
        read_only: report.read_only,
        launch_stack: public_stack(&report.launch_stack),
        terminal_evidence: public_terminal_evidence(report),
        terminal_evidence_waiver: report
            .terminal_evidence_waiver
            .as_ref()
            .map(public_terminal_evidence_waiver),
        findings: public_strings(&report.findings, 320),
        next_actions: public_strings(&report.next_actions, 320),
    }
}

fn generated_at_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn public_stack(stack: &LaunchStackReport) -> LaunchReadinessPublicStack {
    let missing_required_checks = missing_required_checks(stack);
    let missing_required_check_count = missing_required_checks
        .iter()
        .map(|entry| entry.names.len())
        .sum();
    LaunchReadinessPublicStack {
        result: stack.result.clone(),
        from_pr: stack
            .stack_discovery
            .as_ref()
            .map(|discovery| discovery.from_pr),
        stopped_at_base: stack
            .stack_discovery
            .as_ref()
            .map(|discovery| public_text(&discovery.stopped_at_base, 120)),
        pull_request_count: stack.pull_requests.len(),
        pull_request_order: stack
            .stack_discovery
            .as_ref()
            .map(|discovery| discovery.pull_requests.clone())
            .unwrap_or_else(|| stack.pull_requests.iter().map(|pr| pr.number).collect()),
        pull_request_status: status_counts(stack.pull_requests.iter().map(|pr| &pr.status)),
        missing_required_check_count,
        missing_required_checks,
        blocker_issues: stack
            .blocker_issues
            .iter()
            .map(|issue| LaunchReadinessPublicBlockerIssue {
                number: issue.number,
                state: public_text(&issue.state, 80),
                status: issue.status.clone(),
                waived: issue.status == LaunchStackItemStatus::Waived,
            })
            .collect(),
    }
}

fn missing_required_checks(
    stack: &LaunchStackReport,
) -> Vec<LaunchReadinessPublicMissingRequiredCheck> {
    stack
        .pull_requests
        .iter()
        .filter(|pr| !pr.checks.missing_required_names.is_empty())
        .map(|pr| LaunchReadinessPublicMissingRequiredCheck {
            pull_request: pr.number,
            names: public_strings(&pr.checks.missing_required_names, 120),
        })
        .collect()
}

fn status_counts<'a>(
    statuses: impl Iterator<Item = &'a LaunchStackItemStatus>,
) -> LaunchReadinessPublicStatusCounts {
    let mut counts = LaunchReadinessPublicStatusCounts::default();
    for status in statuses {
        match status {
            LaunchStackItemStatus::Passed => counts.passed += 1,
            LaunchStackItemStatus::Waived => counts.waived += 1,
            LaunchStackItemStatus::Warning => counts.warning += 1,
            LaunchStackItemStatus::Failed => counts.failed += 1,
        }
    }
    counts
}

fn public_terminal_evidence(
    report: &LaunchReadinessReport,
) -> LaunchReadinessPublicTerminalEvidence {
    let Some(evidence) = &report.terminal_evidence_issue else {
        return LaunchReadinessPublicTerminalEvidence {
            issue: None,
            report_count: 0,
            platforms: Vec::new(),
            issues: Vec::new(),
        };
    };

    LaunchReadinessPublicTerminalEvidence {
        issue: Some(LaunchReadinessPublicTerminalEvidenceIssue {
            number: evidence.issue.number,
            result: evidence.result.clone(),
            extracted_block_count: evidence.extracted_blocks.len(),
        }),
        report_count: evidence.terminal_evidence.reports.len(),
        platforms: evidence
            .terminal_evidence
            .reports
            .iter()
            .map(|evidence| public_text(&evidence.platform, 80))
            .collect(),
        issues: public_strings(&evidence.terminal_evidence.issues, 320),
    }
}

fn public_terminal_evidence_waiver(
    waiver: &LaunchReadinessTerminalEvidenceWaiver,
) -> LaunchReadinessPublicTerminalEvidenceWaiver {
    LaunchReadinessPublicTerminalEvidenceWaiver {
        issue: waiver.issue,
        applied: waiver.applied,
        reason: public_text(&waiver.reason, 240),
    }
}

fn public_strings(values: &[String], max_len: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| public_text(value, max_len))
        .collect()
}
