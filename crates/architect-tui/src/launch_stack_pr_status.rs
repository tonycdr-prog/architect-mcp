use crate::launch_stack::{LaunchStackCheckSummary, LaunchStackItemStatus};

pub(crate) fn pr_status(
    is_draft: bool,
    review_decision: Option<&str>,
    merge_state_status: &str,
    checks: &LaunchStackCheckSummary,
) -> LaunchStackItemStatus {
    if checks.failed > 0
        || is_hard_merge_block(merge_state_status)
        || is_changes_requested(review_decision)
    {
        LaunchStackItemStatus::Failed
    } else if is_draft
        || is_review_required(review_decision)
        || has_unknown_review_decision(review_decision)
        || checks.pending > 0
        || checks.total == 0
        || merge_state_status != "CLEAN"
    {
        LaunchStackItemStatus::Warning
    } else {
        LaunchStackItemStatus::Passed
    }
}

pub(crate) fn pr_next_action(
    number: u64,
    is_draft: bool,
    review_decision: Option<&str>,
    merge_state_status: &str,
    checks: &LaunchStackCheckSummary,
    status: &LaunchStackItemStatus,
) -> Option<String> {
    match status {
        LaunchStackItemStatus::Passed | LaunchStackItemStatus::Waived => None,
        LaunchStackItemStatus::Failed if is_changes_requested(review_decision) => Some(format!(
            "resolve requested changes on PR #{number} before launch stack can be go"
        )),
        LaunchStackItemStatus::Failed if checks.failed > 0 => Some(format!(
            "fix failing checks on PR #{number} before launch stack can be go"
        )),
        LaunchStackItemStatus::Failed => Some(format!(
            "fix PR #{number} merge state before launch stack can be go"
        )),
        LaunchStackItemStatus::Warning if is_draft => Some(format!(
            "mark PR #{number} ready when it is reviewable and still green"
        )),
        LaunchStackItemStatus::Warning if is_review_required(review_decision) => Some(format!(
            "complete required review on PR #{number} before final launch go"
        )),
        LaunchStackItemStatus::Warning if has_unknown_review_decision(review_decision) => Some(
            format!("inspect PR #{number} review decision before final launch go"),
        ),
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

fn is_changes_requested(review_decision: Option<&str>) -> bool {
    matches!(review_decision, Some("CHANGES_REQUESTED"))
}

fn is_review_required(review_decision: Option<&str>) -> bool {
    matches!(review_decision, Some("REVIEW_REQUIRED"))
}

fn has_unknown_review_decision(review_decision: Option<&str>) -> bool {
    matches!(
        review_decision,
        Some(decision) if !matches!(decision, "APPROVED" | "CHANGES_REQUESTED" | "REVIEW_REQUIRED")
    )
}
