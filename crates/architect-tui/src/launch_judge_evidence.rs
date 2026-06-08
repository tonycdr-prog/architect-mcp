use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::launch_judge_evidence_safety::collect_unsafe_content;
use crate::launch_judge_evidence_validation::normalize_and_validate_reports;
use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeTerminalEvidenceReport,
    LaunchJudgeTerminalEvidenceSummary, check,
};
use crate::launch_scope::LaunchScope;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TerminalEvidenceEnvelope {
    pub(crate) schema_version: u8,
    pub(crate) reports: Vec<LaunchJudgeTerminalEvidenceReport>,
}

#[cfg(test)]
pub(crate) fn read_terminal_evidence(
    paths: &[PathBuf],
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    read_terminal_evidence_for_scope(paths, LaunchScope::PublicCli)
}

pub(crate) fn read_terminal_evidence_for_scope(
    paths: &[PathBuf],
    scope: LaunchScope,
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    if paths.is_empty() {
        let summary = LaunchJudgeTerminalEvidenceSummary {
            supplied: false,
            source_path: None,
            reports: Vec::new(),
            issues: vec!["manual Windows and Linux terminal evidence was not supplied".to_string()],
        };
        return (summary, missing_terminal_evidence_check(scope));
    }

    let source_path = paths
        .iter()
        .map(|path| public_file_name(path))
        .collect::<Vec<_>>()
        .join(", ");
    let mut reports = Vec::new();
    let mut hard_failure = false;
    let mut issues = Vec::new();
    for path in paths {
        let file_name = public_file_name(path);
        match read_terminal_evidence_file(path, &file_name) {
            Ok(mut envelope) => {
                if envelope.schema_version != 1 {
                    hard_failure = true;
                    issues.push(format!(
                        "{file_name}: terminal evidence schemaVersion must be 1"
                    ));
                }
                reports.append(&mut envelope.reports);
            }
            Err(mut file_issues) => {
                hard_failure = true;
                issues.append(&mut file_issues);
            }
        }
    }

    build_terminal_evidence_summary_for_scope(
        Some(source_path),
        reports,
        issues,
        hard_failure,
        scope,
    )
}

fn read_terminal_evidence_file(
    path: &Path,
    file_name: &str,
) -> Result<TerminalEvidenceEnvelope, Vec<String>> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        vec![format!(
            "{file_name}: terminal evidence file could not be read: {error}"
        )]
    })?;

    let value: Value = serde_json::from_str(&text).map_err(|error| {
        vec![format!(
            "{file_name}: terminal evidence JSON could not be parsed: {error}"
        )]
    })?;

    parse_terminal_evidence_value(value, file_name)
}

pub(crate) fn parse_terminal_evidence_value(
    value: Value,
    source_name: &str,
) -> Result<TerminalEvidenceEnvelope, Vec<String>> {
    let mut issues = Vec::new();
    collect_unsafe_content(&value, "$", &mut issues);
    if !issues.is_empty() {
        return Err(issues
            .into_iter()
            .map(|issue| format!("{source_name}: {issue}"))
            .collect());
    }

    serde_json::from_value(value).map_err(|error| {
        vec![format!(
            "{source_name}: terminal evidence schema is invalid: {error}"
        )]
    })
}

pub(crate) fn build_terminal_evidence_summary(
    source_path: Option<String>,
    reports: Vec<LaunchJudgeTerminalEvidenceReport>,
    issues: Vec<String>,
    hard_failure: bool,
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    build_terminal_evidence_summary_for_scope(
        source_path,
        reports,
        issues,
        hard_failure,
        LaunchScope::PublicCli,
    )
}

pub(crate) fn build_terminal_evidence_summary_for_scope(
    source_path: Option<String>,
    mut reports: Vec<LaunchJudgeTerminalEvidenceReport>,
    mut issues: Vec<String>,
    hard_failure: bool,
    scope: LaunchScope,
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    let supplied = !reports.is_empty() || source_path.is_some();
    let mut summary = LaunchJudgeTerminalEvidenceSummary {
        supplied,
        source_path,
        reports: Vec::new(),
        issues: Vec::new(),
    };

    let report_validation_failed = normalize_and_validate_reports(&mut reports, &mut issues);
    summary.reports = reports;
    summary.issues = issues;

    if hard_failure {
        return failed(summary, "terminal evidence file could not be validated");
    }
    if report_validation_failed {
        return failed(summary, "terminal evidence platform data is invalid");
    }
    if summary.issues.iter().any(|issue| issue.contains("failed")) {
        return failed(summary, "terminal evidence includes failed platform QA");
    }
    if !summary.issues.is_empty() {
        return (summary, incomplete_terminal_evidence_check(scope));
    }

    (
        summary,
        check(
            "external terminal evidence",
            LaunchJudgeCheckStatus::Passed,
            "public-safe Linux and Windows terminal evidence was supplied",
            None,
        ),
    )
}

fn missing_terminal_evidence_check(scope: LaunchScope) -> LaunchJudgeCheck {
    if scope.treats_missing_terminal_evidence_as_future() {
        return check(
            "external terminal evidence",
            LaunchJudgeCheckStatus::Info,
            &format!(
                "manual Windows and Linux terminal evidence is future public-cli evidence for {} scope",
                scope.label()
            ),
            Some(
                "collect or link public-safe Windows and Linux terminal QA evidence before public CLI launch go",
            ),
        );
    }

    check(
        "external terminal evidence",
        LaunchJudgeCheckStatus::Warning,
        "manual Windows and Linux terminal evidence is not proven by this local command",
        Some(
            "collect or link public-safe Windows and Linux terminal QA evidence before final launch go",
        ),
    )
}

fn incomplete_terminal_evidence_check(scope: LaunchScope) -> LaunchJudgeCheck {
    if scope.treats_missing_terminal_evidence_as_future() {
        return check(
            "external terminal evidence",
            LaunchJudgeCheckStatus::Info,
            &format!(
                "terminal evidence is incomplete future public-cli evidence for {} scope",
                scope.label()
            ),
            Some(
                "complete clean public-safe Linux and Windows terminal QA evidence before public CLI launch go",
            ),
        );
    }

    check(
        "external terminal evidence",
        LaunchJudgeCheckStatus::Warning,
        "terminal evidence is incomplete or has warnings",
        Some(
            "complete clean public-safe Linux and Windows terminal QA evidence before final launch go",
        ),
    )
}

fn public_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("provided terminal evidence")
        .to_string()
}

fn failed(
    summary: LaunchJudgeTerminalEvidenceSummary,
    detail: &str,
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    (
        summary,
        check(
            "external terminal evidence",
            LaunchJudgeCheckStatus::Failed,
            detail,
            Some("replace terminal evidence with a public-safe linux and windows summary"),
        ),
    )
}
