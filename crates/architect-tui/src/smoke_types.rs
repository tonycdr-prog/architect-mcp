use std::collections::BTreeMap;

use serde::Serialize;

use crate::adapter::AdapterHealth;
use crate::mcp::McpProcessSpec;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmokeStatus {
    Passed,
    PassedWithWarnings,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeReport {
    pub schema_version: u8,
    pub status: SmokeStatus,
    pub workspace: String,
    pub environment: EnvironmentSummary,
    pub binary: BinarySummary,
    pub architect_mcp: McpProcessSpec,
    pub help: CommandCheck,
    pub adapters: AdapterSummary,
    pub gate_only: GateSmoke,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSummary {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub current_dir: String,
    pub terminal: BTreeMap<String, String>,
    pub tools: BTreeMap<String, CommandCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinarySummary {
    pub path: Option<String>,
    pub source: String,
    pub sha256: Option<String>,
    pub npm_cache_root: Option<String>,
    pub npm_cache_env_override: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandCheck {
    pub command: String,
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub first_line: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterSummary {
    pub total: usize,
    pub ready: usize,
    pub health: Vec<AdapterHealth>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateSmoke {
    pub attempted: bool,
    pub command: String,
    pub ok: bool,
    pub status: Option<String>,
    pub event_types: Vec<String>,
    pub jsonl_line_count: usize,
    pub jsonl_tail: Vec<String>,
    pub error: Option<String>,
}
