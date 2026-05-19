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
                scan_truncated: None,
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
        scan_truncated: None,
        errors: None,
        warnings: None,
        violation_count: None,
        detail,
    }
}

pub(crate) fn mcp_review_from_value(value: Value) -> GovernanceMcpReview {
    let gate_status = value
        .pointer("/report/gate/status")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let scan_truncated = value.pointer("/scan/truncated").and_then(Value::as_bool);
    GovernanceMcpReview {
        status: if matches!(gate_status.as_deref(), Some("fail" | "warn")) {
            "warn".to_string()
        } else {
            "ok".to_string()
        },
        gate_status,
        files_reviewed: value.get("filesReviewed").and_then(Value::as_u64),
        scan_truncated,
        errors: value.pointer("/summary/errors").and_then(Value::as_u64),
        warnings: value.pointer("/summary/warnings").and_then(Value::as_u64),
        violation_count: value
            .get("violations")
            .and_then(Value::as_array)
            .map(Vec::len),
        detail: "review_local_workspace mode=audit completed without writing files".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mcp_review_warn_status_and_scan_truncation_are_not_clean() {
        let review = mcp_review_from_value(json!({
            "filesReviewed": 5000,
            "scan": { "truncated": true },
            "summary": { "errors": 0, "warnings": 8 },
            "report": { "gate": { "status": "warn" } },
            "violations": []
        }));

        assert_eq!(review.status, "warn");
        assert_eq!(review.gate_status.as_deref(), Some("warn"));
        assert_eq!(review.scan_truncated, Some(true));
    }
}
