use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeTerminalEvidenceReport,
    LaunchJudgeTerminalEvidenceStatus, LaunchJudgeTerminalEvidenceSummary, check,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalEvidenceEnvelope {
    schema_version: Option<u8>,
    reports: Vec<LaunchJudgeTerminalEvidenceReport>,
}

pub(crate) fn read_terminal_evidence(
    path: Option<&Path>,
) -> (LaunchJudgeTerminalEvidenceSummary, LaunchJudgeCheck) {
    let Some(path) = path else {
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
    };

    let source_path = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("provided terminal evidence")
        .to_string();
    let mut summary = LaunchJudgeTerminalEvidenceSummary {
        supplied: true,
        source_path: Some(source_path),
        reports: Vec::new(),
        issues: Vec::new(),
    };

    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            summary
                .issues
                .push(format!("terminal evidence file could not be read: {error}"));
            return failed(summary, "terminal evidence file could not be read");
        }
    };

    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            summary.issues.push(format!(
                "terminal evidence JSON could not be parsed: {error}"
            ));
            return failed(summary, "terminal evidence JSON could not be parsed");
        }
    };

    collect_unsafe_content(&value, "$", &mut summary.issues);
    if !summary.issues.is_empty() {
        return failed(
            summary,
            "terminal evidence contains unsafe public-sharing content",
        );
    }

    let envelope: TerminalEvidenceEnvelope = match serde_json::from_value(value) {
        Ok(input) => input,
        Err(error) => {
            summary
                .issues
                .push(format!("terminal evidence schema is invalid: {error}"));
            return failed(summary, "terminal evidence schema is invalid");
        }
    };

    if let Some(version) = envelope.schema_version
        && version != 1
    {
        summary
            .issues
            .push("terminal evidence schemaVersion must be 1".to_string());
    }

    let mut reports = envelope.reports;
    normalize_and_validate_reports(&mut reports, &mut summary.issues);
    summary.reports = reports;

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

fn collect_unsafe_content(value: &Value, path: &str, issues: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let child_path = format!("{path}.{key}");
                if risky_key(key) {
                    issues.push(format!(
                        "terminal evidence contains raw or private field at {child_path}"
                    ));
                }
                collect_unsafe_content(value, &child_path, issues);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_unsafe_content(value, &format!("{path}[{index}]"), issues);
            }
        }
        Value::String(text) => {
            if text.len() > 2_000 {
                issues.push(format!(
                    "terminal evidence string at {path} is too long for public-safe summary"
                ));
            }
            if text.lines().count() > 8 {
                issues.push(format!(
                    "terminal evidence string at {path} looks like raw multiline output"
                ));
            }
            if contains_sensitive_text(text) {
                issues.push(format!(
                    "terminal evidence string at {path} contains secret-shaped or local-path content"
                ));
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn risky_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "stdout"
            | "stderr"
            | "raw"
            | "log"
            | "logs"
            | "rawlog"
            | "rawlogs"
            | "localpath"
            | "privatepath"
            | "privaterepo"
    ) || key.contains("stdout")
        || key.contains("stderr")
        || key.contains("raw_log")
        || key.contains("rawlog")
        || key.contains("local_path")
        || key.contains("localpath")
        || key.contains("private_repo")
        || key.contains("privaterepo")
}

fn contains_sensitive_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains("/Users/")
        || text.contains("C:\\Users\\")
        || text.contains("C:/Users/")
        || text.contains("BEGIN PRIVATE KEY")
        || lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
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
