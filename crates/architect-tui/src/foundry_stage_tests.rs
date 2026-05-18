use std::fs;

use crate::foundry::{FoundryPullRequestPlan, RepoFoundryPlan, RepoVisibility};
use crate::foundry_stage::stage_repo_foundry_plan;
use crate::session::TuiSession;

#[test]
fn staging_rejects_path_like_session_id_before_creating_derived_dirs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("workspace");
    let session = sample_session("../../../escape", "launchpad");

    let error = stage_repo_foundry_plan(&workspace, &session).expect_err("invalid session id");

    assert!(
        error
            .to_string()
            .contains("foundry session id contains unsafe path component"),
        "{error}"
    );
    assert!(!temp.path().join("escape").exists());
    assert!(!workspace.join(".architect-mcp").join("escape").exists());
    assert!(!workspace.join(".architect-mcp").join("foundry").exists());
}

#[test]
fn staging_rejects_tampered_path_like_repo_name_before_creating_derived_dirs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("workspace");
    let session = sample_session("session-id", "../escape");

    let error = stage_repo_foundry_plan(&workspace, &session).expect_err("invalid repo name");

    assert!(
        error
            .to_string()
            .contains("foundry repo name contains unsafe path component"),
        "{error}"
    );
    assert!(!workspace.join(".architect-mcp").join("escape").exists());
    assert!(!workspace.join(".architect-mcp").join("foundry").exists());
}

#[cfg(unix)]
#[test]
fn staging_rejects_symlinked_foundry_root_before_creating_stage_dirs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join(".architect-mcp")).expect("workspace metadata");
    let outside = temp.path().join("outside-foundry");
    fs::create_dir_all(&outside).expect("outside foundry");
    std::os::unix::fs::symlink(&outside, workspace.join(".architect-mcp").join("foundry"))
        .expect("symlink foundry");
    let session = sample_session("session-id", "launchpad");

    let error = stage_repo_foundry_plan(&workspace, &session).expect_err("symlinked foundry");

    assert!(
        error
            .to_string()
            .contains("symlinked .architect-mcp/foundry directory"),
        "{error}"
    );
    assert!(!outside.join("session-id").exists());
}

fn sample_session(session_id: &str, repo_name: &str) -> TuiSession {
    let mut session = TuiSession::new("ready app", "codex");
    session.id = session_id.to_string();
    session.foundry_plan = Some(RepoFoundryPlan {
        repo_name: repo_name.to_string(),
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
    });
    session
}
