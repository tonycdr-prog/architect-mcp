use std::collections::BTreeMap;

use crate::adapter::{AdapterHealth, AuthStatus};
use crate::launch_judge_report::LaunchJudgeTerminalEvidenceStatus;
use crate::mcp::McpProcessSpec;
use crate::smoke::SmokeStatus;
use crate::smoke_types::{
    AdapterSummary, BinarySummary, CommandCheck, EnvironmentSummary, GateSmoke, SmokeReport,
};
use crate::terminal_evidence::{TerminalEvidenceOptions, evidence_from_smoke};

#[test]
fn evidence_from_smoke_outputs_public_safe_summary() {
    let smoke = smoke_report(SmokeStatus::PassedWithWarnings, "linux");
    let evidence = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: true,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            skip_gate: false,
            platform: None,
            source: Some(
                "/home/example/private issue #136 public-safe terminal QA report".to_string(),
            ),
            notes: Some("/home/example/private interactive resize and exit passed".to_string()),
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
}

#[test]
fn evidence_from_smoke_rejects_unsupported_auto_platform() {
    let smoke = smoke_report(SmokeStatus::Passed, "macos");
    let error = evidence_from_smoke(
        &smoke,
        &TerminalEvidenceOptions {
            json: true,
            prompt: crate::smoke::SmokeOptions::DEFAULT_PROMPT.to_string(),
            skip_gate: false,
            platform: None,
            source: None,
            notes: None,
        },
    )
    .expect_err("macos is not launch terminal evidence");

    assert!(error.to_string().contains("linux or windows"));
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
