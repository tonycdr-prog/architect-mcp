use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

use crate::gate_calls::call_gate;
use crate::headless::GateReviewState;
use crate::headless::HeadlessRunOptions;
use crate::mcp::StdioMcpClient;

#[derive(Debug, Clone)]
pub(crate) struct DiffEvidence {
    pub changed_files: Vec<Value>,
    pub changed_directories: Vec<String>,
    pub diff_stat: String,
}

pub(crate) fn collect_diff_evidence(workspace: &std::path::Path) -> Result<DiffEvidence> {
    let status = command_output(workspace, ["status", "--short", "--untracked-files=all"])?;
    let numstat = command_output(workspace, ["diff", "--numstat"])?;
    let diff_stat = non_empty_diff_stat(workspace, &status)?;
    let changed_paths = parse_status_paths(&status);
    let line_counts = parse_numstat(&numstat);
    let mut directories = BTreeSet::new();
    let changed_files = changed_paths
        .into_iter()
        .map(|path| {
            if let Some(parent) = std::path::Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                && !parent.is_empty()
            {
                directories.insert(parent.to_string());
            }
            let lines = line_counts
                .get(&path)
                .copied()
                .unwrap_or_else(|| count_file_lines(workspace, &path));
            json!({ "path": path, "lines": lines })
        })
        .collect();
    Ok(DiffEvidence {
        changed_files,
        changed_directories: directories.into_iter().collect(),
        diff_stat,
    })
}

pub(crate) async fn review_adapter_work<W: AsyncWriteExt + Unpin>(
    client: &mut StdioMcpClient,
    output: &mut W,
    options: &HeadlessRunOptions,
    gate_state: &GateReviewState,
    evidence: &DiffEvidence,
) -> Result<()> {
    let contract = gate_state
        .pre_edit
        .get("contract")
        .cloned()
        .unwrap_or_else(|| gate_state.pre_edit.clone());
    let verification = verification_statuses(&gate_state.verification);
    call_gate(
        client,
        output,
        options.jsonl,
        "review_implementation_against_contract",
        "review adapter diff against the approved pre-edit contract",
        json!({
            "input": {
                "contract": contract,
                "changedFiles": evidence.changed_files.clone(),
                "verification": verification.clone()
            }
        }),
    )
    .await?;
    call_gate(
        client,
        output,
        options.jsonl,
        "review_repo_structure",
        "review changed file summaries before promotion",
        repo_structure_args(gate_state, evidence),
    )
    .await?;
    let final_response = final_response_text(evidence, &gate_state.verification);
    call_gate(
        client,
        output,
        options.jsonl,
        "review_agent_final_response",
        "check adapter summary honesty before user approval",
        json!({ "request": { "response": final_response, "requiredChecks": gate_state.verification } }),
    )
    .await?;
    call_gate(
        client,
        output,
        options.jsonl,
        "review_agent_session",
        "review the full adapter session before promotion",
        json!({
            "request": {
                "intent": gate_state.pre_edit.get("intent").cloned().unwrap_or(Value::Null),
                "contract": gate_state.pre_edit.get("contract").cloned().unwrap_or(Value::Null),
                "changedFiles": evidence.changed_files.clone(),
                "verification": verification,
                "finalResponse": final_response
            }
        }),
    )
    .await?;
    Ok(())
}

fn non_empty_diff_stat(workspace: &std::path::Path, status: &str) -> Result<String> {
    let diff_stat = command_output(workspace, ["diff", "--stat"])?;
    if diff_stat.is_empty() {
        Ok(status.to_string())
    } else {
        Ok(diff_stat)
    }
}

fn command_output<const N: usize>(workspace: &std::path::Path, args: [&str; N]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .context("failed to run git evidence command")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn count_file_lines(workspace: &std::path::Path, path: &str) -> u64 {
    let path = workspace.join(path);
    std::fs::read_to_string(path)
        .map(|content| content.lines().count() as u64)
        .unwrap_or(0)
}

fn parse_status_paths(status: &str) -> Vec<String> {
    let mut paths = status
        .lines()
        .filter_map(|line| line.get(3..))
        .map(|path| path.split(" -> ").last().unwrap_or(path).trim().to_string())
        .filter(|path| !path.is_empty())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn parse_numstat(numstat: &str) -> BTreeMap<String, u64> {
    numstat
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let added = parts.next()?.parse::<u64>().unwrap_or(0);
            let deleted = parts.next()?.parse::<u64>().unwrap_or(0);
            let path = parts.next()?.to_string();
            Some((path, added + deleted))
        })
        .collect()
}

fn verification_statuses(checks: &[String]) -> Vec<Value> {
    checks
        .iter()
        .map(|check| json!({ "check": check, "status": "not_run" }))
        .collect()
}

fn repo_structure_args(gate_state: &GateReviewState, evidence: &DiffEvidence) -> Value {
    let mut args = json!({
        "files": evidence.changed_files.clone(),
        "directories": evidence.changed_directories.clone(),
        "mode": "audit"
    });
    if let Some(contract) = architecture_contract(gate_state) {
        args["contract"] = contract;
    }
    args
}

fn architecture_contract(gate_state: &GateReviewState) -> Option<Value> {
    for candidate in [
        gate_state.pre_edit.get("architectureContract"),
        gate_state.pre_edit.get("contract"),
        Some(&gate_state.pre_edit),
    ]
    .into_iter()
    .flatten()
    {
        if candidate.get("contractVersion").is_some()
            && candidate.get("generatedBy").is_some()
            && candidate.get("directories").is_some()
        {
            return Some(candidate.clone());
        }
    }
    None
}

fn final_response_text(evidence: &DiffEvidence, checks: &[String]) -> String {
    let files = evidence
        .changed_files
        .iter()
        .filter_map(|file| file.get("path").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(", ");
    let checks = if checks.is_empty() {
        "No verification checks were supplied.".to_string()
    } else {
        format!("Verification not run yet: {}.", checks.join(", "))
    };
    format!(
        "Adapter produced isolated worktree changes in: {files}. {checks} Assumptions: changes are not promoted until explicit approval. Not done: verification and promotion remain pending."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_structure_args_omits_pre_edit_contract_shape() {
        let gate_state = GateReviewState {
            pre_edit: json!({
                "contract": {
                    "likelyFiles": ["src/**/*"],
                    "verificationChecks": ["npm test"]
                }
            }),
            verification: vec!["npm test".to_string()],
        };
        let evidence = DiffEvidence {
            changed_files: vec![json!({ "path": "src/main.ts", "lines": 1 })],
            changed_directories: vec!["src".to_string()],
            diff_stat: "src/main.ts | 1 +".to_string(),
        };

        let args = repo_structure_args(&gate_state, &evidence);

        assert!(args.get("contract").is_none());
        assert_eq!(args["files"][0]["path"], "src/main.ts");
    }

    #[test]
    fn repo_structure_args_includes_architecture_contract_shape() {
        let architecture_contract = json!({
            "contractVersion": "1",
            "generatedBy": { "tool": "architect-mcp", "version": "0.0.0", "generatedAt": "2026-05-17" },
            "name": "Example",
            "purpose": "Example purpose",
            "stack": {},
            "stackPacks": [],
            "directories": [],
            "fileRules": [],
            "moduleBoundaries": [],
            "testingExpectations": [],
            "agentInstructions": [],
            "foundationPacks": []
        });
        let gate_state = GateReviewState {
            pre_edit: json!({ "architectureContract": architecture_contract }),
            verification: vec![],
        };
        let evidence = DiffEvidence {
            changed_files: vec![],
            changed_directories: vec![],
            diff_stat: String::new(),
        };

        let args = repo_structure_args(&gate_state, &evidence);

        assert_eq!(args["contract"]["name"], "Example");
    }
}
