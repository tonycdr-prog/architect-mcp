use serde_json::json;

use crate::issue_terminal_evidence::{
    IssueTerminalEvidenceBlockStatus, build_issue_terminal_evidence_report_from_value,
    extract_json_blocks,
};
use crate::issue_terminal_evidence_source::public_text;
use crate::launch_judge_report::LaunchJudgeResult;

#[test]
fn standalone_extraction_requires_whole_body_json() {
    let body = r#"
{"schemaVersion":1,"reports":[]}

```json
{"schemaVersion":1,"reports":[]}
```
"#;

    assert_eq!(
        extract_json_blocks(body),
        vec![r#"{"schemaVersion":1,"reports":[]}"#]
    );

    assert!(extract_json_blocks(r#"{"schemaVersion":1,"reports":[]} trailing text"#).is_empty());
}

#[test]
fn public_text_preserves_harmless_identifiers_and_redacts_real_tokens() {
    let text = public_text(
        "risk-assessment npm_test_helper https://github.com/example/users/tmp token=ghp_abcdefghijklmnopqrstuvwxyz1234567890 path=/tmp/work C:/Users/example/secret",
        500,
    );

    assert!(text.contains("risk-assessment"));
    assert!(text.contains("npm_test_helper"));
    assert!(text.contains("https://github.com/example/users/tmp"));
    assert!(!text.contains("ghp_abcdefghijklmnopqrstuvwxyz1234567890"));
    assert!(text.contains("[redacted-secret]"));
    assert!(text.contains("[redacted-local-path]"));
}

#[test]
fn collector_rejects_unknown_schema_without_merging_reports() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [{
            "body": "```json\n{\"schemaVersion\":2,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue #136 linux\",\"commandSummary\":\"passed\"}]}\n```"
        }]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(report.extracted_blocks.len(), 1);
    assert_eq!(
        report.extracted_blocks[0].status,
        IssueTerminalEvidenceBlockStatus::Rejected
    );
    assert_eq!(report.extracted_blocks[0].report_count, 0);
    assert!(report.terminal_evidence.reports.is_empty());
    assert!(report.merged_evidence.is_none());
    assert!(
        report.extracted_blocks[0]
            .issues
            .iter()
            .any(|issue| issue.contains("schemaVersion must be 1"))
    );
}

#[test]
fn collector_labels_body_only_terminal_evidence_source() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue body linux\",\"commandSummary\":\"passed\"},{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"issue body windows\",\"commandSummary\":\"passed\"}]}",
        "comments": []
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(
        report.terminal_evidence.source_path.as_deref(),
        Some("issue #136 body")
    );
}

#[test]
fn collector_labels_mixed_body_and_comment_terminal_evidence_source() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\",\"status\":\"passed\",\"environment\":\"local_terminal\",\"source\":\"issue body linux\",\"commandSummary\":\"passed\"}]}",
        "comments": [{
            "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"windows\",\"status\":\"passed\",\"environment\":\"vm_or_cloud_terminal\",\"source\":\"comment windows\",\"commandSummary\":\"passed\"}]}\n```"
        }]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(
        report.terminal_evidence.source_path.as_deref(),
        Some("issue #136 body + comments")
    );
}

#[test]
fn collector_keeps_block_parse_errors_out_of_top_level_findings() {
    let value = json!({
        "number": 136,
        "title": "Run post-release TUI terminal QA on Windows and Linux",
        "url": "https://github.com/example/repo/issues/136",
        "body": "",
        "comments": [{
            "body": "```json\n{\"schemaVersion\":1,\"reports\":[{\"platform\":\"linux\"\n```"
        }]
    });

    let report = build_issue_terminal_evidence_report_from_value(None, 136, &value);

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert!(
        report.extracted_blocks[0]
            .issues
            .iter()
            .any(|issue| issue.contains("JSON could not be parsed"))
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| !finding.contains("JSON could not be parsed"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("terminal evidence block rejected"))
    );
}
