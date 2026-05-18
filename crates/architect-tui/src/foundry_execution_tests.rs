use std::path::PathBuf;

use crate::foundry::{FoundryPullRequestPlan, RepoFoundryPlan, RepoVisibility};
use crate::foundry_execution::{
    FoundryCommandOutput, ensure_staged_repo_exists, execute_repo_foundry_plan_with_runner,
    foundry_command_specs,
};
use crate::foundry_stage::{RepoFoundryStage, expected_foundry_stage_path};

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

#[test]
fn staged_repo_validation_rejects_tampered_stage_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let tampered = temp.path().join("source-checkout");
    std::fs::create_dir_all(tampered.join(".git")).expect("tampered git");
    std::fs::create_dir_all(tampered.join("docs")).expect("tampered docs");
    let pr_body_path = tampered.join("docs/first-pr-draft.md");
    std::fs::write(&pr_body_path, "body").expect("pr body");
    let plan = sample_plan();
    let stage = RepoFoundryStage {
        path: tampered,
        bootstrap_branch: "architect/bootstrap".to_string(),
        artifacts_written: 1,
        pr_body_path,
    };

    let error =
        ensure_staged_repo_exists(&workspace, "session-id", &plan, &stage).expect_err("invalid");

    assert!(
        error
            .to_string()
            .contains("run foundry stage before foundry create --execute")
            || error
                .to_string()
                .contains("refusing to execute unexpected foundry stage path"),
        "{error}"
    );
}

#[test]
fn staged_repo_validation_rejects_pr_body_outside_stage() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let plan = sample_plan();
    let stage_path =
        expected_foundry_stage_path(&workspace, "session-id", &plan.repo_name).expect("stage path");
    std::fs::create_dir_all(stage_path.join(".git")).expect("stage git");
    let pr_body_path = temp.path().join("outside-pr-body.md");
    std::fs::write(&pr_body_path, "body").expect("outside body");
    let stage = RepoFoundryStage {
        path: stage_path,
        bootstrap_branch: "architect/bootstrap".to_string(),
        artifacts_written: 1,
        pr_body_path,
    };

    let error =
        ensure_staged_repo_exists(&workspace, "session-id", &plan, &stage).expect_err("invalid");

    assert!(
        error
            .to_string()
            .contains("PR body outside staged foundry repo"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn staged_repo_validation_rejects_symlinked_session_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let foundry_root = workspace.join(".architect-mcp").join("foundry");
    std::fs::create_dir_all(&foundry_root).expect("foundry root");
    let outside_session = temp.path().join("outside-session");
    let plan = sample_plan();
    let outside_stage = outside_session.join(&plan.repo_name);
    std::fs::create_dir_all(outside_stage.join(".git")).expect("stage git");
    std::fs::create_dir_all(outside_stage.join("docs")).expect("stage docs");
    let pr_body_path = outside_stage.join("docs/first-pr-draft.md");
    std::fs::write(&pr_body_path, "body").expect("pr body");
    std::os::unix::fs::symlink(&outside_session, foundry_root.join("session-id"))
        .expect("symlink session");
    let stage_path =
        expected_foundry_stage_path(&workspace, "session-id", &plan.repo_name).expect("stage");
    let stage = RepoFoundryStage {
        path: stage_path,
        bootstrap_branch: "architect/bootstrap".to_string(),
        artifacts_written: 1,
        pr_body_path,
    };

    let error =
        ensure_staged_repo_exists(&workspace, "session-id", &plan, &stage).expect_err("invalid");

    assert!(
        error
            .to_string()
            .contains("run foundry stage before foundry create --execute"),
        "{error}"
    );
    assert!(
        error
            .chain()
            .any(|cause| cause.to_string().contains("symlinked foundry session")),
        "{error:#}"
    );
}

#[test]
fn staged_repo_validation_accepts_expected_stage_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let plan = sample_plan();
    let stage_path =
        expected_foundry_stage_path(&workspace, "session-id", &plan.repo_name).expect("stage path");
    std::fs::create_dir_all(stage_path.join(".git")).expect("stage git");
    std::fs::create_dir_all(stage_path.join("docs")).expect("stage docs");
    let pr_body_path = stage_path.join("docs/first-pr-draft.md");
    std::fs::write(&pr_body_path, "body").expect("pr body");
    let stage = RepoFoundryStage {
        path: stage_path,
        bootstrap_branch: "architect/bootstrap".to_string(),
        artifacts_written: 1,
        pr_body_path,
    };

    ensure_staged_repo_exists(&workspace, "session-id", &plan, &stage).expect("valid stage");
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
