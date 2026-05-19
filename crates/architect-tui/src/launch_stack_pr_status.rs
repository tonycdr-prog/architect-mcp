use crate::launch_stack::{LaunchStackCheckSummary, LaunchStackItemStatus};

pub(crate) struct PrStatusEvidence<'a> {
    pub is_draft: bool,
    pub review_decision: Option<&'a str>,
    pub merge_state_status: &'a str,
    pub mergeable: Option<&'a str>,
    pub checks: &'a LaunchStackCheckSummary,
    pub required_checks_supplied: bool,
}

pub(crate) fn pr_status(evidence: &PrStatusEvidence<'_>) -> LaunchStackItemStatus {
    if evidence.checks.failed > 0
        || !evidence.checks.missing_required_names.is_empty()
        || is_hard_merge_block(evidence.merge_state_status)
        || is_changes_requested(evidence.review_decision)
    {
        LaunchStackItemStatus::Failed
    } else if evidence.is_draft
        || is_review_required(evidence.review_decision)
        || has_unknown_review_decision(evidence.review_decision)
        || evidence.checks.pending > 0
        || evidence.checks.total == 0
        || is_unresolved_merge_state(
            evidence.merge_state_status,
            evidence.mergeable,
            evidence.checks,
            evidence.required_checks_supplied,
        )
    {
        LaunchStackItemStatus::Warning
    } else {
        LaunchStackItemStatus::Passed
    }
}

pub(crate) fn pr_next_action(
    number: u64,
    evidence: &PrStatusEvidence<'_>,
    status: &LaunchStackItemStatus,
) -> Option<String> {
    match status {
        LaunchStackItemStatus::Passed | LaunchStackItemStatus::Waived => None,
        LaunchStackItemStatus::Failed if is_changes_requested(evidence.review_decision) => Some(
            format!("resolve requested changes on PR #{number} before launch stack can be go"),
        ),
        LaunchStackItemStatus::Failed if !evidence.checks.missing_required_names.is_empty() => {
            Some(format!(
                "restore required checks on PR #{number}: {}",
                evidence.checks.missing_required_names.join(", ")
            ))
        }
        LaunchStackItemStatus::Failed if evidence.checks.failed > 0 => Some(format!(
            "fix failing checks on PR #{number} before launch stack can be go"
        )),
        LaunchStackItemStatus::Failed => Some(format!(
            "fix PR #{number} merge state before launch stack can be go"
        )),
        LaunchStackItemStatus::Warning if evidence.is_draft => Some(format!(
            "mark PR #{number} ready when it is reviewable and still green"
        )),
        LaunchStackItemStatus::Warning if is_review_required(evidence.review_decision) => Some(
            format!("complete required review on PR #{number} before final launch go"),
        ),
        LaunchStackItemStatus::Warning if has_unknown_review_decision(evidence.review_decision) => {
            Some(format!(
                "inspect PR #{number} review decision before final launch go"
            ))
        }
        LaunchStackItemStatus::Warning if evidence.checks.pending > 0 => Some(format!(
            "wait for PR #{number} checks to finish before final launch go"
        )),
        LaunchStackItemStatus::Warning
            if evidence.merge_state_status == "UNSTABLE"
                && is_mergeable(evidence.mergeable)
                && !evidence.required_checks_supplied =>
        {
            Some(format!(
                "supply explicit --required-check evidence for PR #{number} before treating mergeable UNSTABLE as green"
            ))
        }
        LaunchStackItemStatus::Warning if evidence.merge_state_status != "CLEAN" => Some(format!(
            "inspect PR #{number} merge state before final launch go; GitHub reports {}",
            evidence.merge_state_status
        )),
        LaunchStackItemStatus::Warning => Some(format!(
            "confirm PR #{number} has required checks before final launch go"
        )),
    }
}

fn is_hard_merge_block(merge_state_status: &str) -> bool {
    matches!(merge_state_status, "DIRTY" | "UNKNOWN")
}

fn is_unresolved_merge_state(
    merge_state_status: &str,
    mergeable: Option<&str>,
    checks: &LaunchStackCheckSummary,
    required_checks_supplied: bool,
) -> bool {
    if merge_state_status == "CLEAN" {
        return false;
    }
    !is_mergeable_unstable_with_required_checks(
        merge_state_status,
        mergeable,
        checks,
        required_checks_supplied,
    )
}

fn is_mergeable_unstable_with_required_checks(
    merge_state_status: &str,
    mergeable: Option<&str>,
    checks: &LaunchStackCheckSummary,
    required_checks_supplied: bool,
) -> bool {
    merge_state_status == "UNSTABLE"
        && is_mergeable(mergeable)
        && required_checks_supplied
        && checks.total > 0
        && checks.pending == 0
        && checks.failed == 0
        && checks.missing_required_names.is_empty()
}

fn is_mergeable(mergeable: Option<&str>) -> bool {
    matches!(mergeable, Some("MERGEABLE"))
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
