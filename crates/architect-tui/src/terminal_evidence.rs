use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use crate::config::TuiConfig;
use crate::launch_judge_report::{
    LaunchJudgeTerminalEvidenceReport, LaunchJudgeTerminalEvidenceStatus,
};
use crate::smoke::{SmokeOptions, SmokeReport, SmokeStatus, build_smoke_report};

#[derive(Debug, Clone)]
pub struct TerminalEvidenceOptions {
    pub json: bool,
    pub markdown: bool,
    pub prompt: String,
    pub skip_gate: bool,
    pub platform: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TerminalEvidenceFile {
    pub schema_version: u8,
    pub reports: Vec<LaunchJudgeTerminalEvidenceReport>,
}

pub async fn run_terminal_evidence(
    workspace: PathBuf,
    config: TuiConfig,
    options: TerminalEvidenceOptions,
) -> Result<()> {
    validate_output_mode(&options)?;
    let report = build_terminal_evidence(workspace, config, &options).await?;
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if options.markdown {
        print!("{}", render_markdown(&report)?);
    } else {
        print_text_report(&report);
    }
    Ok(())
}

pub async fn build_terminal_evidence(
    workspace: PathBuf,
    config: TuiConfig,
    options: &TerminalEvidenceOptions,
) -> Result<TerminalEvidenceFile> {
    let smoke = build_smoke_report(
        workspace,
        config,
        SmokeOptions {
            json: true,
            prompt: options.prompt.clone(),
            skip_gate: options.skip_gate,
        },
    )
    .await;
    evidence_from_smoke(&smoke, options)
}

pub(crate) fn evidence_from_smoke(
    smoke: &SmokeReport,
    options: &TerminalEvidenceOptions,
) -> Result<TerminalEvidenceFile> {
    let platform = match &options.platform {
        Some(platform) => normalize_platform(platform)?,
        None => platform_from_os(&smoke.environment.os)?,
    };
    let source = options
        .source
        .as_deref()
        .map(sanitize_note)
        .filter(|source| !source.trim().is_empty())
        .unwrap_or_else(|| "architect-mcp-tui terminal-evidence public-safe summary".to_string());
    let report = LaunchJudgeTerminalEvidenceReport {
        platform,
        status: status_from_smoke(&smoke.status),
        source,
        command_summary: command_summary(smoke),
        collected_at: None,
        notes: Some(notes_summary(smoke, options.notes.as_deref())),
    };
    Ok(TerminalEvidenceFile {
        schema_version: 1,
        reports: vec![report],
    })
}

pub(crate) fn validate_output_mode(options: &TerminalEvidenceOptions) -> Result<()> {
    if options.json && options.markdown {
        anyhow::bail!("choose only one terminal-evidence output mode: --json or --markdown");
    }
    Ok(())
}

pub(crate) fn render_markdown(report: &TerminalEvidenceFile) -> Result<String> {
    let json = serde_json::to_string_pretty(report)?;
    Ok(format!(
        "## architect-mcp TUI terminal evidence\n\nPublic-safe summary only. Keep raw logs local. This command does not create, edit, or close GitHub issues; paste this comment manually into the Terminal QA issue.\n\n```json\n{json}\n```\n"
    ))
}

fn platform_from_os(os: &str) -> Result<String> {
    normalize_platform(os).map_err(|_| {
        anyhow::anyhow!(
            "terminal evidence is launch-relevant only on linux or windows; rerun on one of those platforms or pass --platform when summarizing verified external evidence"
        )
    })
}

fn normalize_platform(platform: &str) -> Result<String> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "linux" => Ok("linux".to_string()),
        "windows" | "win32" | "win" => Ok("windows".to_string()),
        other => {
            anyhow::bail!("unsupported terminal evidence platform '{other}'; use linux or windows")
        }
    }
}

fn status_from_smoke(status: &SmokeStatus) -> LaunchJudgeTerminalEvidenceStatus {
    match status {
        SmokeStatus::Passed => LaunchJudgeTerminalEvidenceStatus::Passed,
        SmokeStatus::PassedWithWarnings => LaunchJudgeTerminalEvidenceStatus::PassedWithWarnings,
        SmokeStatus::Failed => LaunchJudgeTerminalEvidenceStatus::Failed,
    }
}

fn command_summary(smoke: &SmokeReport) -> String {
    let help = pass_fail(smoke.help.ok);
    let gate = if smoke.gate_only.attempted {
        format!(
            "gate-only run --jsonl {} with status {}",
            pass_fail(smoke.gate_only.ok),
            smoke.gate_only.status.as_deref().unwrap_or("unknown")
        )
    } else {
        "gate-only run --jsonl skipped".to_string()
    };
    format!(
        "architect-mcp-tui --help {help}; config adapters --json reported {}/{} ready; {gate}; smoke status {:?}",
        smoke.adapters.ready, smoke.adapters.total, smoke.status
    )
}

fn notes_summary(smoke: &SmokeReport, operator_notes: Option<&str>) -> String {
    let mut parts = vec![
        format!(
            "os={} arch={}",
            smoke.environment.os, smoke.environment.arch
        ),
        format!("binarySource={}", smoke.binary.source),
        format!("tools={}", tool_summary(smoke)),
        "summary only, no raw logs or local paths".to_string(),
    ];
    if let Some(notes) = operator_notes
        .map(str::trim)
        .filter(|notes| !notes.is_empty())
    {
        parts.push(format!("operatorNotes={}", sanitize_note(notes)));
    }
    parts.join("; ")
}

fn tool_summary(smoke: &SmokeReport) -> String {
    ["node", "npm", "rustc", "cargo"]
        .into_iter()
        .map(|name| {
            let value = smoke
                .environment
                .tools
                .get(name)
                .and_then(|check| check.first_line.as_deref())
                .unwrap_or("unavailable");
            format!("{name}={}", sanitize_note(value))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn sanitize_note(note: &str) -> String {
    let first_line = note.lines().next().unwrap_or_default().replace('\\', "/");
    first_line
        .split_whitespace()
        .map(|part| {
            if looks_like_local_path(part) {
                "[redacted-path]"
            } else if looks_like_secret(part) {
                "[redacted-secret]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}

fn looks_like_local_path(part: &str) -> bool {
    let lower = part
        .trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
            )
        })
        .to_ascii_lowercase();
    lower.starts_with("/")
        || lower.contains("/users/")
        || lower.contains("/home/")
        || (lower.len() >= 3 && lower.as_bytes()[1] == b':' && lower.as_bytes()[2] == b'/')
}

fn looks_like_secret(part: &str) -> bool {
    let lower = part
        .trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
            )
        })
        .to_ascii_lowercase();
    lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("gho_")
        || lower.contains("ghu_")
        || lower.contains("ghs_")
        || lower.contains("ghr_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
}

fn pass_fail(ok: bool) -> &'static str {
    if ok { "passed" } else { "failed" }
}

fn print_text_report(report: &TerminalEvidenceFile) {
    println!("architect-mcp-tui terminal evidence");
    for item in &report.reports {
        println!(
            "- {}: {:?} - {}",
            item.platform, item.status, item.command_summary
        );
    }
}
