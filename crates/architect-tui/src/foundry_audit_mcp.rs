use std::path::Path;

use anyhow::Result;
use serde_json::{Value, json};

use crate::config::TuiConfig;
use crate::foundry_audit_report::{FoundryAuditReport, failed_report, report_from_mcp_values};
use crate::mcp::{ArchitectMcpBridge, McpToolOutcome, StdioMcpClient};

pub(crate) fn run_mcp_foundry_audit(
    workspace: &Path,
    mcp_workspace: &Path,
    config: TuiConfig,
    max_files: usize,
) -> FoundryAuditReport {
    let mut tools_called = Vec::new();
    let workspace_string = workspace.display().to_string();
    let bridge = ArchitectMcpBridge::new(mcp_workspace, config);
    let mut client = match StdioMcpClient::connect(&bridge) {
        Ok(client) => client,
        Err(error) => return failed_report(workspace_string, tools_called, error.to_string()),
    };

    match call_foundry_chain(
        &mut client,
        workspace,
        requires_outside_cwd(workspace, mcp_workspace),
        max_files,
        &mut tools_called,
    ) {
        Ok((review, constitution, inventory, actionability, ledger, forge)) => {
            report_from_mcp_values(
                workspace_string,
                review,
                constitution,
                inventory,
                actionability,
                ledger,
                forge,
            )
        }
        Err(error) => failed_report(workspace_string, tools_called, error.to_string()),
    }
}

fn requires_outside_cwd(workspace: &Path, mcp_workspace: &Path) -> bool {
    if workspace == mcp_workspace {
        return false;
    }
    match (workspace.canonicalize(), mcp_workspace.canonicalize()) {
        (Ok(workspace), Ok(mcp_workspace)) => workspace != mcp_workspace,
        _ => workspace != mcp_workspace,
    }
}

fn call_foundry_chain(
    client: &mut StdioMcpClient,
    workspace: &Path,
    allow_outside_cwd: bool,
    max_files: usize,
    tools_called: &mut Vec<String>,
) -> Result<(Value, Value, Value, Value, Value, Value)> {
    let root_path = workspace.display().to_string();
    let review = call_required(
        client,
        tools_called,
        "review_local_workspace",
        json!({
            "rootPath": root_path,
            "mode": "audit",
            "maxFiles": max_files,
            "maxDetailedFindings": 25,
            "allowOutsideCwd": allow_outside_cwd
        }),
    )?;
    let constitution = call_required(
        client,
        tools_called,
        "derive_local_repo_constitution",
        json!({
            "rootPath": root_path,
            "maxFiles": max_files,
            "allowOutsideCwd": allow_outside_cwd
        }),
    )?;
    let repo_constitution = constitution
        .get("constitution")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let inventory = call_required(
        client,
        tools_called,
        "normalize_foundry_evidence",
        json!({
            "reviewReports": [review.clone()],
            "repoConstitution": repo_constitution
        }),
    )?
    .get("inventory")
    .cloned()
    .unwrap_or_else(|| json!({}));
    let actionability = call_required(
        client,
        tools_called,
        "score_foundry_actionability",
        json!({ "inventory": inventory }),
    )?
    .get("actionability")
    .cloned()
    .unwrap_or_else(|| json!({}));
    let ledger = call_required(
        client,
        tools_called,
        "route_foundry_decisions",
        json!({ "actionability": actionability }),
    )?
    .get("ledger")
    .cloned()
    .unwrap_or_else(|| json!({}));
    let forge = call_required(
        client,
        tools_called,
        "forge_foundry_previews",
        json!({
            "ledger": ledger,
            "repoConstitution": constitution.get("constitution").cloned().unwrap_or_else(|| json!({}))
        }),
    )?
    .get("forge")
    .cloned()
    .unwrap_or_else(|| json!({}));

    Ok((
        review,
        constitution,
        inventory,
        actionability,
        ledger,
        forge,
    ))
}

fn call_required(
    client: &mut StdioMcpClient,
    tools_called: &mut Vec<String>,
    name: &str,
    args: Value,
) -> Result<Value> {
    tools_called.push(name.to_string());
    match client.call_tool(name, args)? {
        McpToolOutcome::Ok { value } => Ok(value),
        McpToolOutcome::Error { error } => anyhow::bail!("{name} failed: {}", error.message),
    }
}

#[cfg(test)]
mod tests {
    use super::requires_outside_cwd;

    #[test]
    fn outside_cwd_detection_canonicalizes_existing_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        assert!(!requires_outside_cwd(root, &root.join(".")));

        let other = temp.path().join("other");
        std::fs::create_dir_all(&other).expect("other");
        assert!(requires_outside_cwd(&other, root));
    }
}
