use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::foundry::{RepoFoundryPlan, repo_target};
use crate::foundry_stage::{RepoFoundryStage, canonical_existing_foundry_stage_path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFoundryExecution {
    pub target: String,
    pub status: String,
    pub commands: Vec<FoundryCommandResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryCommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryCommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

pub fn execute_repo_foundry_plan(
    plan: &RepoFoundryPlan,
    stage: &RepoFoundryStage,
) -> Result<RepoFoundryExecution> {
    execute_repo_foundry_plan_with_runner(plan, stage, run_command)
}

pub fn execute_repo_foundry_plan_with_runner<F>(
    plan: &RepoFoundryPlan,
    stage: &RepoFoundryStage,
    mut runner: F,
) -> Result<RepoFoundryExecution>
where
    F: FnMut(&FoundryCommandSpec) -> Result<FoundryCommandOutput>,
{
    let target = repo_target(plan.owner.as_deref(), &plan.repo_name);
    let specs = foundry_command_specs(plan, stage);
    let mut commands = Vec::new();
    for spec in specs {
        let output = runner(&spec).with_context(|| format!("failed to run {}", spec.display()))?;
        let result = FoundryCommandResult {
            command: spec.display(),
            exit_code: output.exit_code,
            stdout_tail: tail(&output.stdout),
            stderr_tail: tail(&output.stderr),
        };
        if output.exit_code != 0 {
            commands.push(result);
            return Ok(RepoFoundryExecution {
                target,
                status: "failed".to_string(),
                commands,
            });
        }
        commands.push(result);
    }
    Ok(RepoFoundryExecution {
        target,
        status: "passed".to_string(),
        commands,
    })
}

pub fn foundry_command_specs(
    plan: &RepoFoundryPlan,
    stage: &RepoFoundryStage,
) -> Vec<FoundryCommandSpec> {
    let target = repo_target(plan.owner.as_deref(), &plan.repo_name);
    let stage_path = stage.path.clone();
    vec![
        FoundryCommandSpec {
            program: "gh".to_string(),
            args: vec![
                "repo".to_string(),
                "create".to_string(),
                target.clone(),
                "--private".to_string(),
                "--source".to_string(),
                stage_path.display().to_string(),
                "--remote".to_string(),
                "origin".to_string(),
            ],
            cwd: None,
        },
        FoundryCommandSpec {
            program: "git".to_string(),
            args: vec![
                "-C".to_string(),
                stage_path.display().to_string(),
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                "main".to_string(),
            ],
            cwd: None,
        },
        FoundryCommandSpec {
            program: "git".to_string(),
            args: vec![
                "-C".to_string(),
                stage_path.display().to_string(),
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                "architect/bootstrap".to_string(),
            ],
            cwd: None,
        },
        FoundryCommandSpec {
            program: "gh".to_string(),
            args: vec![
                "-R".to_string(),
                target,
                "pr".to_string(),
                "create".to_string(),
                "--draft".to_string(),
                "--base".to_string(),
                "main".to_string(),
                "--head".to_string(),
                "architect/bootstrap".to_string(),
                "--title".to_string(),
                plan.first_pr.title.clone(),
                "--body-file".to_string(),
                stage.pr_body_path.display().to_string(),
            ],
            cwd: None,
        },
    ]
}

fn run_command(spec: &FoundryCommandSpec) -> Result<FoundryCommandOutput> {
    let mut command = Command::new(&spec.program);
    command.args(&spec.args);
    if let Some(cwd) = &spec.cwd {
        command.current_dir(cwd);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to spawn {}", spec.display()))?;
    Ok(FoundryCommandOutput {
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

impl FoundryCommandSpec {
    pub fn display(&self) -> String {
        let mut parts = vec![self.program.clone()];
        parts.extend(self.args.iter().map(|arg| shell_escape(arg)));
        parts.join(" ")
    }
}

fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "-_./:=@".contains(ch))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn tail(value: &str) -> String {
    const MAX: usize = 4096;
    if value.len() <= MAX {
        return value.to_string();
    }
    let start = value
        .char_indices()
        .map(|(idx, _)| idx)
        .find(|idx| value.len() - idx <= MAX)
        .unwrap_or(value.len());
    format!("[truncated]\n{}", &value[start..])
}

pub fn ensure_staged_repo_exists(
    workspace: &Path,
    session_id: &str,
    plan: &RepoFoundryPlan,
    stage: &RepoFoundryStage,
) -> Result<()> {
    let metadata = fs::symlink_metadata(&stage.path)
        .with_context(|| format!("failed to inspect {}", stage.path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("refusing to execute symlinked foundry stage path");
    }
    if !metadata.is_dir() {
        bail!("foundry stage path is not a directory");
    }
    let canonical_stage = stage
        .path
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", stage.path.display()))?;
    let canonical_expected =
        canonical_existing_foundry_stage_path(workspace, session_id, &plan.repo_name)
            .with_context(|| "run foundry stage before foundry create --execute")?;
    if canonical_stage != canonical_expected {
        bail!("refusing to execute unexpected foundry stage path");
    }
    let git_path = canonical_stage.join(".git");
    if !git_path.exists() {
        bail!("run foundry stage before foundry create --execute");
    }
    validate_pr_body_path(&canonical_stage, &stage.pr_body_path)?;
    Ok(())
}

fn validate_pr_body_path(canonical_stage: &Path, pr_body_path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(pr_body_path)
        .with_context(|| format!("failed to inspect {}", pr_body_path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("refusing to execute with symlinked foundry PR body path");
    }
    if !metadata.is_file() {
        bail!("foundry PR body path is not a file");
    }
    let canonical_pr_body = pr_body_path
        .canonicalize()
        .with_context(|| format!("failed to inspect {}", pr_body_path.display()))?;
    if !canonical_pr_body.starts_with(canonical_stage) {
        bail!("refusing to execute with PR body outside staged foundry repo");
    }
    Ok(())
}
