use crate::launch_judge_report::{LaunchJudgeCheckStatus, release_check_status, skipped_command};

use super::launch_judge_command::tail_lines;

#[test]
fn skipped_release_check_is_conditional_not_go() {
    let evidence = skipped_command("npm run release:check");
    let check = release_check_status(&evidence);

    assert!(!evidence.attempted);
    assert_eq!(evidence.ok, None);
    assert!(
        !serde_json::to_string(&evidence)
            .expect("serialize skipped evidence")
            .contains("\"ok\"")
    );
    assert_eq!(check.status, LaunchJudgeCheckStatus::Skipped);
    assert_eq!(
        check.next_action.as_deref(),
        Some("rerun with --run-release-check before release-sensitive go")
    );
}

#[test]
fn tail_lines_truncates_each_large_line() {
    let text = format!("short\n{}\nlast", "x".repeat(900));
    let tail = tail_lines(&text, 3);

    assert_eq!(tail.len(), 3);
    assert_eq!(tail[0], "short");
    assert_eq!(tail[1].chars().count(), 811);
    assert!(tail[1].ends_with("[truncated]"));
    assert_eq!(tail[2], "last");
}
