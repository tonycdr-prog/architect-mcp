use std::collections::BTreeMap;

use crate::adapter::{AdapterHealth, AuthStatus};
use crate::issue_terminal_evidence_source::extract_json_blocks;
use crate::launch_judge_report::{
    LaunchJudgeTerminalEvidenceEnvironment, LaunchJudgeTerminalEvidenceStatus,
};
use crate::mcp::McpProcessSpec;
use crate::smoke::SmokeStatus;
use crate::smoke_types::{
    AdapterSummary, BinarySummary, CommandCheck, EnvironmentSummary, GateSmoke, SmokeReport,
};
use crate::terminal_evidence::{
    TerminalEvidenceOptions, evidence_from_smoke, render_markdown, validate_output_mode,
};
use crate::terminal_evidence_date::unix_days_to_date;

#[test]
fn evidence_from_smoke_outputs_public_safe_summary() {
    let smoke = smoke_report(SmokeStatus::PassedWithWarnings, "linux");
    let evidence = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: true,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            markdown: false,
            skip_gate: false,
            platform: None,
            environment: Some("local-terminal".to_string()),
            source: Some(
                "/home/example/private issue #136 public-safe terminal QA report ghp_secret"
                    .to_string(),
            ),
            notes: Some(
                "/home/example/private interactive resize and exit passed npm_secret".to_string(),
            ),
            collected_at: Some("2026-05-17".to_string()),
        },
    )
    .expect("terminal evidence");

    assert_eq!(evidence.schema_version, 1);
    assert_eq!(evidence.reports.len(), 1);
    let report = &evidence.reports[0];
    assert_eq!(report.platform, "linux");
    assert_eq!(
        report.status,
        LaunchJudgeTerminalEvidenceStatus::PassedWithWarnings
    );
    assert_eq!(
        report.environment,
        Some(LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal)
    );
    assert_eq!(report.collected_at.as_deref(), Some("2026-05-17"));
    assert!(report.command_summary.contains("approval_required"));
    assert!(
        report
            .notes
            .as_deref()
            .unwrap_or_default()
            .contains("node=v")
    );
    assert!(
        !serde_json::to_string(&evidence)
            .unwrap()
            .contains("/Users/")
    );
    assert!(!serde_json::to_string(&evidence).unwrap().contains("/home/"));
    assert!(
        !serde_json::to_string(&evidence)
            .unwrap()
            .contains("example/private")
    );
    assert!(!serde_json::to_string(&evidence).unwrap().contains("ghp_"));
    assert!(!serde_json::to_string(&evidence).unwrap().contains("npm_"));
}

#[test]
fn evidence_from_smoke_rejects_unsupported_auto_platform() {
    let smoke = smoke_report(SmokeStatus::Passed, "macos");
    let error = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: true,
            markdown: false,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            skip_gate: false,
            platform: None,
            environment: None,
            source: None,
            notes: None,
            collected_at: None,
        },
    )
    .expect_err("macos is not launch terminal evidence");

    assert!(error.to_string().contains("linux or windows"));
}

#[test]
fn terminal_evidence_markdown_is_ready_for_issue_comments() {
    let smoke = smoke_report(SmokeStatus::Passed, "windows");
    let evidence = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: false,
            markdown: true,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            skip_gate: false,
            platform: None,
            environment: Some("vm-or-cloud-terminal".to_string()),
            source: Some("issue #136 Windows terminal report".to_string()),
            notes: Some("resize, mouse, and exit checks passed".to_string()),
            collected_at: Some("2026-05-17".to_string()),
        },
    )
    .expect("terminal evidence");

    let markdown = render_markdown(&evidence).expect("markdown");
    assert!(markdown.contains("```json"));
    assert!(markdown.contains("\"schemaVersion\": 1"));
    assert!(markdown.contains("\"platform\": \"windows\""));
    assert!(markdown.contains("\"environment\": \"vm_or_cloud_terminal\""));
    assert!(markdown.contains("\"collectedAt\": \"2026-05-17\""));
    assert!(markdown.contains("This command does not create, edit, or close GitHub issues"));
    assert_eq!(extract_json_blocks(&markdown).len(), 1);
    assert!(!markdown.contains("/Users/"));
    assert!(!markdown.contains("/home/"));
    assert!(!markdown.contains("ghp_"));
    assert!(!markdown.contains("npm_"));
}

#[test]
fn terminal_evidence_rejects_ambiguous_output_modes() {
    let error = validate_output_mode(&TerminalEvidenceOptions {
        json: true,
        markdown: true,
        prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
        skip_gate: false,
        platform: None,
        environment: None,
        source: None,
        notes: None,
        collected_at: None,
    })
    .expect_err("ambiguous output mode should fail");

    assert!(error.to_string().contains("--json or --markdown"));
}

#[test]
fn terminal_evidence_rejects_invalid_collected_at_override_without_echoing_input() {
    let smoke = smoke_report(SmokeStatus::Passed, "linux");
    let error = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: true,
            markdown: false,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            skip_gate: false,
            platform: None,
            environment: None,
            source: None,
            notes: None,
            collected_at: Some("/home/example/private 2026-99-99 npm_secret".to_string()),
        },
    )
    .expect_err("invalid collectedAt override should fail");

    let message = error.to_string();
    assert!(message.contains("--collected-at must use YYYY-MM-DD"));
    assert!(!message.contains("/home/"));
    assert!(!message.contains("npm_"));
}

#[test]
fn terminal_evidence_defaults_collected_at_to_utc_date() {
    let smoke = smoke_report(SmokeStatus::Passed, "linux");
    let evidence = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: true,
            markdown: false,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            skip_gate: false,
            platform: None,
            environment: None,
            source: None,
            notes: None,
            collected_at: None,
        },
    )
    .expect("terminal evidence");

    let collected_at = evidence.reports[0]
        .collected_at
        .as_deref()
        .expect("collectedAt should be populated");
    assert_eq!(collected_at.len(), 10);
    assert_eq!(&collected_at[4..5], "-");
    assert_eq!(&collected_at[7..8], "-");
}

#[test]
fn unix_days_to_date_uses_utc_calendar_dates() {
    assert_eq!(unix_days_to_date(0), "1970-01-01");
    assert_eq!(unix_days_to_date(20_225), "2025-05-17");
    assert_eq!(unix_days_to_date(20_590), "2026-05-17");
}

fn smoke_report(status: SmokeStatus, os: &str) -> SmokeReport {
    let mut tools = BTreeMap::new();
    tools.insert(
        "node".to_string(),
        command_check("node --version", "v24.0.0"),
    );
    tools.insert("npm".to_string(), command_check("npm --version", "11.0.0"));
    tools.insert(
        "rustc".to_string(),
        command_check("rustc --version", "rustc 1.94.0"),
    );
    tools.insert(
        "cargo".to_string(),
        command_check("cargo --version", "cargo 1.94.0"),
    );
    SmokeReport {
        schema_version: 1,
        status,
        workspace: "/Users/example/private".to_string(),
        environment: EnvironmentSummary {
            os: os.to_string(),
            arch: "x86_64".to_string(),
            family: "unix".to_string(),
            current_dir: "/Users/example/private".to_string(),
            terminal: BTreeMap::new(),
            tools,
        },
        binary: BinarySummary {
            path: Some("/Users/example/bin/architect-mcp-tui".to_string()),
            source: "source_build".to_string(),
            sha256: Some("abc123".to_string()),
            npm_cache_root: Some("/Users/example/.cache".to_string()),
            npm_cache_env_override: None,
        },
        architect_mcp: McpProcessSpec {
            command: "node".to_string(),
            args: vec!["dist/index.js".to_string()],
            tool_surface: "advanced".to_string(),
        },
        help: command_check("architect-mcp-tui --help", "Usage"),
        adapters: AdapterSummary {
            total: 2,
            ready: 1,
            health: vec![AdapterHealth {
                name: "codex".to_string(),
                command: "codex".to_string(),
                installed: true,
                version: Some("codex-cli 1.0.0".to_string()),
                auth_status: AuthStatus::Authenticated,
                ready: true,
                detail: "Logged in".to_string(),
            }],
        },
        gate_only: GateSmoke {
            attempted: true,
            command: "architect-mcp-tui run --jsonl".to_string(),
            ok: true,
            status: Some("approval_required".to_string()),
            event_types: vec!["mcp_call".to_string(), "complete".to_string()],
            jsonl_line_count: 2,
            jsonl_tail: Vec::new(),
            error: None,
        },
    }
}

fn command_check(command: &str, first_line: &str) -> CommandCheck {
    CommandCheck {
        command: command.to_string(),
        ok: true,
        exit_code: Some(0),
        timed_out: false,
        first_line: Some(first_line.to_string()),
    }
}
