use std::path::Path;

use serde_json::{Value, json};

use crate::launch_stack::{
    LaunchStackCheckSummary, LaunchStackIssue, LaunchStackItemStatus, LaunchStackPullRequest,
    LaunchStackReviewThread,
};
pub(crate) use crate::launch_stack_github_support::{public_text, run_gh_json};
use crate::launch_stack_pr_status::{PrStatusEvidence, pr_next_action, pr_status};
use crate::launch_stack_required_checks::apply_required_checks;
use crate::launch_stack_review_threads::fetch_unresolved_review_threads;

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
        "id,number,title,url,isDraft,reviewDecision,mergeStateStatus,mergeable,statusCheckRollup"
            .to_string(),
    ];
    append_repo_args(&mut args, repo);
    let mut value =
        run_gh_json(workspace, &args).map_err(|error| format!("PR #{number}: {error}"))?;
    let pr_id = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| format!("PR #{number}: GitHub response omitted PR node id"))?;
    let unresolved_review_thread_details = fetch_unresolved_review_threads(workspace, pr_id)
        .map_err(|error| format!("PR #{number}: {error}"))?;
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "unresolvedReviewThreads".to_string(),
            json!(unresolved_review_thread_details.len()),
        );
        object.insert(
            "unresolvedReviewThreadDetails".to_string(),
            json!(unresolved_review_thread_details),
        );
    }
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
    let unresolved_review_threads = value
        .get("unresolvedReviewThreads")
        .and_then(Value::as_u64)
        .unwrap_or_default() as usize;
    let unresolved_review_thread_details =
        parse_unresolved_review_thread_details(value.get("unresolvedReviewThreadDetails"));
    let mut checks = summarize_checks(value.get("statusCheckRollup"));
    apply_required_checks(&mut checks, required_checks);
    let evidence = PrStatusEvidence {
        is_draft,
        review_decision: review_decision.as_deref(),
        unresolved_review_threads,
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
        unresolved_review_threads,
        unresolved_review_thread_details,
        merge_state_status: public_text(&merge_state_status, 80),
        mergeable,
        checks,
        status,
        next_action,
    }
}

fn parse_unresolved_review_thread_details(value: Option<&Value>) -> Vec<LaunchStackReviewThread> {
    let Some(items) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let url = item.get("url").and_then(Value::as_str).unwrap_or("");
            let path = item.get("path").and_then(Value::as_str).unwrap_or("");
            if url.trim().is_empty() && path.trim().is_empty() {
                return None;
            }
            Some(LaunchStackReviewThread {
                url: public_text(url, 240),
                path: public_text(path, 180),
                line: item.get("line").and_then(Value::as_u64),
                author: item
                    .get("author")
                    .and_then(Value::as_str)
                    .map(|author| public_text(author, 80)),
                outdated: item
                    .get("outdated")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
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

pub(crate) fn append_repo_args(args: &mut Vec<String>, repo: Option<&str>) {
    if let Some(repo) = repo.filter(|repo| !repo.trim().is_empty()) {
        args.push("--repo".to_string());
        args.push(repo.to_string());
    }
}
