use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

use crate::config::TuiConfig;
use crate::mcp::McpProcessSpec;
use crate::orchestrator::{HeadlessRunOptions, Orchestrator};
use crate::smoke_report::{
    adapter_summary, binary_summary, environment_summary, help_check, print_human_report,
    tail_lines,
};
use crate::smoke_types::{AdapterSummary, GateSmoke};

pub use crate::smoke_types::{SmokeReport, SmokeStatus};

#[derive(Debug, Clone)]
pub struct SmokeOptions {
    pub json: bool,
    pub prompt: String,
    pub skip_gate: bool,
}

impl SmokeOptions {
    pub const DEFAULT_PROMPT: &'static str = "Build a tiny local notes app for one developer. Flows: create, edit, delete, and search notes. Stack: TypeScript CLI. Risks: file corruption and unclear persistence. Verification: unit tests for CRUD and search plus npm test.";
}

pub async fn run_smoke(workspace: PathBuf, config: TuiConfig, options: SmokeOptions) -> Result<()> {
    let report = build_smoke_report(workspace, config, options).await;
    if report.options_json {
        println!("{}", serde_json::to_string_pretty(&report.report)?);
    } else {
        print_human_report(&report.report);
    }
    if matches!(report.report.status, SmokeStatus::Failed) {
        anyhow::bail!("terminal smoke failed");
    }
    Ok(())
}

struct BuiltSmokeReport {
    report: SmokeReport,
    options_json: bool,
}

async fn build_smoke_report(
    workspace: PathBuf,
    config: TuiConfig,
    options: SmokeOptions,
) -> BuiltSmokeReport {
    let orchestrator = Orchestrator::new(workspace.clone(), config.clone());
    let help = help_check();
    let adapters = adapter_summary(&config);
    let gate_only = if options.skip_gate {
        skipped_gate(&options.prompt, &config)
    } else {
        run_gate_smoke(&orchestrator, &options.prompt, &config).await
    };
    let status = smoke_status(help.ok, gate_only.ok, warning_count(&adapters));
    let report = SmokeReport {
        schema_version: 1,
        status,
        workspace: workspace.display().to_string(),
        environment: environment_summary(&workspace),
        binary: binary_summary(),
        architect_mcp: orchestrator_bridge_spec(&orchestrator),
        help,
        adapters,
        gate_only,
    };

    BuiltSmokeReport {
        report,
        options_json: options.json,
    }
}

fn warning_count(adapters: &AdapterSummary) -> usize {
    adapters
        .health
        .iter()
        .filter(|health| !health.ready)
        .count()
}

fn orchestrator_bridge_spec(orchestrator: &Orchestrator) -> McpProcessSpec {
    let bridge = crate::mcp::ArchitectMcpBridge::new(
        orchestrator.workspace.clone(),
        orchestrator.config.clone(),
    );
    bridge.process_spec()
}

fn smoke_status(help_ok: bool, gate_ok: bool, warning_count: usize) -> SmokeStatus {
    if !help_ok || !gate_ok {
        SmokeStatus::Failed
    } else if warning_count > 0 {
        SmokeStatus::PassedWithWarnings
    } else {
        SmokeStatus::Passed
    }
}

async fn run_gate_smoke(
    orchestrator: &Orchestrator,
    prompt: &str,
    config: &TuiConfig,
) -> GateSmoke {
    let adapter = config.agents.default_adapter.clone();
    let command = format!(
        "architect-mcp-tui run --prompt {:?} --adapter {} --jsonl",
        prompt, adapter
    );
    let mut output = Vec::new();
    let result = orchestrator
        .run_headless_to_writer(
            HeadlessRunOptions {
                prompt: prompt.to_string(),
                adapter,
                jsonl: true,
                concurrency: 1,
                execute: false,
            },
            &mut output,
        )
        .await;
    let jsonl = String::from_utf8_lossy(&output);
    let lines: Vec<String> = jsonl.lines().map(ToString::to_string).collect();
    let (status, event_types) = summarize_jsonl(&lines);
    let error = result.err().map(|error| error.to_string());
    let ok = error.is_none() && status.as_deref() == Some("approval_required");
    GateSmoke {
        attempted: true,
        command,
        ok,
        status,
        event_types,
        jsonl_line_count: lines.len(),
        jsonl_tail: tail_lines(&lines, 12),
        error,
    }
}

fn skipped_gate(prompt: &str, config: &TuiConfig) -> GateSmoke {
    GateSmoke {
        attempted: false,
        command: format!(
            "architect-mcp-tui run --prompt {:?} --adapter {} --jsonl",
            prompt, config.agents.default_adapter
        ),
        ok: true,
        status: Some("skipped".to_string()),
        event_types: Vec::new(),
        jsonl_line_count: 0,
        jsonl_tail: Vec::new(),
        error: None,
    }
}

fn summarize_jsonl(lines: &[String]) -> (Option<String>, Vec<String>) {
    let mut status = None;
    let mut event_types = Vec::new();
    for line in lines {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(event_type) = value.get("type").and_then(Value::as_str) {
            event_types.push(event_type.to_string());
        }
        if value.get("type").and_then(Value::as_str) == Some("complete") {
            status = value
                .get("status")
                .and_then(Value::as_str)
                .map(ToString::to_string);
        }
    }
    (status, event_types)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_complete_status_and_event_types_from_jsonl() {
        let lines = vec![
            r#"{"type":"session_started"}"#.to_string(),
            r#"{"type":"mcp_call","name":"grill_me"}"#.to_string(),
            r#"{"type":"complete","status":"approval_required"}"#.to_string(),
        ];

        let (status, event_types) = summarize_jsonl(&lines);

        assert_eq!(status.as_deref(), Some("approval_required"));
        assert_eq!(event_types, ["session_started", "mcp_call", "complete"]);
    }

    #[test]
    fn smoke_status_treats_adapter_warnings_as_non_blocking() {
        assert_eq!(smoke_status(true, true, 1), SmokeStatus::PassedWithWarnings);
        assert_eq!(smoke_status(true, true, 0), SmokeStatus::Passed);
        assert_eq!(smoke_status(false, true, 0), SmokeStatus::Failed);
        assert_eq!(smoke_status(true, false, 0), SmokeStatus::Failed);
    }
}
