use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::foundry::{RepoFoundryPlan, repo_target};
use crate::foundry_stage::RepoFoundryStage;

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

pub fn ensure_staged_repo_exists(stage: &RepoFoundryStage) -> Result<()> {
    if !Path::new(&stage.path).join(".git").exists() {
        bail!("run foundry stage before foundry create --execute");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundry::{FoundryPullRequestPlan, RepoFoundryPlan, RepoVisibility};
    use crate::foundry_stage::RepoFoundryStage;

    #[test]
    fn command_specs_are_private_and_pr_based() {
        let plan = sample_plan();
        let stage = sample_stage();
        let commands = foundry_command_specs(&plan, &stage);
        assert_eq!(commands[0].program, "gh");
        assert_eq!(
            commands[0].args[0..4],
            ["repo", "create", "owner/app", "--private"]
        );
        assert!(commands[0].args.contains(&"--source".to_string()));
        assert!(
            commands[2]
                .args
                .contains(&"architect/bootstrap".to_string())
        );
        assert_eq!(commands[3].args[0..4], ["-R", "owner/app", "pr", "create"]);
        assert!(commands[3].args.contains(&"--draft".to_string()));
    }

    #[test]
    fn execution_uses_runner_and_stops_on_failure() {
        let plan = sample_plan();
        let stage = sample_stage();
        let result = execute_repo_foundry_plan_with_runner(&plan, &stage, |spec| {
            Ok(FoundryCommandOutput {
                exit_code: i32::from(spec.program == "git"),
                stdout: "ok".to_string(),
                stderr: "nope".to_string(),
            })
        })
        .expect("execution result");
        assert_eq!(result.status, "failed");
        assert_eq!(result.commands.len(), 2);
    }

    fn sample_plan() -> RepoFoundryPlan {
        RepoFoundryPlan {
            repo_name: "app".to_string(),
            owner: Some("owner".to_string()),
            visibility: RepoVisibility::Private,
            default_branch: "main".to_string(),
            artifacts: Vec::new(),
            verification: vec!["npm test".to_string()],
            first_pr: FoundryPullRequestPlan {
                title: "Bootstrap".to_string(),
                body_sections: vec!["Evidence".to_string()],
                draft: true,
            },
            mutation_commands: Vec::new(),
            approval_required_for: Vec::new(),
        }
    }

    fn sample_stage() -> RepoFoundryStage {
        RepoFoundryStage {
            path: PathBuf::from("/tmp/stage"),
            bootstrap_branch: "architect/bootstrap".to_string(),
            artifacts_written: 1,
            pr_body_path: PathBuf::from("/tmp/stage/docs/first-pr-draft.md"),
        }
    }
}
