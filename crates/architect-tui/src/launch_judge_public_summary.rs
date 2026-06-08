use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::launch_judge_future::FutureLaunchReadiness;
use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeCommandEvidence, LaunchJudgeReport,
    LaunchJudgeResult, LaunchJudgeTerminalEvidenceEnvironment, LaunchJudgeTerminalEvidenceStatus,
    LaunchJudgeTerminalEvidenceSummary,
};
use crate::launch_scope::LaunchScopeSummary;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgePublicSummary {
    pub schema_version: u8,
    pub generated_at_unix_seconds: u64,
    pub scope: LaunchScopeSummary,
    pub result: LaunchJudgeResult,
    pub checks: Vec<LaunchJudgePublicCheck>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_actions: Vec<String>,
    pub release_check: LaunchJudgePublicReleaseCheck,
    pub terminal_evidence: LaunchJudgePublicTerminalEvidence,
    pub future_launch_readiness: LaunchJudgePublicFutureLaunchReadiness,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgePublicCheck {
    pub name: String,
    pub status: LaunchJudgeCheckStatus,
    pub detail: String,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgePublicReleaseCheck {
    pub command: String,
    pub attempted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgePublicTerminalEvidence {
    pub supplied: bool,
    pub source_path: Option<String>,
    pub reports: Vec<LaunchJudgePublicTerminalEvidenceReport>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgePublicTerminalEvidenceReport {
    pub platform: String,
    pub status: LaunchJudgeTerminalEvidenceStatus,
    pub environment: Option<LaunchJudgeTerminalEvidenceEnvironment>,
    pub collected_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchJudgePublicFutureLaunchReadiness {
    pub public_cli: LaunchJudgeResult,
    pub missing_evidence: Vec<String>,
}

pub(crate) fn build_public_summary(report: &LaunchJudgeReport) -> LaunchJudgePublicSummary {
    build_public_summary_at(report, generated_at_unix_seconds())
}

pub(crate) fn build_public_summary_at(
    report: &LaunchJudgeReport,
    generated_at_unix_seconds: u64,
) -> LaunchJudgePublicSummary {
    LaunchJudgePublicSummary {
        schema_version: 1,
        generated_at_unix_seconds,
        scope: public_scope(&report.scope),
        result: report.result.clone(),
        checks: report.checks.iter().map(public_check).collect(),
        blockers: public_strings(&report.blockers, 320),
        warnings: public_strings(&report.warnings, 320),
        next_actions: public_strings(&report.next_actions, 320),
        release_check: public_release_check(&report.release_check),
        terminal_evidence: public_terminal_evidence(&report.terminal_evidence),
        future_launch_readiness: public_future_launch_readiness(&report.future_launch_readiness),
    }
}

fn generated_at_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn public_check(check: &LaunchJudgeCheck) -> LaunchJudgePublicCheck {
    LaunchJudgePublicCheck {
        name: public_text(&check.name, 120),
        status: check.status.clone(),
        detail: public_text(&check.detail, 320),
        next_action: check
            .next_action
            .as_deref()
            .map(|action| public_text(action, 320)),
    }
}

fn public_release_check(evidence: &LaunchJudgeCommandEvidence) -> LaunchJudgePublicReleaseCheck {
    LaunchJudgePublicReleaseCheck {
        command: public_text(&evidence.command, 120),
        attempted: evidence.attempted,
        ok: evidence.ok,
        exit_code: evidence.exit_code,
    }
}

fn public_scope(scope: &LaunchScopeSummary) -> LaunchScopeSummary {
    LaunchScopeSummary {
        name: scope.name,
        resolution: scope.resolution,
        signals: public_strings(&scope.signals, 160),
    }
}

fn public_terminal_evidence(
    evidence: &LaunchJudgeTerminalEvidenceSummary,
) -> LaunchJudgePublicTerminalEvidence {
    LaunchJudgePublicTerminalEvidence {
        supplied: evidence.supplied,
        source_path: evidence
            .source_path
            .as_deref()
            .map(|source| public_text(source, 240)),
        reports: evidence
            .reports
            .iter()
            .map(|report| LaunchJudgePublicTerminalEvidenceReport {
                platform: public_text(&report.platform, 80),
                status: report.status.clone(),
                environment: report.environment.clone(),
                collected_at: report
                    .collected_at
                    .as_deref()
                    .map(|collected_at| public_text(collected_at, 80)),
            })
            .collect(),
        issues: public_strings(&evidence.issues, 320),
    }
}

fn public_future_launch_readiness(
    readiness: &FutureLaunchReadiness,
) -> LaunchJudgePublicFutureLaunchReadiness {
    LaunchJudgePublicFutureLaunchReadiness {
        public_cli: readiness.public_cli.clone(),
        missing_evidence: public_strings(&readiness.missing_evidence, 320),
    }
}

fn public_strings(values: &[String], max_len: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| public_text(value, max_len))
        .collect()
}

fn public_text(value: &str, max_len: usize) -> String {
    let normalized = value
        .replace('\\', "/")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut redacted = normalized
        .split_whitespace()
        .map(redact_token)
        .collect::<Vec<_>>()
        .join(" ");
    if redacted.chars().count() > max_len {
        redacted = redacted.chars().take(max_len).collect::<String>();
        redacted.push_str(" [truncated]");
    }
    redacted
}

fn redact_token(token: &str) -> String {
    let trimmed = token.trim_matches(|ch: char| {
        matches!(
            ch,
            ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
        )
    });
    let lower = trimmed.to_ascii_lowercase();
    if is_secret_like(&lower) {
        "[redacted-secret]".to_string()
    } else if is_local_path_like(&lower) {
        "[redacted-local-path]".to_string()
    } else {
        token.to_string()
    }
}

fn is_secret_like(lower: &str) -> bool {
    lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
        || lower.contains("begin") && lower.contains("private") && lower.contains("key")
}

fn is_local_path_like(lower: &str) -> bool {
    lower.starts_with("/")
        || lower.contains("/users/")
        || lower.contains("/home/")
        || (lower.len() >= 3 && lower.as_bytes()[1] == b':' && lower.as_bytes()[2] == b'/')
}
