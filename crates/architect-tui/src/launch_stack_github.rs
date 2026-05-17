use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::launch_stack::{
    LaunchStackCheckSummary, LaunchStackIssue, LaunchStackItemStatus, LaunchStackPullRequest,
};

pub(crate) fn fetch_pr(
    workspace: &Path,
    repo: Option<&str>,
    number: u64,
) -> Result<LaunchStackPullRequest, String> {
    let mut args = vec![
        "pr".to_string(),
        "view".to_string(),
        number.to_string(),
        "--json".to_string(),
        "number,title,url,isDraft,mergeStateStatus,statusCheckRollup".to_string(),
    ];
    append_repo_args(&mut args, repo);
    let value = run_gh_json(workspace, &args).map_err(|error| format!("PR #{number}: {error}"))?;
    Ok(pr_from_value(number, &value))
}

pub(crate) fn fetch_issue(
    workspace: &Path,
    repo: Option<&str>,
    number: u64,
) -> Result<LaunchStackIssue, String> {
    let mut args = vec![
        "issue".to_string(),
        "view".to_string(),
        number.to_string(),
        "--json".to_string(),
        "number,title,url,state".to_string(),
    ];
    append_repo_args(&mut args, repo);
    let value =
        run_gh_json(workspace, &args).map_err(|error| format!("issue #{number}: {error}"))?;
    Ok(issue_from_value(number, &value))
}

pub(crate) fn pr_from_value(fallback_number: u64, value: &Value) -> LaunchStackPullRequest {
    let number = value
        .get("number")
        .and_then(Value::as_u64)
        .unwrap_or(fallback_number);
    let is_draft = value
        .get("isDraft")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let merge_state_status = value
        .get("mergeStateStatus")
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_string();
    let checks = summarize_checks(value.get("statusCheckRollup"));
    let status = pr_status(is_draft, &merge_state_status, &checks);
    let next_action = pr_next_action(number, is_draft, &merge_state_status, &checks, &status);
    LaunchStackPullRequest {
        number,
        title: public_text(
            value.get("title").and_then(Value::as_str).unwrap_or(""),
            180,
        ),
        url: public_text(value.get("url").and_then(Value::as_str).unwrap_or(""), 240),
        is_draft,
        merge_state_status: public_text(&merge_state_status, 80),
        checks,
        status,
        next_action,
    }
}

pub(crate) fn issue_from_value(fallback_number: u64, value: &Value) -> LaunchStackIssue {
    let number = value
        .get("number")
        .and_then(Value::as_u64)
        .unwrap_or(fallback_number);
    let state = value
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_ascii_uppercase();
    let status = if state == "CLOSED" {
        LaunchStackItemStatus::Passed
    } else if state == "OPEN" {
        LaunchStackItemStatus::Warning
    } else {
        LaunchStackItemStatus::Failed
    };
    let next_action = match status {
        LaunchStackItemStatus::Passed | LaunchStackItemStatus::Waived => None,
        LaunchStackItemStatus::Warning => Some(format!(
            "resolve or explicitly waive blocker issue #{number} before final launch go"
        )),
        LaunchStackItemStatus::Failed => Some(format!(
            "inspect blocker issue #{number}; state could not be trusted"
        )),
    };
    LaunchStackIssue {
        number,
        title: public_text(
            value.get("title").and_then(Value::as_str).unwrap_or(""),
            180,
        ),
        url: public_text(value.get("url").and_then(Value::as_str).unwrap_or(""), 240),
        state: public_text(&state, 80),
        status,
        waiver_reason: None,
        next_action,
    }
}

pub(crate) fn summarize_checks(value: Option<&Value>) -> LaunchStackCheckSummary {
    let mut summary = LaunchStackCheckSummary {
        total: 0,
        passed: 0,
        pending: 0,
        failed: 0,
        pending_names: Vec::new(),
        failed_names: Vec::new(),
    };
    let Some(items) = value.and_then(Value::as_array) else {
        return summary;
    };
    summary.total = items.len();
    for item in items {
        let name = public_text(
            item.get("name").and_then(Value::as_str).unwrap_or("check"),
            120,
        );
        let status = item
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let conclusion = item
            .get("conclusion")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if status != "COMPLETED" {
            summary.pending += 1;
            summary.pending_names.push(name);
        } else if matches!(conclusion, "SUCCESS" | "SKIPPED" | "NEUTRAL") {
            summary.passed += 1;
        } else if matches!(
            conclusion,
            "FAILURE" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED"
        ) {
            summary.failed += 1;
            summary.failed_names.push(name);
        } else {
            summary.pending += 1;
            summary.pending_names.push(name);
        }
    }
    summary
}

pub(crate) fn public_text(value: &str, max_len: usize) -> String {
    let normalized = value
        .replace('\\', "/")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut text = normalized
        .split_whitespace()
        .map(redact_token)
        .collect::<Vec<_>>()
        .join(" ");
    if text.chars().count() > max_len {
        text = text.chars().take(max_len).collect::<String>();
        text.push_str(" [truncated]");
    }
    text
}

pub(crate) fn append_repo_args(args: &mut Vec<String>, repo: Option<&str>) {
    if let Some(repo) = repo.filter(|repo| !repo.trim().is_empty()) {
        args.push("--repo".to_string());
        args.push(repo.to_string());
    }
}

pub(crate) fn run_gh_json(workspace: &Path, args: &[String]) -> Result<Value, String> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(workspace)
        .output()
        .map_err(|error| {
            format!(
                "gh command could not run: {}",
                public_text(&error.to_string(), 240)
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "gh command failed: {}",
            public_text(stderr.trim(), 240)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("gh JSON could not be parsed: {error}"))
}

fn pr_status(
    is_draft: bool,
    merge_state_status: &str,
    checks: &LaunchStackCheckSummary,
) -> LaunchStackItemStatus {
    if checks.failed > 0 || is_hard_merge_block(merge_state_status) {
        LaunchStackItemStatus::Failed
    } else if is_draft || checks.pending > 0 || checks.total == 0 || merge_state_status != "CLEAN" {
        LaunchStackItemStatus::Warning
    } else {
        LaunchStackItemStatus::Passed
    }
}

fn pr_next_action(
    number: u64,
    is_draft: bool,
    merge_state_status: &str,
    checks: &LaunchStackCheckSummary,
    status: &LaunchStackItemStatus,
) -> Option<String> {
    match status {
        LaunchStackItemStatus::Passed | LaunchStackItemStatus::Waived => None,
        LaunchStackItemStatus::Failed if checks.failed > 0 => Some(format!(
            "fix failing checks on PR #{number} before launch stack can be go"
        )),
        LaunchStackItemStatus::Failed => Some(format!(
            "fix PR #{number} merge state before launch stack can be go"
        )),
        LaunchStackItemStatus::Warning if is_draft => Some(format!(
            "mark PR #{number} ready when it is reviewable and still green"
        )),
        LaunchStackItemStatus::Warning if checks.pending > 0 => Some(format!(
            "wait for PR #{number} checks to finish before final launch go"
        )),
        LaunchStackItemStatus::Warning if merge_state_status != "CLEAN" => Some(format!(
            "resolve PR #{number} merge state before final launch go"
        )),
        LaunchStackItemStatus::Warning => Some(format!(
            "confirm PR #{number} has required checks before final launch go"
        )),
    }
}

fn is_hard_merge_block(merge_state_status: &str) -> bool {
    matches!(merge_state_status, "DIRTY" | "UNKNOWN")
}

fn redact_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    if lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
    {
        "[redacted-secret]".to_string()
    } else if lower.starts_with("/")
        || lower.contains("/users/")
        || lower.contains("/home/")
        || (lower.len() >= 3 && lower.as_bytes()[1] == b':' && lower.as_bytes()[2] == b'/')
    {
        "[redacted-local-path]".to_string()
    } else {
        token.to_string()
    }
}
