use serde_json::json;

use crate::issue_terminal_evidence::{
    IssueTerminalEvidenceBlockStatus, build_issue_terminal_evidence_report_from_value,
    extract_json_blocks,
};
use crate::issue_terminal_evidence_source::issue_view_args;
use crate::launch_judge_report::LaunchJudgeResult;

#[test]
fn extracts_fenced_and_standalone_terminal_evidence_json() {
    let blocks = extract_json_blocks(
        r#"
Some note.

```json
{"schemaVersion":1,"reports":[]}
```
"#,
    );
    assert_eq!(blocks, vec![r#"{"schemaVersion":1,"reports":[]}"#]);

    let standalone = extract_json_blocks(r#"{"schemaVersion":1,"reports":[]}"#);
    assert_eq!(standalone, vec![r#"{"schemaVersion":1,"reports":[]}"#]);

    let non_json_fence = extract_json_blocks("```bash\necho test\n```");
    assert!(non_json_fence.is_empty());

    let standalone_missing_schema = extract_json_blocks(r#"{"reports":[]}"#);
    assert!(standalone_missing_schema.is_empty());
}

#[test]
fn collector_uses_read_only_issue_view_command() {
    assert_eq!(
        issue_view_args(136),
        vec![
            "issue".to_string(),
            "view".to_string(),
            "136".to_string(),
            "--json".to_string(),
            "number,title,url,body,comments".to_string(),
        ]
    );
}

#[test]
fn collector_passes_complete_linux_and_windows_issue_evidence() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [
            {
                "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux public-safe summary\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on linux\",\"collectedAt\":\"2026-05-17\",\"notes\":\"summary only, no raw logs\"}]}\n```"
            },
            {
                "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"issue #136 windows public-safe summary\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed on windows\",\"collectedAt\":\"2026-05-17\",\"notes\":\"summary only, no raw logs\"}]}\n```"
            }
        ]
    });

    let report = build_issue_terminal_evidence_report_from_value(
        Some("example/repo".to_string()),
        136,
        &value,
    );

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(report.extracted_blocks.len(), 2);
    assert_eq!(report.terminal_evidence.reports.len(), 2);
    assert!(report.findings.is_empty());
    assert_eq!(
        report
            .merged_evidence
            .as_ref()
            .expect("merged evidence")
            .reports
            .len(),
        2
    );
}

#[test]
fn collector_warns_when_issue_has_no_terminal_evidence() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "Please run terminal QA.",
        "comments": [{"body": "No JSON yet."}]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert!(report.extracted_blocks.is_empty());
    assert!(!report.terminal_evidence.supplied);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("does not contain terminal-evidence"))
    );
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("Linux and Windows testers"))
    );
    assert!(
        report
            .next_actions
            .iter()
            .any(|action| action.contains("terminal-evidence --markdown"))
    );
}

#[test]
fn collector_warns_on_template_placeholder_evidence() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [
            {
                "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"REPLACE with public issue or PR link for this real terminal run\",\"commandSummary\":\"REPLACE with commands that passed or failed on this real machine\",\"notes\":\"REPLACE with rendering, mouse, resize, install, or checksum notes; keep raw logs local\"},{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"issue #136 public-safe terminal QA report\",\"commandSummary\":\"architect-mcp-tui terminal-evidence --json passed; help, adapter summary, and gate-only run were summarized\",\"notes\":\"summary only, no raw logs\"}]}\n```"
            }
        ]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert_eq!(report.extracted_blocks.len(), 1);
    assert_eq!(report.terminal_evidence.reports.len(), 2);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("template placeholder"))
    );
}

#[test]
fn collector_rejects_unsafe_evidence_blocks() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [
            {
                "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"source\":\"issue #136\",\"commandSummary\":\"passed\",\"notes\":\"/Users/example/private\"}]}\n```"
            }
        ]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(
        report.extracted_blocks[0].status,
        IssueTerminalEvidenceBlockStatus::Rejected
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("secret-shaped or local-path"))
    );
}

#[test]
fn collector_rejects_malformed_json_blocks() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [
            {
                "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\"\n```"
            }
        ]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(
        report.extracted_blocks[0].status,
        IssueTerminalEvidenceBlockStatus::Rejected
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("JSON could not be parsed"))
    );
}

#[test]
fn collector_warns_on_duplicate_or_missing_platforms() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [
            {
                "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux 1\",\"commandSummary\":\"passed\"},{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux 2\",\"commandSummary\":\"passed\"}]}\n```"
            }
        ]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("duplicated"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("windows"))
    );
}
