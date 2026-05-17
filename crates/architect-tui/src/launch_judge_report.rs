use std::path::Path;

use serde::Serialize;

use crate::governance_audit_report::{GovernanceAuditReport, GovernanceAuditStatus};
use crate::smoke_types::{SmokeReport, SmokeStatus};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchJudgeResult {
    Go,
    ConditionalGo,
    NoGo,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchJudgeCheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgeCheck {
    pub name: String,
    pub status: LaunchJudgeCheckStatus,
    pub detail: String,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgeCommandEvidence {
    pub command: String,
    pub attempted: bool,
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub stdout_tail: Vec<String>,
    pub stderr_tail: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgeReport {
    pub schema_version: u8,
    pub result: LaunchJudgeResult,
    pub workspace: String,
    pub checks: Vec<LaunchJudgeCheck>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_actions: Vec<String>,
    pub governance_audit: GovernanceAuditReport,
    pub smoke: Option<SmokeReport>,
    pub release_check: LaunchJudgeCommandEvidence,
}

pub(crate) fn build_report(
    workspace: &Path,
    governance_audit: GovernanceAuditReport,
    smoke: Option<SmokeReport>,
    release_check: LaunchJudgeCommandEvidence,
    git_clean: LaunchJudgeCheck,
) -> LaunchJudgeReport {
    let mut checks = vec![
        governance_check(&governance_audit),
        smoke_check(smoke.as_ref()),
        release_check_status(&release_check),
        git_clean,
        manual_terminal_check(),
    ];
    checks.sort_by(|left, right| left.name.cmp(&right.name));

    let blockers: Vec<String> = checks
        .iter()
        .filter(|check| check.status == LaunchJudgeCheckStatus::Failed)
        .map(check_summary)
        .collect();
    let warnings: Vec<String> = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                LaunchJudgeCheckStatus::Warning | LaunchJudgeCheckStatus::Skipped
            )
        })
        .map(check_summary)
        .collect();
    let next_actions: Vec<String> = checks
        .iter()
        .filter_map(|check| check.next_action.clone())
        .collect();
    let result = judge_result(&blockers, &warnings);

    LaunchJudgeReport {
        schema_version: 1,
        result,
        workspace: workspace.display().to_string(),
        checks,
        blockers,
        warnings,
        next_actions,
        governance_audit,
        smoke,
        release_check,
    }
}

fn governance_check(report: &GovernanceAuditReport) -> LaunchJudgeCheck {
    match report.status {
        GovernanceAuditStatus::Passed => check(
            "governance audit",
            LaunchJudgeCheckStatus::Passed,
            "read-only governance audit passed",
            None,
        ),
        GovernanceAuditStatus::PassedWithWarnings => check(
            "governance audit",
            LaunchJudgeCheckStatus::Warning,
            "read-only governance audit passed with warnings",
            Some("review governance-audit warnings before release claims"),
        ),
        GovernanceAuditStatus::Failed => check(
            "governance audit",
            LaunchJudgeCheckStatus::Failed,
            "read-only governance audit failed",
            Some("fix governance-audit errors before launch"),
        ),
    }
}

fn smoke_check(report: Option<&SmokeReport>) -> LaunchJudgeCheck {
    let Some(report) = report else {
        return check(
            "terminal smoke",
            LaunchJudgeCheckStatus::Skipped,
            "terminal smoke was skipped",
            Some("run architect-mcp-tui launch-judge without --skip-smoke"),
        );
    };
    match report.status {
        SmokeStatus::Passed => check(
            "terminal smoke",
            LaunchJudgeCheckStatus::Passed,
            "terminal smoke passed",
            None,
        ),
        SmokeStatus::PassedWithWarnings => check(
            "terminal smoke",
            LaunchJudgeCheckStatus::Warning,
            "terminal smoke passed with adapter availability warnings",
            Some("review adapter readiness warnings before launch claims"),
        ),
        SmokeStatus::Failed => check(
            "terminal smoke",
            LaunchJudgeCheckStatus::Failed,
            "terminal smoke failed",
            Some("fix terminal smoke failure before launch"),
        ),
    }
}

pub(crate) fn release_check_status(evidence: &LaunchJudgeCommandEvidence) -> LaunchJudgeCheck {
    if !evidence.attempted {
        return check(
            "release gate",
            LaunchJudgeCheckStatus::Skipped,
            "npm run release:check was not run by launch-judge",
            Some("rerun with --run-release-check before release-sensitive go"),
        );
    }
    if evidence.ok {
        check(
            "release gate",
            LaunchJudgeCheckStatus::Passed,
            "npm run release:check passed",
            None,
        )
    } else {
        check(
            "release gate",
            LaunchJudgeCheckStatus::Failed,
            "npm run release:check failed",
            Some("fix release gate failures before launch"),
        )
    }
}

fn manual_terminal_check() -> LaunchJudgeCheck {
    check(
        "external terminal evidence",
        LaunchJudgeCheckStatus::Warning,
        "manual Windows and Linux terminal evidence is not proven by this local command",
        Some(
            "collect or link public-safe Windows and Linux terminal QA evidence before final launch go",
        ),
    )
}

pub(crate) fn check(
    name: &str,
    status: LaunchJudgeCheckStatus,
    detail: &str,
    next_action: Option<&str>,
) -> LaunchJudgeCheck {
    LaunchJudgeCheck {
        name: name.to_string(),
        status,
        detail: detail.to_string(),
        next_action: next_action.map(ToString::to_string),
    }
}

pub(crate) fn judge_result(blockers: &[String], warnings: &[String]) -> LaunchJudgeResult {
    if !blockers.is_empty() {
        LaunchJudgeResult::NoGo
    } else if !warnings.is_empty() {
        LaunchJudgeResult::ConditionalGo
    } else {
        LaunchJudgeResult::Go
    }
}

pub(crate) fn skipped_command(command: &str) -> LaunchJudgeCommandEvidence {
    LaunchJudgeCommandEvidence {
        command: command.to_string(),
        attempted: false,
        ok: false,
        exit_code: None,
        stdout_tail: Vec::new(),
        stderr_tail: Vec::new(),
        error: None,
    }
}

pub(crate) fn tail_lines(text: &str, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
    if lines.len() > max {
        lines = lines.split_off(lines.len() - max);
    }
    lines
}

pub(crate) fn print_text_report(report: &LaunchJudgeReport) {
    println!("architect-mcp-tui launch judge: {:?}", report.result);
    println!("workspace: {}", report.workspace);
    for check in &report.checks {
        println!("- {}: {:?} - {}", check.name, check.status, check.detail);
    }
    if !report.blockers.is_empty() {
        println!("blockers:");
        for blocker in &report.blockers {
            println!("- {blocker}");
        }
    }
    if !report.next_actions.is_empty() {
        println!("next actions:");
        for action in &report.next_actions {
            println!("- {action}");
        }
    }
}

fn check_summary(check: &LaunchJudgeCheck) -> String {
    format!("{}: {}", check.name, check.detail)
}
