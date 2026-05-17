use crate::governance_audit_report::{GovernanceAuditReport, GovernanceAuditStatus};
use crate::launch_judge_public_summary::build_public_summary_at;
use crate::launch_judge_report::{
    LaunchJudgeCheckStatus, LaunchJudgeCommandEvidence, LaunchJudgeTerminalEvidenceReport,
    LaunchJudgeTerminalEvidenceStatus, LaunchJudgeTerminalEvidenceSummary, build_report, check,
};

#[test]
fn public_summary_omits_workspace_and_raw_command_tails() {
    let report = build_report(
        std::path::Path::new("/Users/example/private/architect-mcp"),
        governance_report("/Users/example/private/architect-mcp"),
        None,
        LaunchJudgeCommandEvidence {
            command: "npm run release:check".to_string(),
            attempted: true,
            ok: false,
            exit_code: Some(1),
            stdout_tail: vec!["built from /Users/example/private/architect-mcp".to_string()],
            stderr_tail: vec!["npm_SECRET=do-not-print".to_string()],
            error: Some("failed at /home/example/private".to_string()),
        },
        check(
            "git worktree",
            LaunchJudgeCheckStatus::Warning,
            "dirty workspace at /Users/example/private/architect-mcp with npm_SECRET token",
            Some("inspect /home/example/private before release"),
        ),
        LaunchJudgeTerminalEvidenceSummary {
            supplied: true,
            source_path: Some("/Users/example/private/linux-evidence.json".to_string()),
            reports: vec![LaunchJudgeTerminalEvidenceReport {
                platform: "linux".to_string(),
                status: LaunchJudgeTerminalEvidenceStatus::Passed,
                source: "issue #136 /Users/example/private".to_string(),
                command_summary: "architect-mcp-tui passed from /home/example/private".to_string(),
                collected_at: Some("2026-05-17".to_string()),
                notes: Some("token npm_SECRET and C:/Users/example/cache omitted".to_string()),
            }],
            issues: vec!["manual Windows evidence missing at /home/example".to_string()],
        },
        check(
            "external terminal evidence",
            LaunchJudgeCheckStatus::Warning,
            "terminal evidence incomplete at /Users/example/private",
            Some("collect Windows evidence from C:/Users/example"),
        ),
    );

    let summary = build_public_summary_at(&report, 1_779_000_000);
    let text = serde_json::to_string_pretty(&summary).expect("serialize public summary");

    assert!(text.contains("\"schemaVersion\": 1"));
    assert!(text.contains("\"generatedAtUnixSeconds\": 1779000000"));
    assert!(text.contains("\"releaseCheck\""));
    assert!(text.contains("\"attempted\": true"));
    assert!(text.contains("\"exitCode\": 1"));
    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("\"workspace\""));
    assert!(!text.contains("stdoutTail"));
    assert!(!text.contains("stderrTail"));
    assert!(!text.contains("commandSummary"));
    assert!(!text.contains("\"source\""));
    assert!(!text.contains("\"notes\""));
    assert!(!text.contains("issue #136"));
    assert!(!text.contains("npm_SECRET"));
    assert!(!text.contains("/Users/"));
    assert!(!text.contains("/home/"));
    assert!(!text.contains("C:/Users/"));
}

#[test]
fn public_summary_keeps_terminal_platform_statuses() {
    let report = build_report(
        std::path::Path::new("/tmp/workspace"),
        governance_report("/tmp/workspace"),
        None,
        LaunchJudgeCommandEvidence {
            command: "npm run release:check".to_string(),
            attempted: false,
            ok: false,
            exit_code: None,
            stdout_tail: Vec::new(),
            stderr_tail: Vec::new(),
            error: None,
        },
        check(
            "git worktree",
            LaunchJudgeCheckStatus::Passed,
            "git worktree is clean",
            None,
        ),
        LaunchJudgeTerminalEvidenceSummary {
            supplied: true,
            source_path: Some("linux-evidence.json, windows-evidence.json".to_string()),
            reports: vec![
                LaunchJudgeTerminalEvidenceReport {
                    platform: "linux".to_string(),
                    status: LaunchJudgeTerminalEvidenceStatus::Passed,
                    source: "issue #136 linux report".to_string(),
                    command_summary: "terminal evidence passed on linux".to_string(),
                    collected_at: None,
                    notes: None,
                },
                LaunchJudgeTerminalEvidenceReport {
                    platform: "windows".to_string(),
                    status: LaunchJudgeTerminalEvidenceStatus::Passed,
                    source: "issue #136 windows report".to_string(),
                    command_summary: "terminal evidence passed on windows".to_string(),
                    collected_at: None,
                    notes: None,
                },
            ],
            issues: Vec::new(),
        },
        check(
            "external terminal evidence",
            LaunchJudgeCheckStatus::Passed,
            "public-safe Linux and Windows terminal evidence was supplied",
            None,
        ),
    );

    let summary = build_public_summary_at(&report, 1);

    assert!(summary.terminal_evidence.supplied);
    assert_eq!(
        summary.terminal_evidence.source_path.as_deref(),
        Some("linux-evidence.json, windows-evidence.json")
    );
    assert_eq!(summary.terminal_evidence.reports.len(), 2);
    assert_eq!(summary.terminal_evidence.reports[0].platform, "linux");
    assert_eq!(summary.terminal_evidence.reports[1].platform, "windows");
}

fn governance_report(workspace: &str) -> GovernanceAuditReport {
    GovernanceAuditReport {
        schema_version: 1,
        status: GovernanceAuditStatus::Passed,
        workspace: workspace.to_string(),
        read_only: true,
        categories: Vec::new(),
        deterministic_gates: Vec::new(),
        smoke_evidence: Vec::new(),
        memory_proposals: Vec::new(),
        mcp_review: None,
        findings: Vec::new(),
    }
}
