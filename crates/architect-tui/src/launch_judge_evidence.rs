use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::launch_judge_evidence_safety::collect_unsafe_content;
use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeTerminalEvidenceReport,
    LaunchJudgeTerminalEvidenceStatus, LaunchJudgeTerminalEvidenceSummary, check,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TerminalEvidenceEnvelope {
    pub(crate) schema_version: Option<u8>,
    pub(crate) reports: Vec<LaunchJudgeTerminalEvidenceReport>,
}

pub(crate) fn read_terminal_evidence(
    paths: &[PathBuf],
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    if paths.is_empty() {
        let summary = LaunchJudgeTerminalEvidenceSummary {
            supplied: false,
            source_path: None,
            reports: Vec::new(),
            issues: vec!["manual Windows and Linux terminal evidence was not supplied".to_string()],
        };
        return (
            summary,
            check(
                "external terminal evidence",
                LaunchJudgeCheckStatus::Warning,
                "manual Windows and Linux terminal evidence is not proven by this local command",
                Some(
                    "collect or link public-safe Windows and Linux terminal QA evidence before final launch go",
                ),
            ),
        );
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
                if let Some(version) = envelope.schema_version
                    && version != 1
                {
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

    build_terminal_evidence_summary(Some(source_path), reports, issues, hard_failure)
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
    mut reports: Vec<LaunchJudgeTerminalEvidenceReport>,
    mut issues: Vec<String>,
    hard_failure: bool,
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    let supplied = !reports.is_empty() || source_path.is_some();
    let mut summary = LaunchJudgeTerminalEvidenceSummary {
        supplied,
        source_path,
        reports: Vec::new(),
        issues: Vec::new(),
    };

    normalize_and_validate_reports(&mut reports, &mut issues);
    summary.reports = reports;
    summary.issues = issues;

    if hard_failure {
        return failed(summary, "terminal evidence file could not be validated");
    }
    if summary.issues.iter().any(|issue| issue.contains("failed")) {
        return failed(summary, "terminal evidence includes failed platform QA");
    }
    if !summary.issues.is_empty() {
        return (
            summary,
            check(
                "external terminal evidence",
                LaunchJudgeCheckStatus::Warning,
                "terminal evidence is incomplete or has warnings",
                Some(
                    "complete clean public-safe Linux and Windows terminal QA evidence before final launch go",
                ),
            ),
        );
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

fn public_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("provided terminal evidence")
        .to_string()
}

fn normalize_and_validate_reports(
    reports: &mut [LaunchJudgeTerminalEvidenceReport],
    issues: &mut Vec<String>,
) {
    if reports.is_empty() {
        issues.push("terminal evidence reports must include linux and windows".to_string());
        return;
    }

    let mut seen = HashSet::new();
    for report in reports.iter_mut() {
        report.platform = report.platform.trim().to_ascii_lowercase();
        report.source = report.source.trim().to_string();
        report.command_summary = report.command_summary.trim().to_string();
        report.collected_at = report
            .collected_at
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        report.notes = report
            .notes
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        if !matches!(report.platform.as_str(), "linux" | "windows") {
            issues.push(format!(
                "terminal evidence platform '{}' must be linux or windows",
                report.platform
            ));
            continue;
        }
        if !seen.insert(report.platform.clone()) {
            issues.push(format!(
                "terminal evidence platform '{}' is duplicated",
                report.platform
            ));
        }
        if report.source.is_empty() {
            issues.push(format!(
                "terminal evidence for '{}' needs a public-safe source",
                report.platform
            ));
        }
        if report.command_summary.is_empty() {
            issues.push(format!(
                "terminal evidence for '{}' needs a command summary",
                report.platform
            ));
        }
        if uses_template_placeholder(report) {
            issues.push(format!(
                "terminal evidence for '{}' appears to use a template placeholder; replace it with real platform-specific QA evidence",
                report.platform
            ));
        }
        match report.status {
            LaunchJudgeTerminalEvidenceStatus::Passed => {}
            LaunchJudgeTerminalEvidenceStatus::PassedWithWarnings => issues.push(format!(
                "terminal evidence for '{}' passed with warnings",
                report.platform
            )),
            LaunchJudgeTerminalEvidenceStatus::Failed => issues.push(format!(
                "terminal evidence for '{}' failed",
                report.platform
            )),
        }
    }

    for required in ["linux", "windows"] {
        if !seen.contains(required) {
            issues.push(format!(
                "terminal evidence for '{required}' was not supplied"
            ));
        }
    }
}

fn uses_template_placeholder(report: &LaunchJudgeTerminalEvidenceReport) -> bool {
    [
        report.source.as_str(),
        report.command_summary.as_str(),
        report.notes.as_deref().unwrap_or_default(),
    ]
    .iter()
    .any(|value| looks_like_template_placeholder(value))
}

fn looks_like_template_placeholder(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.contains("replace with")
        || normalized == "issue #136 public-safe terminal qa report"
        || normalized
            == "architect-mcp-tui terminal-evidence --json passed; help, adapter summary, and gate-only run were summarized"
        || normalized == "summary only, no raw logs"
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
