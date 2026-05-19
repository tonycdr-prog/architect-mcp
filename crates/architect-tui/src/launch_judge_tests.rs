use crate::launch_judge_report::{
    LaunchJudgeCheckStatus, LaunchJudgeCommandEvidence, LaunchJudgeResult, judge_result,
    release_check_status, skipped_command,
};

#[test]
fn judge_result_requires_no_blockers_or_warnings_for_go() {
    assert_eq!(judge_result(&[], &[]), LaunchJudgeResult::Go);
    assert_eq!(
        judge_result(&[], &["manual terminal evidence missing".to_string()]),
        LaunchJudgeResult::ConditionalGo
    );
    assert_eq!(
        judge_result(&["release check failed".to_string()], &[]),
        LaunchJudgeResult::NoGo
    );
}

#[test]
fn skipped_release_check_is_conditional_not_go() {
    let check = release_check_status(&skipped_command("npm run release:check"));

    assert_eq!(check.status, LaunchJudgeCheckStatus::Skipped);
    assert_eq!(
        check.next_action.as_deref(),
        Some("rerun with --run-release-check before release-sensitive go")
    );
}

#[test]
fn failed_release_check_is_no_go_blocker() {
    let evidence = LaunchJudgeCommandEvidence {
        command: "npm run release:check".to_string(),
        attempted: true,
        ok: false,
        exit_code: Some(1),
        stdout_tail: Vec::new(),
        stderr_tail: vec!["failed".to_string()],
        error: None,
    };
    let check = release_check_status(&evidence);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
}
