use std::process::Command;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::foundry_smoke_report::FoundryGithubVerification;

pub(crate) fn verify_github_target(target: &str) -> Result<FoundryGithubVerification> {
    let repo = run_json_command(
        Command::new("gh")
            .args(["repo", "view", target, "--json"])
            .arg("nameWithOwner,visibility,isPrivate,url"),
    )?;
    let prs = run_json_command(
        Command::new("gh")
            .args(["pr", "list", "-R", target, "--head", "architect/bootstrap"])
            .args(["--json", "number,url,isDraft,state", "--limit", "1"]),
    )?;
    let pr = prs.as_array().and_then(|items| items.first());
    Ok(FoundryGithubVerification {
        repo_url: string_field(&repo, "url")?,
        repo_visibility: string_field(&repo, "visibility")?,
        is_private: repo
            .get("isPrivate")
            .and_then(Value::as_bool)
            .unwrap_or_default(),
        draft_pr_url: pr
            .and_then(|value| value.get("url"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        draft_pr_number: pr
            .and_then(|value| value.get("number"))
            .and_then(Value::as_u64),
        draft_pr_is_draft: pr
            .and_then(|value| value.get("isDraft"))
            .and_then(Value::as_bool),
        draft_pr_state: pr
            .and_then(|value| value.get("state"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

pub(crate) fn foundry_github_verification_passed(verification: &FoundryGithubVerification) -> bool {
    verification.is_private
        && verification.draft_pr_url.is_some()
        && verification.draft_pr_number.is_some()
        && verification.draft_pr_is_draft == Some(true)
        && verification.draft_pr_state.as_deref() == Some("OPEN")
}

pub(crate) fn ensure_repo_absent(target: &str) -> Result<()> {
    let output = Command::new("gh")
        .args(["repo", "view", target])
        .output()
        .with_context(|| format!("failed to check whether {target} already exists"))?;
    if output.status.success() {
        anyhow::bail!("refusing to overwrite existing GitHub repository {target}");
    }
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if stderr.contains("could not resolve")
        || stderr.contains("not found")
        || stderr.contains("not exist")
    {
        return Ok(());
    }
    anyhow::bail!("could not confirm {target} is absent: {}", stderr.trim())
}

fn run_json_command(command: &mut Command) -> Result<Value> {
    let output = command.output().context("failed to run gh command")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    serde_json::from_slice(&output.stdout).context("failed to parse gh JSON output")
}

fn string_field(value: &Value, field: &str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .with_context(|| format!("gh response omitted {field}"))
}
