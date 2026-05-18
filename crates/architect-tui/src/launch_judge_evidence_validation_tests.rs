use std::io::Write;

use crate::launch_judge_evidence::read_terminal_evidence;
use crate::launch_judge_report::LaunchJudgeCheckStatus;

#[test]
fn missing_schema_version_fails_closed() {
    let file = write_evidence(
        r##"{
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux"
            },
            {
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows"
            }
          ]
        }"##,
    );

    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(summary.issues.iter().any(|issue| issue.contains("schema")));
}

#[test]
fn invalid_schema_version_fails_closed() {
    let file = write_evidence(&valid_reports_with(r#""schemaVersion": 2"#));

    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("schemaVersion must be 1"))
    );
}

#[test]
fn duplicate_platform_evidence_fails_closed() {
    let file = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux"
            },
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 duplicate linux summary",
              "commandSummary": "terminal evidence passed on linux"
            },
            {
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows"
            }
          ]
        }"##,
    );

    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("duplicated"))
    );
}

#[test]
fn unsupported_platform_evidence_fails_closed() {
    let file = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux"
            },
            {
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows"
            },
            {
              "platform": "macos",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 macOS summary",
              "commandSummary": "terminal evidence passed on macOS"
            }
          ]
        }"##,
    );

    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("must be linux or windows"))
    );
}

#[test]
fn unreadable_terminal_evidence_file_fails_closed() {
    let missing = std::path::PathBuf::from("target/does-not-exist/terminal-evidence.json");

    let (summary, check) = read_terminal_evidence(&[missing]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("could not be read"))
    );
}

#[test]
fn common_unix_absolute_paths_are_unsafe_terminal_evidence() {
    let file = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux from /var/folders/private-run"
            },
            {
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows",
              "notes": "cache was under /private/tmp/architect-mcp"
            }
          ]
        }"##,
    );

    let (summary, check) = read_terminal_evidence(&[file.path().to_path_buf()]);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("local-path content"))
    );
}

#[test]
fn user_path_detection_uses_boundaries_consistently() {
    let unsafe_file = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux from /Users/tony/repo"
            },
            {
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows from C:\\Users\\tony\\repo"
            }
          ]
        }"##,
    );

    let (unsafe_summary, unsafe_check) =
        read_terminal_evidence(&[unsafe_file.path().to_path_buf()]);

    assert_eq!(unsafe_check.status, LaunchJudgeCheckStatus::Failed);
    assert!(
        unsafe_summary
            .issues
            .iter()
            .any(|issue| issue.contains("local-path content"))
    );

    let safe_file = write_evidence(
        r##"{
          "schemaVersion": 1,
          "reports": [
            {
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux with abc/users/tony text"
            },
            {
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows with abcC:\\Users\\tony text"
            }
          ]
        }"##,
    );

    let (safe_summary, safe_check) = read_terminal_evidence(&[safe_file.path().to_path_buf()]);

    assert_eq!(safe_check.status, LaunchJudgeCheckStatus::Passed);
    assert!(safe_summary.issues.is_empty());
}

fn valid_reports_with(schema_line: &str) -> String {
    format!(
        r##"{{
          {schema_line},
          "reports": [
            {{
              "platform": "linux",
              "status": "passed",
              "environment": "local_terminal",
              "source": "issue #136 linux summary",
              "commandSummary": "terminal evidence passed on linux"
            }},
            {{
              "platform": "windows",
              "status": "passed",
              "environment": "vm_or_cloud_terminal",
              "source": "issue #136 windows summary",
              "commandSummary": "terminal evidence passed on windows"
            }}
          ]
        }}"##
    )
}

fn write_evidence(contents: &str) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().expect("create terminal evidence fixture");
    file.write_all(contents.as_bytes())
        .expect("write terminal evidence fixture");
    file
}
