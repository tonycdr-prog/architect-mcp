use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::session::{ApprovalStatus, SessionPhase, TuiSession};
use crate::verification::ensure_verification_passed;

pub(crate) const REQUIRED_REVIEW_GATES: &[&str] = &[
    "review_implementation_against_contract",
    "review_repo_structure",
    "review_agent_final_response",
    "review_agent_session",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromotionReadiness {
    pub ready: bool,
    pub blockers: Vec<String>,
    pub next_actions: Vec<String>,
}

pub fn promote_approved_changes(
    session: &mut TuiSession,
    workspace: &Path,
) -> Result<Vec<PathBuf>> {
    let readiness = promotion_readiness(session, workspace);
    if !readiness.ready {
        anyhow::bail!(
            "promotion blocked: {}; next: {}",
            readiness.blockers.join("; "),
            readiness.next_actions.join("; ")
        );
    }
    let worktree = session
        .worktree
        .as_ref()
        .expect("readiness checked worktree");

    let mut promoted = Vec::new();
    for file in &session.changed_files {
        let relative = changed_file_relative_path(file)?;
        let source = worktree.join(&relative);
        let destination = workspace.join(&relative);
        if source.exists() {
            copy_file_from_worktree(&source, &destination)?;
        } else if destination.exists() {
            fs::remove_file(&destination)
                .with_context(|| format!("failed to remove {}", destination.display()))?;
        }
        promoted.push(relative);
    }
    session.mark_promoted();
    Ok(promoted)
}

pub(crate) fn promotion_readiness(session: &TuiSession, workspace: &Path) -> PromotionReadiness {
    let mut blockers = Vec::new();
    let mut next_actions = Vec::new();
    let override_recorded = session.approval_status == ApprovalStatus::Override;

    if !session.can_promote() {
        blockers.push("promotion approval missing".to_string());
        if session.phase == SessionPhase::Complete {
            push_action(&mut next_actions, "run approve <reason>");
        } else {
            push_action(
                &mut next_actions,
                "complete final/session review, then run approve <reason>",
            );
        }
    }

    match &session.worktree {
        Some(worktree) => {
            if let Err(error) = ensure_isolated_worktree(workspace, worktree) {
                blockers.push(error.to_string());
                push_action(&mut next_actions, "rerun adapter in an isolated worktree");
            }
        }
        None => {
            blockers.push("adapter isolated worktree evidence missing".to_string());
            push_action(&mut next_actions, "run adapter after execution approval");
        }
    }

    if session.changed_files.is_empty() {
        blockers.push("changed-file evidence missing".to_string());
        push_action(
            &mut next_actions,
            "run diff summary after adapter execution",
        );
    } else {
        for file in &session.changed_files {
            if let Err(error) = changed_file_relative_path(file) {
                blockers.push(format!("changed-file evidence invalid: {error}"));
                push_action(
                    &mut next_actions,
                    "rerun adapter to refresh changed-file evidence",
                );
            }
        }
    }

    if !override_recorded {
        for issue in &session.adapter_run_issues {
            blockers.push(format!("adapter run issue: {issue}"));
        }
        if !session.adapter_run_issues.is_empty() {
            push_action(
                &mut next_actions,
                "rerun adapter successfully or record an explicit override",
            );
        }
        if let Err(error) = ensure_verification_passed(session) {
            blockers.push(error.to_string());
            push_action(
                &mut next_actions,
                "run verification status and record verification <required check>=passed",
            );
        }
        review_gate_blockers(session, &mut blockers, &mut next_actions);
    }

    PromotionReadiness {
        ready: blockers.is_empty(),
        blockers,
        next_actions,
    }
}

pub(crate) fn promotion_status_lines(session: &TuiSession, workspace: &Path) -> Vec<String> {
    let readiness = promotion_readiness(session, workspace);
    let mut lines = vec![format!(
        "promotion: {}",
        if readiness.ready { "ready" } else { "blocked" }
    )];
    if session.approval_status == ApprovalStatus::Override {
        lines.push("override: recorded; review and verification blockers are bypassed".to_string());
    }
    lines.extend(
        readiness
            .blockers
            .into_iter()
            .map(|blocker| format!("blocker: {blocker}")),
    );
    lines.extend(
        readiness
            .next_actions
            .into_iter()
            .map(|action| format!("next: {action}")),
    );
    lines
}

fn review_gate_blockers(
    session: &TuiSession,
    blockers: &mut Vec<String>,
    next_actions: &mut Vec<String>,
) {
    if session.approval_status == ApprovalStatus::Override {
        return;
    }
    for gate in REQUIRED_REVIEW_GATES {
        match session.gates.get(*gate) {
            Some(review) => {
                if review_blocks_promotion(review) {
                    blockers.push(format!("review gate blocking: {gate}"));
                    push_action(next_actions, review_gate_next_action(gate));
                }
            }
            None => {
                blockers.push(format!("missing review gate: {gate}"));
                push_action(next_actions, review_gate_next_action(gate));
            }
        }
    }
}

fn review_blocks_promotion(review: &Value) -> bool {
    review.get("valid").and_then(Value::as_bool) == Some(false)
        || status_blocks(review.get("status"))
        || status_blocks(review.pointer("/report/gate/status"))
        || review
            .pointer("/summary/errors")
            .and_then(Value::as_u64)
            .is_some_and(|errors| errors > 0)
        || review
            .get("violations")
            .and_then(Value::as_array)
            .is_some_and(|violations| violations.iter().any(violation_blocks))
        || review
            .get("sections")
            .and_then(Value::as_array)
            .is_some_and(|sections| {
                sections
                    .iter()
                    .any(|section| status_blocks(section.get("status")))
            })
}

fn status_blocks(status: Option<&Value>) -> bool {
    matches!(
        status.and_then(Value::as_str),
        Some("fail" | "failed" | "blocked" | "blocker")
    )
}

fn violation_blocks(violation: &Value) -> bool {
    matches!(
        violation.get("severity").and_then(Value::as_str),
        Some("error")
    ) || status_blocks(violation.get("status"))
}

fn review_gate_next_action(gate: &str) -> &'static str {
    match gate {
        "review_agent_final_response" => "run final review <response>",
        "review_agent_session" => "run session review",
        _ => "rerun adapter to refresh implementation review evidence",
    }
}

fn push_action(next_actions: &mut Vec<String>, action: &str) {
    if !next_actions.iter().any(|existing| existing == action) {
        next_actions.push(action.to_string());
    }
}

fn ensure_isolated_worktree(workspace: &Path, worktree: &Path) -> Result<()> {
    let root = workspace.join(".architect-mcp").join("worktrees");
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to read {}", root.display()))?;
    let worktree = worktree
        .canonicalize()
        .with_context(|| format!("failed to read {}", worktree.display()))?;
    if !worktree.starts_with(&root) {
        anyhow::bail!("promotion requires an architect-mcp isolated worktree");
    }
    Ok(())
}

fn changed_file_relative_path(file: &serde_json::Value) -> Result<PathBuf> {
    let path = file
        .get("path")
        .and_then(|value| value.as_str())
        .context("changed file evidence is missing a path")?;
    safe_relative_path(path)
}

fn safe_relative_path(path: &str) -> Result<PathBuf> {
    let relative = Path::new(path);
    if relative.is_absolute() {
        anyhow::bail!("absolute changed file paths cannot be promoted");
    }
    for component in relative.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            anyhow::bail!("unsafe changed file path cannot be promoted: {path}");
        }
    }
    Ok(relative.to_path_buf())
}

fn copy_file_from_worktree(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_file() {
        anyhow::bail!(
            "promotion only supports regular files: {}",
            source.display()
        );
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to promote {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}
