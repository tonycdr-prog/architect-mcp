use std::collections::BTreeMap;

use crate::adapter::{AdapterHealth, AuthStatus};
use crate::issue_terminal_evidence_source::extract_json_blocks;
use crate::launch_judge_report::{
    LaunchJudgeTerminalEvidenceEnvironment, LaunchJudgeTerminalEvidenceReport,
    LaunchJudgeTerminalEvidenceStatus,
};
use crate::mcp::McpProcessSpec;
use crate::smoke::SmokeStatus;
use crate::smoke_types::{
    AdapterSummary, BinarySummary, CommandCheck, EnvironmentSummary, GateSmoke, SmokeReport,
};
use crate::terminal_evidence::{
    TerminalEvidenceFile, TerminalEvidenceOptions, evidence_from_smoke,
    has_failed_terminal_evidence, render_markdown, resolve_platform, validate_output_mode,
};
use crate::terminal_evidence_issue_url::validate_issue_url;

#[test]
fn terminal_evidence_platform_override_must_match_current_os() {
    let error = resolve_platform("linux", Some("windows"))
        .expect_err("platform override must match actual smoke OS");

    assert!(
        error
            .to_string()
            .contains("--platform must match the current terminal OS")
    );
}

#[test]
fn terminal_evidence_platform_override_cannot_relabel_unsupported_os() {
    let error = resolve_platform("macos", Some("linux"))
        .expect_err("unsupported OS must not be relabelled");

    assert!(error.to_string().contains("linux or windows"));
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
            issue_url: None,
        },
    )
    .expect_err("macos is not launch terminal evidence");

    assert!(error.to_string().contains("linux or windows"));
}

#[test]
fn terminal_evidence_rejects_skipped_gate_evidence() {
    let mut smoke = smoke_report(SmokeStatus::Passed, "linux");
    smoke.gate_only.attempted = false;
    smoke.gate_only.ok = true;
    smoke.gate_only.status = Some("skipped".to_string());

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
            issue_url: None,
        },
    )
    .expect_err("skipped gate evidence should fail closed");

    assert!(error.to_string().contains("skipped gates cannot produce"));
}

#[test]
fn terminal_evidence_rejects_skip_gate_option_before_smoke() {
    let error = validate_output_mode(&TerminalEvidenceOptions {
        json: true,
        markdown: false,
        prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
        skip_gate: true,
        platform: None,
        environment: None,
        source: None,
        notes: None,
        collected_at: None,
        issue_url: None,
    })
    .expect_err("skip-gate launch evidence should be rejected");

    assert!(error.to_string().contains("cannot use --skip-gate"));
}

#[test]
fn terminal_evidence_marks_failed_smoke_for_nonzero_exit() {
    let smoke = smoke_report(SmokeStatus::Failed, "linux");
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
            issue_url: None,
        },
    )
    .expect("failed smoke still prints public evidence before exit");

    assert!(has_failed_terminal_evidence(&evidence));
}

#[test]
fn terminal_evidence_markdown_can_name_target_issue_url() {
    let evidence = TerminalEvidenceFile {
        schema_version: 1,
        reports: vec![LaunchJudgeTerminalEvidenceReport {
            platform: "linux".to_string(),
            status: LaunchJudgeTerminalEvidenceStatus::Passed,
            environment: Some(LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal),
            source: "issue #136 Linux terminal report".to_string(),
            command_summary: "architect-mcp-tui terminal-evidence passed on linux".to_string(),
            collected_at: Some("2026-05-17".to_string()),
            notes: Some("summary only, no raw logs".to_string()),
        }],
    };

    let markdown = render_markdown(
        &evidence,
        Some("https://github.com/tonycdr-prog/architect-mcp/issues/136"),
    )
    .expect("markdown");

    assert!(markdown.contains(
        "paste this comment manually into https://github.com/tonycdr-prog/architect-mcp/issues/136"
    ));
    assert!(markdown.contains("This command does not create, edit, or close GitHub issues"));
    assert_eq!(extract_json_blocks(&markdown).len(), 1);
}

#[test]
fn terminal_evidence_rejects_issue_url_without_markdown_mode() {
    let error = validate_output_mode(&TerminalEvidenceOptions {
        json: true,
        markdown: false,
        prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
        skip_gate: false,
        platform: None,
        environment: None,
        source: None,
        notes: None,
        collected_at: None,
        issue_url: Some("https://github.com/tonycdr-prog/architect-mcp/issues/136".to_string()),
    })
    .expect_err("issue-url should only be meaningful in markdown mode");

    assert!(
        error
            .to_string()
            .contains("--issue-url can only be used with --markdown")
    );
}

#[test]
fn terminal_evidence_issue_url_rejects_non_github_issue_urls_without_echoing_input() {
    let error = validate_issue_url("https://example.com/Users/private/issues/136?token=npm_secret")
        .expect_err("non-GitHub issue URL should fail");
    let message = error.to_string();

    assert!(message.contains("--issue-url must be a plain GitHub issue URL"));
    assert!(!message.contains("example.com"));
    assert!(!message.contains("/Users/"));
    assert!(!message.contains("npm_"));
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
