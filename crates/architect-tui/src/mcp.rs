use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::TuiConfig;
pub use crate::mcp_client::{McpResponseError, McpToolOutcome, StdioMcpClient};

pub const CORE_WORK_GATE_TOOLS: [&str; 6] = [
    "grill_me",
    "create_pre_edit_contract",
    "review_build_plan",
    "review_proposed_file_plan",
    "review_implementation_against_contract",
    "review_agent_session",
];

pub const FINAL_REVIEW_TOOLS: [&str; 2] = ["review_agent_final_response", "review_agent_session"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpToolCall {
    pub name: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpProcessSpec {
    pub command: String,
    pub args: Vec<String>,
    pub tool_surface: String,
}

#[derive(Debug, Clone)]
pub struct ArchitectMcpBridge {
    workspace: PathBuf,
    config: TuiConfig,
}

impl ArchitectMcpBridge {
    pub fn new(workspace: impl Into<PathBuf>, config: TuiConfig) -> Self {
        Self {
            workspace: workspace.into(),
            config,
        }
    }

    pub fn process_spec(&self) -> McpProcessSpec {
        if let Some(command) = &self.config.architect_mcp.command {
            return McpProcessSpec {
                command: command.clone(),
                args: self.config.architect_mcp.args.clone(),
                tool_surface: self.config.architect_mcp.tool_surface.clone(),
            };
        }

        let dist_index = self.workspace.join("dist").join("index.js");
        if dist_index.exists() {
            McpProcessSpec {
                command: "node".to_string(),
                args: vec![dist_index.display().to_string()],
                tool_surface: "advanced".to_string(),
            }
        } else {
            McpProcessSpec {
                command: "architect-mcp".to_string(),
                args: Vec::new(),
                tool_surface: "advanced".to_string(),
            }
        }
    }

    pub fn spawn(&self) -> Result<std::process::Child> {
        let spec = self.process_spec();
        Command::new(&spec.command)
            .args(&spec.args)
            .env("ARCHITECT_MCP_TOOL_SURFACE", &spec.tool_surface)
            .current_dir(&self.workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to spawn architect-mcp command '{}'", spec.command))
    }

    pub fn work_gate_plan(&self, task: &str) -> Vec<McpToolCall> {
        vec![
            tool("grill_me", format!("clarify task before edits: {task}")),
            tool(
                "create_pre_edit_contract",
                "freeze goals, constraints, risks, and evidence",
            ),
            tool(
                "review_build_plan",
                "constrain workflow slices before agent execution",
            ),
            tool(
                "review_proposed_file_plan",
                "approve touched paths and artifact hygiene",
            ),
            tool(
                "review_implementation_against_contract",
                "review implementation drift before approval",
            ),
            tool(
                "review_agent_final_response",
                "check final answer evidence and gaps",
            ),
            tool(
                "review_agent_session",
                "review whole session for missed gates",
            ),
        ]
    }

    pub fn integration_catalog_tools(&self) -> Vec<&'static str> {
        vec![
            "list_mcp_catalog_entries",
            "recommend_mcp_servers",
            "create_mcp_install_plan",
            "review_mcp_install_plan",
            "apply_mcp_install_plan",
        ]
    }
}

fn tool(name: &str, purpose: impl Into<String>) -> McpToolCall {
    McpToolCall {
        name: name.to_string(),
        purpose: purpose.into(),
    }
}

pub fn dist_index_path(workspace: &Path) -> PathBuf {
    workspace.join("dist").join("index.js")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TuiConfig;

    #[test]
    fn work_gate_plan_starts_with_grill_and_keeps_reviews() {
        let bridge = ArchitectMcpBridge::new(".", TuiConfig::default());
        let calls = bridge.work_gate_plan("build a thing");
        let names: Vec<_> = calls.iter().map(|call| call.name.as_str()).collect();
        assert_eq!(names[0], "grill_me");
        assert!(names.contains(&"review_build_plan"));
        assert!(names.contains(&"review_agent_final_response"));
        assert!(names.contains(&"review_agent_session"));
    }

    #[test]
    fn default_process_uses_advanced_surface() {
        let bridge = ArchitectMcpBridge::new(".", TuiConfig::default());
        let spec = bridge.process_spec();
        assert_eq!(spec.tool_surface, "advanced");
    }
}
