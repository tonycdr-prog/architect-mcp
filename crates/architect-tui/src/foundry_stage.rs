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
    let root = expected_foundry_stage_path(workspace, &session.id, &plan.repo_name)?;
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

pub(crate) fn expected_foundry_stage_path(
    workspace: &Path,
    session_id: &str,
    repo_name: &str,
) -> Result<PathBuf> {
    let session_id = validated_path_component("foundry session id", session_id)?;
    let repo_name = validated_path_component("foundry repo name", repo_name)?;
    Ok(workspace
        .join(".architect-mcp")
        .join("foundry")
        .join(session_id)
        .join(repo_name))
}

pub(crate) fn canonical_existing_foundry_stage_path(
    workspace: &Path,
    session_id: &str,
    repo_name: &str,
) -> Result<PathBuf> {
    let expected_path = expected_foundry_stage_path(workspace, session_id, repo_name)?;
    reject_symlinked_existing_path(
        &workspace.join(".architect-mcp"),
        ".architect-mcp directory",
    )?;
    reject_symlinked_existing_path(
        &workspace.join(".architect-mcp").join("foundry"),
        ".architect-mcp/foundry directory",
    )?;
    let parent = expected_path
        .parent()
        .context("foundry stage path must have a parent directory")?;
    reject_symlinked_existing_path(parent, "foundry session directory")?;
    reject_symlinked_existing_path(&expected_path, "foundry stage path")?;
    expected_path
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", expected_path.display()))
}

fn prepare_clean_stage_root(workspace: &Path, root: &Path) -> Result<()> {
    let allowed_root = prepare_foundry_root(workspace)?;
    let parent = root
        .parent()
        .context("foundry stage path must have a parent directory")?;
    let canonical_parent = prepare_stage_parent(parent)?;
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

fn prepare_foundry_root(workspace: &Path) -> Result<PathBuf> {
    let app_root = workspace.join(".architect-mcp");
    prepare_non_symlinked_directory(&app_root, ".architect-mcp directory")?;
    let foundry_root = app_root.join("foundry");
    prepare_non_symlinked_directory(&foundry_root, ".architect-mcp/foundry directory")?;
    let canonical_app_root = app_root
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", app_root.display()))?;
    let canonical_foundry_root = foundry_root
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", foundry_root.display()))?;
    if !canonical_foundry_root.starts_with(&canonical_app_root) {
        bail!("refusing to use foundry root outside .architect-mcp");
    }
    Ok(canonical_foundry_root)
}

fn prepare_non_symlinked_directory(path: &Path, label: &str) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            bail!("refusing to use symlinked {label}");
        }
        if !metadata.is_dir() {
            bail!("{label} is not a directory");
        }
        return Ok(());
    }
    fs::create_dir_all(path).with_context(|| format!("failed to prepare {}", path.display()))?;
    reject_symlinked_existing_path(path, label)?;
    Ok(())
}

fn prepare_stage_parent(parent: &Path) -> Result<PathBuf> {
    if let Ok(metadata) = fs::symlink_metadata(parent) {
        if metadata.file_type().is_symlink() {
            bail!("refusing to prepare symlinked foundry parent");
        }
        return parent
            .canonicalize()
            .with_context(|| format!("failed to inspect {}", parent.display()));
    }
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to prepare {}", parent.display()))?;
    let metadata = fs::symlink_metadata(parent)
        .with_context(|| format!("failed to inspect {}", parent.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("refusing to prepare symlinked foundry parent");
    }
    parent
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", parent.display()))
}

fn reject_symlinked_existing_path(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("refusing to use symlinked {label}");
    }
    Ok(())
}

fn validated_path_component<'a>(label: &str, value: &'a str) -> Result<&'a str> {
    if value.is_empty() {
        bail!("{label} cannot be blank");
    }
    let path = Path::new(value);
    if path.is_absolute() {
        bail!("{label} must be a single relative path component");
    }
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(value),
        _ => bail!("{label} contains unsafe path component"),
    }
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
