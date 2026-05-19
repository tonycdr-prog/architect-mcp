use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::launch_stack::{
    LaunchStackCheckSummary, LaunchStackIssue, LaunchStackItemStatus, LaunchStackPullRequest,
};
use crate::launch_stack_pr_status::{PrStatusEvidence, pr_next_action, pr_status};
use crate::launch_stack_required_checks::apply_required_checks;

pub(crate) fn fetch_pr(
    workspace: &Path,
    repo: Option<&str>,
    number: u64,
    required_checks: &[String],
) -> Result<LaunchStackPullRequest, String> {
    let mut args = vec![
        "pr".to_string(),
        "view".to_string(),
        number.to_string(),
        "--json".to_string(),
        "number,title,url,isDraft,reviewDecision,mergeStateStatus,mergeable,statusCheckRollup"
            .to_string(),
    ];
    append_repo_args(&mut args, repo);
    let value = run_gh_json(workspace, &args).map_err(|error| format!("PR #{number}: {error}"))?;
    Ok(pr_from_value_with_required_checks(
        number,
        &value,
        required_checks,
    ))
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
    pr_from_value_with_required_checks(fallback_number, value, &[])
}

pub(crate) fn pr_from_value_with_required_checks(
    fallback_number: u64,
    value: &Value,
    required_checks: &[String],
) -> LaunchStackPullRequest {
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
    let review_decision = value
        .get("reviewDecision")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|decision| !decision.is_empty())
        .map(|decision| public_text(&decision.to_ascii_uppercase(), 80));
    let mergeable = value
        .get("mergeable")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| public_text(&value.to_ascii_uppercase(), 80));
    let mut checks = summarize_checks(value.get("statusCheckRollup"));
    apply_required_checks(&mut checks, required_checks);
    let evidence = PrStatusEvidence {
        is_draft,
        review_decision: review_decision.as_deref(),
        merge_state_status: &merge_state_status,
        mergeable: mergeable.as_deref(),
        checks: &checks,
        required_checks_supplied: !required_checks.is_empty(),
    };
    let status = pr_status(&evidence);
    let next_action = pr_next_action(number, &evidence, &status);
    LaunchStackPullRequest {
        number,
        title: public_text(
            value.get("title").and_then(Value::as_str).unwrap_or(""),
            180,
        ),
        url: public_text(value.get("url").and_then(Value::as_str).unwrap_or(""), 240),
        is_draft,
        review_decision,
        merge_state_status: public_text(&merge_state_status, 80),
        mergeable,
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
        names: Vec::new(),
        pending_names: Vec::new(),
        failed_names: Vec::new(),
        missing_required_names: Vec::new(),
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
        summary.names.push(name.clone());
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
    if is_private_key_material(&normalized) {
        return "[redacted-secret]".to_string();
    }
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

fn redact_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    if lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
        || is_private_key_material(token)
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

fn is_private_key_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("begin") && lower.contains("private") && lower.contains("key")
}
