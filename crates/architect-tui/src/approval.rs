use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

use crate::session::{ApprovalStatus, TuiSession};

const REQUIRED_REVIEW_GATES: &[&str] = &[
    "review_implementation_against_contract",
    "review_repo_structure",
    "review_agent_final_response",
    "review_agent_session",
];

pub fn promote_approved_changes(
    session: &mut TuiSession,
    workspace: &Path,
) -> Result<Vec<PathBuf>> {
    if !session.can_promote() {
        anyhow::bail!("approval is required before promotion");
    }
    ensure_reviews_or_override(session)?;
    let worktree = session
        .worktree
        .as_ref()
        .context("adapter run has no isolated worktree evidence")?;
    ensure_isolated_worktree(workspace, worktree)?;

    let mut promoted = Vec::new();
    for file in &session.changed_files {
        let path = file
            .get("path")
            .and_then(|value| value.as_str())
            .context("changed file evidence is missing a path")?;
        let relative = safe_relative_path(path)?;
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

fn ensure_reviews_or_override(session: &TuiSession) -> Result<()> {
    if session.approval_status == ApprovalStatus::Override {
        return Ok(());
    }
    for gate in REQUIRED_REVIEW_GATES {
        let review = session
            .gates
            .get(*gate)
            .with_context(|| format!("promotion requires {gate} or explicit override"))?;
        let text = review.to_string().to_lowercase();
        if text.contains("fail") || text.contains("blocker") {
            anyhow::bail!("promotion blocked by {gate}");
        }
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn promotion_requires_approval_and_isolated_worktree() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path();
        let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
        fs::create_dir_all(worktree.join("docs")).expect("worktree");
        fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

        let mut session = TuiSession::new("build", "codex");
        session.worktree = Some(worktree);
        session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
        assert!(promote_approved_changes(&mut session, workspace).is_err());

        session.approve("reviewed");
        for gate in REQUIRED_REVIEW_GATES {
            session.set_gate(gate, json!({ "ok": true }));
        }
        let promoted = promote_approved_changes(&mut session, workspace).expect("promote");
        assert_eq!(promoted, vec![PathBuf::from("docs/result.md")]);
        assert_eq!(
            fs::read_to_string(workspace.join("docs/result.md")).expect("promoted"),
            "done\n"
        );
    }

    #[test]
    fn promotion_rejects_unsafe_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path();
        let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
        fs::create_dir_all(&worktree).expect("worktree");

        let mut session = TuiSession::new("build", "codex");
        session.approve("reviewed");
        session.worktree = Some(worktree);
        for gate in REQUIRED_REVIEW_GATES {
            session.set_gate(gate, json!({ "ok": true }));
        }
        session.changed_files = vec![json!({ "path": "../secrets.env", "lines": 1 })];
        assert!(promote_approved_changes(&mut session, workspace).is_err());
    }

    #[test]
    fn promotion_requires_review_gates_unless_overridden() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path();
        let worktree = workspace.join(".architect-mcp/worktrees/session/codex");
        fs::create_dir_all(worktree.join("docs")).expect("worktree");
        fs::write(worktree.join("docs/result.md"), "done\n").expect("file");

        let mut session = TuiSession::new("build", "codex");
        session.approve("reviewed");
        session.worktree = Some(worktree);
        session.changed_files = vec![json!({ "path": "docs/result.md", "lines": 1 })];
        assert!(promote_approved_changes(&mut session, workspace).is_err());

        session.override_approval("maintainer override for smoke fixture");
        assert!(promote_approved_changes(&mut session, workspace).is_ok());
    }
}
