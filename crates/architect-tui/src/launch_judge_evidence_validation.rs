use std::collections::HashSet;

use crate::launch_judge_report::{
    LaunchJudgeTerminalEvidenceReport, LaunchJudgeTerminalEvidenceStatus,
};
use crate::terminal_evidence_environment::validate_report_environment;

pub(crate) fn normalize_and_validate_reports(
    reports: &mut [LaunchJudgeTerminalEvidenceReport],
    issues: &mut Vec<String>,
) -> bool {
    let mut failed = false;
    if reports.is_empty() {
        issues.push("terminal evidence reports must include linux and windows".to_string());
        return failed;
    }

    let mut seen = HashSet::new();
    for report in reports.iter_mut() {
        normalize_report(report);

        if !matches!(report.platform.as_str(), "linux" | "windows") {
            failed = true;
            issues.push(format!(
                "terminal evidence platform '{}' must be linux or windows",
                report.platform
            ));
            continue;
        }
        if !seen.insert(report.platform.clone()) {
            failed = true;
            issues.push(format!(
                "terminal evidence platform '{}' is duplicated",
                report.platform
            ));
        }
        validate_report_fields(report, issues);
    }

    for required in ["linux", "windows"] {
        if !seen.contains(required) {
            issues.push(format!(
                "terminal evidence for '{required}' was not supplied"
            ));
        }
    }

    failed
}

fn normalize_report(report: &mut LaunchJudgeTerminalEvidenceReport) {
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
}

fn validate_report_fields(report: &LaunchJudgeTerminalEvidenceReport, issues: &mut Vec<String>) {
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
    validate_report_environment(report, issues);
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
}
