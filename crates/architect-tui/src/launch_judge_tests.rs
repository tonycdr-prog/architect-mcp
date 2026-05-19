use std::io::Write;

use crate::launch_judge_evidence::read_terminal_evidence;
use crate::launch_judge_report::{
    LaunchJudgeCheckStatus, LaunchJudgeCommandEvidence, LaunchJudgeResult,
    LaunchJudgeTerminalEvidenceStatus, judge_result, release_check_status, skipped_command,
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

#[test]
fn missing_terminal_evidence_is_conditional() {
    let (summary, check) = read_terminal_evidence(&[]);

    assert!(!summary.supplied);
    assert_eq!(summary.reports.len(), 0);
    assert_eq!(check.status, LaunchJudgeCheckStatus::Warning);
    assert!(
        check
            .detail
            .contains("manual Windows and Linux terminal evidence")
    );
}

#[test]
fn complete_terminal_evidence_passes() {
    let evidence = r##"{
      "schemaVersion": 1,
      "reports": [
        {
          "platform": "linux",
          "status": "passed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "architect-mcp-tui help, adapters JSON, and gate-only JSONL passed",
          "collectedAt": "2026-05-17",
          "notes": "summary only"
        },
        {
          "platform": "windows",
          "status": "passed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "architect-mcp-tui help, adapters JSON, and gate-only JSONL passed",
          "collectedAt": "2026-05-17",
          "notes": "summary only"
        }
      ]
    }"##;

    let file = write_evidence(evidence);
    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert!(summary.supplied);
    assert_eq!(summary.issues.len(), 0);
    assert_eq!(summary.reports.len(), 2);
    assert_eq!(
        summary.reports[0].status,
        LaunchJudgeTerminalEvidenceStatus::Passed
    );
    assert_eq!(check.status, LaunchJudgeCheckStatus::Passed);
}

#[test]
fn multiple_terminal_evidence_files_are_merged() {
    let linux = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "source": "issue #136 linux public-safe summary",
              "commandSummary": "architect-mcp-tui terminal-evidence --json passed on linux"
            }
          ]
        }"##,
    );
    let windows = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "windows",
              "status": "passed",
              "source": "issue #136 windows public-safe summary",
              "commandSummary": "architect-mcp-tui terminal-evidence --json passed on windows"
            }
          ]
        }"##,
    );

    let (summary, check) =
        read_terminal_evidence(&[linux.path().to_path_buf(), windows.path().to_path_buf()]);

    assert!(summary.supplied);
    assert_eq!(
        summary
            .source_path
            .as_deref()
            .unwrap_or_default()
            .matches('/')
            .count(),
        0
    );
    assert_eq!(summary.issues.len(), 0);
    assert_eq!(summary.reports.len(), 2);
    assert_eq!(check.status, LaunchJudgeCheckStatus::Passed);
}

#[test]
fn missing_platform_terminal_evidence_stays_conditional() {
    let evidence = r##"{
      "schemaVersion": 1,
      "reports": [
        {
          "platform": "linux",
          "status": "passed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "architect-mcp-tui help, adapters JSON, and gate-only JSONL passed"
        }
      ]
    }"##;

    let file = write_evidence(evidence);
    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Warning);
    assert!(summary.issues.iter().any(|issue| issue.contains("windows")));
}

#[test]
fn failed_terminal_evidence_is_no_go() {
    let evidence = r##"{
      "schemaVersion": 1,
      "reports": [
        {
          "platform": "linux",
          "status": "failed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "launch command failed before drawing"
        },
        {
          "platform": "windows",
          "status": "passed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "architect-mcp-tui help, adapters JSON, and gate-only JSONL passed"
        }
      ]
    }"##;

    let file = write_evidence(evidence);
    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(summary.issues.iter().any(|issue| issue.contains("failed")));
}

#[test]
fn unsafe_terminal_evidence_fails_closed() {
    let evidence = r##"{
      "schemaVersion": 1,
      "reports": [
        {
          "platform": "linux",
          "status": "passed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "architect-mcp-tui help passed",
          "stdout": "/Users/example/private/path"
        },
        {
          "platform": "windows",
          "status": "passed",
          "source": "issue #136 public-safe summary",
          "commandSummary": "architect-mcp-tui help passed"
        }
      ]
    }"##;

    let file = write_evidence(evidence);
    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("unsafe") || issue.contains("local-path"))
    );
}

fn write_evidence(contents: &str) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().expect("create terminal evidence fixture");
    file.write_all(contents.as_bytes())
        .expect("write terminal evidence fixture");
    file
}
