use std::path::Path;

use serde_json::{Value, json};

use crate::config::TuiConfig;
use crate::governance_audit_report::GovernanceMcpReview;
use crate::mcp::{ArchitectMcpBridge, McpToolOutcome, StdioMcpClient};

pub(crate) fn run_mcp_repo_review(
    workspace: &Path,
    config: TuiConfig,
    max_files: usize,
) -> GovernanceMcpReview {
    let bridge = ArchitectMcpBridge::new(workspace, config);
    let mut client = match StdioMcpClient::connect(&bridge) {
        Ok(client) => client,
        Err(error) => {
            return GovernanceMcpReview {
                status: "error".to_string(),
                gate_status: None,
                files_reviewed: None,
                errors: None,
                warnings: None,
                violation_count: None,
                detail: error.to_string(),
            };
        }
    };
    let args = json!({
        "rootPath": workspace.display().to_string(),
        "mode": "audit",
        "maxFiles": max_files,
        "maxDetailedFindings": 25,
        "allowOutsideCwd": false
    });
    match client.call_tool("review_local_workspace", args) {
        Ok(McpToolOutcome::Ok { value }) => mcp_review_from_value(value),
        Ok(McpToolOutcome::Error { error }) => error_review(error.message),
        Err(error) => error_review(error.to_string()),
    }
}

fn error_review(detail: String) -> GovernanceMcpReview {
    GovernanceMcpReview {
        status: "error".to_string(),
        gate_status: None,
        files_reviewed: None,
        errors: None,
        warnings: None,
        violation_count: None,
        detail,
    }
}

fn mcp_review_from_value(value: Value) -> GovernanceMcpReview {
    let gate_status = value
        .pointer("/report/gate/status")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    GovernanceMcpReview {
        status: if gate_status.as_deref() == Some("fail") {
            "warn".to_string()
        } else {
            "ok".to_string()
        },
        gate_status,
        files_reviewed: value.get("filesReviewed").and_then(Value::as_u64),
        errors: value.pointer("/summary/errors").and_then(Value::as_u64),
        warnings: value.pointer("/summary/warnings").and_then(Value::as_u64),
        violation_count: value
            .get("violations")
            .and_then(Value::as_array)
            .map(Vec::len),
        detail: "review_local_workspace mode=audit completed without writing files".to_string(),
    }
}
