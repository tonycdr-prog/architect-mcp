use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use crate::config::TuiConfig;
use crate::governance_audit::{GovernanceAuditOptions, build_governance_audit_report};
use crate::launch_judge_evidence::read_terminal_evidence;
use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeCommandEvidence, LaunchJudgeReport,
    LaunchJudgeResult, build_report, check, print_text_report, skipped_command, tail_lines,
};
use crate::smoke::{SmokeOptions, build_smoke_report};

#[derive(Debug, Clone)]
pub struct LaunchJudgeOptions {
    pub json: bool,
    pub skip_mcp: bool,
    pub skip_smoke: bool,
    pub run_release_check: bool,
    pub require_clean_git: bool,
    pub max_files: usize,
    pub terminal_evidence: Option<PathBuf>,
}

pub async fn run_launch_judge(
    workspace: PathBuf,
    config: TuiConfig,
    options: LaunchJudgeOptions,
) -> Result<()> {
    let report = build_launch_judge_report(workspace, config, &options).await;
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }
    if report.result == LaunchJudgeResult::NoGo {
        anyhow::bail!("launch judge result is no-go");
    }
    Ok(())
}

pub async fn build_launch_judge_report(
    workspace: PathBuf,
    config: TuiConfig,
    options: &LaunchJudgeOptions,
) -> LaunchJudgeReport {
    let governance_audit = build_governance_audit_report(
        workspace.clone(),
        config.clone(),
        &GovernanceAuditOptions {
            json: true,
            skip_mcp: options.skip_mcp,
            max_files: options.max_files,
        },
    )
    .await;

    let smoke = if options.skip_smoke {
        None
    } else {
        Some(
            build_smoke_report(
                workspace.clone(),
                config,
                SmokeOptions {
                    json: true,
                    prompt: SmokeOptions::DEFAULT_PROMPT.to_string(),
                    skip_gate: false,
                },
            )
            .await,
        )
    };

    let release_check = if options.run_release_check {
        run_release_check(&workspace)
    } else {
        skipped_command("npm run release:check")
    };
    let git_clean = git_clean_check(&workspace, options.require_clean_git);
    let (terminal_evidence, terminal_evidence_check) =
        read_terminal_evidence(options.terminal_evidence.as_deref());
    build_report(
        &workspace,
        governance_audit,
        smoke,
        release_check,
        git_clean,
        terminal_evidence,
        terminal_evidence_check,
    )
}

fn git_clean_check(workspace: &Path, required: bool) -> LaunchJudgeCheck {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["status", "--short"])
        .output();
    let Ok(output) = output else {
        return git_status_problem(required, "git status could not be read");
    };
    if !output.status.success() {
        return git_status_problem(required, "git status returned a non-zero exit");
    }
    let status = String::from_utf8_lossy(&output.stdout);
    if status.trim().is_empty() {
        check(
            "git worktree",
            LaunchJudgeCheckStatus::Passed,
            "git worktree is clean",
            None,
        )
    } else {
        check(
            "git worktree",
            if required {
                LaunchJudgeCheckStatus::Failed
            } else {
                LaunchJudgeCheckStatus::Warning
            },
            "git worktree has uncommitted changes",
            Some("commit, stash, or intentionally document local changes before launch"),
        )
    }
}

fn git_status_problem(required: bool, detail: &str) -> LaunchJudgeCheck {
    check(
        "git worktree",
        if required {
            LaunchJudgeCheckStatus::Failed
        } else {
            LaunchJudgeCheckStatus::Warning
        },
        detail,
        Some("inspect the workspace git state manually"),
    )
}

fn run_release_check(workspace: &Path) -> LaunchJudgeCommandEvidence {
    let command = "npm run release:check";
    let output = Command::new("npm")
        .arg("run")
        .arg("release:check")
        .current_dir(workspace)
        .output();
    match output {
        Ok(output) => LaunchJudgeCommandEvidence {
            command: command.to_string(),
            attempted: true,
            ok: output.status.success(),
            exit_code: output.status.code(),
            stdout_tail: tail_lines(&String::from_utf8_lossy(&output.stdout), 20),
            stderr_tail: tail_lines(&String::from_utf8_lossy(&output.stderr), 20),
            error: None,
        },
        Err(error) => LaunchJudgeCommandEvidence {
            command: command.to_string(),
            attempted: true,
            ok: false,
            exit_code: None,
            stdout_tail: Vec::new(),
            stderr_tail: Vec::new(),
            error: Some(error.to_string()),
        },
    }
}
