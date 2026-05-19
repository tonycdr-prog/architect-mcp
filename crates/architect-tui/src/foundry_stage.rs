use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::foundry_stage_content::{artifact_content, first_pr_body};
use crate::session::TuiSession;

const BOOTSTRAP_BRANCH: &str = "architect/bootstrap";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFoundryStage {
    pub path: PathBuf,
    pub bootstrap_branch: String,
    pub artifacts_written: usize,
    pub pr_body_path: PathBuf,
}

pub fn stage_repo_foundry_plan(workspace: &Path, session: &TuiSession) -> Result<RepoFoundryStage> {
    let plan = session
        .foundry_plan
        .as_ref()
        .context("run foundry plan before foundry stage")?;
    let root = workspace
        .join(".architect-mcp")
        .join("foundry")
        .join(&session.id)
        .join(&plan.repo_name);
    prepare_clean_stage_root(workspace, &root)?;
    fs::create_dir_all(&root).with_context(|| format!("failed to create {}", root.display()))?;

    run_git(&root, &["init"])?;
    run_git(
        &root,
        &["config", "user.email", "architect-mcp@example.invalid"],
    )?;
    run_git(&root, &["config", "user.name", "architect-mcp foundry"])?;
    run_git(&root, &["checkout", "-B", "main"])?;
    run_git(
        &root,
        &[
            "commit",
            "--allow-empty",
            "-m",
            "Initialize private app repository",
        ],
    )?;
    run_git(&root, &["checkout", "-B", BOOTSTRAP_BRANCH])?;

    let mut written = 0;
    for artifact in &plan.artifacts {
        let path = safe_join(&root, &artifact.path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&path, artifact_content(plan, session, &artifact.path))
            .with_context(|| format!("failed to write {}", path.display()))?;
        written += 1;
    }
    let pr_body_path = safe_join(&root, "docs/first-pr-draft.md")?;
    if let Some(parent) = pr_body_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&pr_body_path, first_pr_body(plan, session))?;
    written += 1;

    run_git(&root, &["add", "."])?;
    run_git(&root, &["commit", "-m", "Bootstrap governed app scaffold"])?;

    Ok(RepoFoundryStage {
        path: root,
        bootstrap_branch: BOOTSTRAP_BRANCH.to_string(),
        artifacts_written: written,
        pr_body_path,
    })
}

fn prepare_clean_stage_root(workspace: &Path, root: &Path) -> Result<()> {
    let allowed_root = workspace.join(".architect-mcp").join("foundry");
    let allowed_root = allowed_root
        .canonicalize()
        .or_else(|_| {
            fs::create_dir_all(&allowed_root)?;
            allowed_root.canonicalize()
        })
        .with_context(|| format!("failed to prepare {}", allowed_root.display()))?;
    let parent = root
        .parent()
        .context("foundry stage path must have a parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to prepare {}", parent.display()))?;
    let canonical_parent = parent
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", parent.display()))?;
    if !canonical_parent.starts_with(&allowed_root) {
        bail!("refusing to stage repo outside .architect-mcp/foundry");
    }
    if let Ok(metadata) = fs::symlink_metadata(root) {
        if metadata.file_type().is_symlink() {
            bail!("refusing to clean symlinked foundry path");
        }
        let canonical = root
            .canonicalize()
            .with_context(|| format!("failed to inspect {}", root.display()))?;
        if !canonical.starts_with(&allowed_root) {
            bail!("refusing to clean foundry path outside .architect-mcp/foundry");
        }
        fs::remove_dir_all(root).with_context(|| format!("failed to clean {}", root.display()))?;
    }
    Ok(())
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute() {
        bail!("foundry artifact path must be relative: {relative}");
    }
    let mut out = root.to_path_buf();
    for component in path.components() {
        match component {
            Component::Normal(value) => out.push(value),
            _ => bail!("foundry artifact path contains unsafe component: {relative}"),
        }
    }
    Ok(out)
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn git {}", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }
    bail!(
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    )
}
