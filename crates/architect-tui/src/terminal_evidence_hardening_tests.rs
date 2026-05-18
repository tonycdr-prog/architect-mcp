use crate::issue_terminal_evidence_source::extract_json_blocks;
use crate::launch_judge_report::{
    LaunchJudgeTerminalEvidenceEnvironment, LaunchJudgeTerminalEvidenceReport,
    LaunchJudgeTerminalEvidenceStatus,
};
use crate::terminal_evidence::{TerminalEvidenceFile, render_markdown, resolve_platform};
use crate::terminal_evidence_issue_url::validate_issue_url;

#[test]
fn terminal_evidence_platform_override_must_match_current_os() {
    let error = resolve_platform("linux", Some("windows"))
        .expect_err("platform override must match actual smoke OS");

    assert!(
        error
            .to_string()
            .contains("--platform must match the current terminal OS")
    );
}

#[test]
fn terminal_evidence_platform_override_cannot_relabel_unsupported_os() {
    let error = resolve_platform("macos", Some("linux"))
        .expect_err("unsupported OS must not be relabelled");

    assert!(error.to_string().contains("linux or windows"));
}

#[test]
fn terminal_evidence_markdown_can_name_target_issue_url() {
    let evidence = TerminalEvidenceFile {
        schema_version: 1,
        reports: vec![LaunchJudgeTerminalEvidenceReport {
            platform: "linux".to_string(),
            status: LaunchJudgeTerminalEvidenceStatus::Passed,
            environment: Some(LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal),
            source: "issue #136 Linux terminal report".to_string(),
            command_summary: "architect-mcp-tui terminal-evidence passed on linux".to_string(),
            collected_at: Some("2026-05-17".to_string()),
            notes: Some("summary only, no raw logs".to_string()),
        }],
    };

    let markdown = render_markdown(
        &evidence,
        Some("https://github.com/tonycdr-prog/architect-mcp/issues/136"),
    )
    .expect("markdown");

    assert!(markdown.contains(
        "paste this comment manually into https://github.com/tonycdr-prog/architect-mcp/issues/136"
    ));
    assert!(markdown.contains("This command does not create, edit, or close GitHub issues"));
    assert_eq!(extract_json_blocks(&markdown).len(), 1);
}

#[test]
fn terminal_evidence_issue_url_rejects_non_github_issue_urls_without_echoing_input() {
    let error = validate_issue_url("https://example.com/Users/private/issues/136?token=npm_secret")
        .expect_err("non-GitHub issue URL should fail");
    let message = error.to_string();

    assert!(message.contains("--issue-url must be a plain GitHub issue URL"));
    assert!(!message.contains("example.com"));
    assert!(!message.contains("/Users/"));
    assert!(!message.contains("npm_"));
}
