use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::interactive::InteractiveWorkflowEngine;
use crate::promotion_smoke::apply_reported_command;
use crate::promotion_smoke_report::{
    PromotionSmokeCommandReport, PromotionSmokeVerificationReport,
};
use crate::session::TuiSession;

pub(crate) async fn run_required_verification(
    engine: &mut InteractiveWorkflowEngine,
    commands: &mut Vec<PromotionSmokeCommandReport>,
) -> Result<Vec<PromotionSmokeVerificationReport>> {
    let session = engine.active()?.clone();
    let worktree = session
        .worktree
        .clone()
        .context("adapter worktree missing after run adapter")?;
    let checks = if session.required_verification.is_empty() {
        vec!["npm test".to_string(), "review_repo_structure".to_string()]
    } else {
        session.required_verification.clone()
    };
    let mut reports = Vec::new();
    for check in checks {
        let report = verify_check(&session, &worktree, &check)?;
        let command = format!("record verification {check}={}", report.status);
        apply_reported_command(engine, commands, &command).await?;
        reports.push(report);
    }
    if reports.iter().any(|report| report.status != "passed") {
        anyhow::bail!("one or more promotion smoke verification checks failed");
    }
    Ok(reports)
}

fn verify_check(
    session: &TuiSession,
    worktree: &Path,
    check: &str,
) -> Result<PromotionSmokeVerificationReport> {
    match check {
        "review_repo_structure" => Ok(gate_verification(
            check,
            session.gates.contains_key("review_repo_structure"),
            "review_repo_structure gate exists after adapter review",
        )),
        "architecture contract validates" => Ok(gate_verification(
            check,
            session.gates.contains_key("create_pre_edit_contract"),
            "create_pre_edit_contract gate exists",
        )),
        "npm test" | "npm run test" | "npm run build" => run_shell_verification(worktree, check),
        _ => Ok(PromotionSmokeVerificationReport {
            check: check.to_string(),
            status: "failed".to_string(),
            command: None,
            output: "unsupported promotion-smoke verification check".to_string(),
        }),
    }
}

fn gate_verification(check: &str, passed: bool, output: &str) -> PromotionSmokeVerificationReport {
    PromotionSmokeVerificationReport {
        check: check.to_string(),
        status: if passed { "passed" } else { "failed" }.to_string(),
        command: None,
        output: output.to_string(),
    }
}

fn run_shell_verification(
    worktree: &Path,
    command: &str,
) -> Result<PromotionSmokeVerificationReport> {
    let output = shell_command(worktree, command)
        .with_context(|| format!("failed to run verification command '{command}'"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(PromotionSmokeVerificationReport {
        check: command.to_string(),
        status: if output.status.success() {
            "passed"
        } else {
            "failed"
        }
        .to_string(),
        command: Some(command.to_string()),
        output: text.trim().chars().take(4096).collect(),
    })
}

fn shell_command(worktree: &Path, command: &str) -> Result<std::process::Output> {
    let mut shell = if cfg!(windows) {
        let mut command_builder = Command::new("cmd");
        command_builder.args(["/C", command]);
        command_builder
    } else {
        let mut command_builder = Command::new("sh");
        command_builder.args(["-c", command]);
        command_builder
    };
    shell.current_dir(worktree).output().with_context(|| {
        format!(
            "failed to spawn shell verification in {}",
            worktree.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_promotion_smoke_verification_fails_closed() {
        let session = TuiSession::new("build", "codex");
        let temp = tempfile::tempdir().expect("tempdir");
        let report = verify_check(&session, temp.path(), "publish release").expect("report");
        assert_eq!(report.status, "failed");
        assert!(report.command.is_none());
    }
}
