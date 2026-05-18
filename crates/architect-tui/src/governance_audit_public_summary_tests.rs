use crate::governance_audit_public_summary::build_public_summary;
use crate::governance_audit_report::{
    GovernanceAuditCategory, GovernanceAuditReport, GovernanceAuditStatus, GovernanceFinding,
    GovernanceFindingSeverity, GovernanceGateEvidence, GovernanceMcpReview,
};

#[test]
fn governance_public_summary_preserves_counts_without_raw_payloads() {
    let report = report_with_sensitive_details();

    let summary = build_public_summary(&report);

    assert_eq!(summary.schema_version, 1);
    assert_eq!(summary.status, GovernanceAuditStatus::Failed);
    assert!(summary.read_only);
    assert_eq!(summary.categories.len(), 1);
    assert_eq!(summary.deterministic_gates.count, 2);
    assert_eq!(summary.deterministic_gates.required_for_release_count, 1);
    assert_eq!(summary.smoke_evidence.count, 1);
    assert_eq!(summary.memory.proposal_count, 2);
    assert_eq!(summary.memory.safe_to_store_count, 1);
    assert_eq!(summary.memory.unsafe_proposal_count, 1);
    assert_eq!(summary.finding_counts.errors, 1);
    assert_eq!(summary.finding_counts.warnings, 1);
    assert_eq!(summary.next_actions.len(), 2);
}

#[test]
fn governance_public_summary_redacts_and_omits_sensitive_fields() {
    let report = report_with_sensitive_details();

    let text =
        serde_json::to_string_pretty(&build_public_summary(&report)).expect("serialize summary");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("\"workspace\""));
    assert!(!text.contains("\"command\""));
    assert!(!text.contains("\"source\""));
    assert!(!text.contains("\"evidence\""));
    assert!(!text.contains("\"detail\""));
    assert!(!text.contains("\"text\""));
    assert!(!text.contains("raw chat transcript"));
    assert!(!text.contains("npm_SECRET"));
    assert!(!text.contains("/Users/example/private"));
    assert!(!text.contains("ghp_exampleSecret"));
}

fn report_with_sensitive_details() -> GovernanceAuditReport {
    GovernanceAuditReport {
        schema_version: 1,
        status: GovernanceAuditStatus::Failed,
        workspace: "/Users/example/private/repo".to_string(),
        read_only: true,
        categories: vec![GovernanceAuditCategory {
            name: "security".to_string(),
            status: GovernanceAuditStatus::Failed,
            summary: "Secret-like config found near /Users/example/private".to_string(),
        }],
        deterministic_gates: vec![
            gate(
                "release gate",
                "npm run release:check --token npm_SECRET",
                true,
            ),
            gate("typecheck", "npm run typecheck", false),
        ],
        smoke_evidence: vec![gate("tui live qa", "npm run tui:live-qa", false)],
        memory_proposals: vec![
            crate::governance_audit_report::GovernanceMemoryProposal {
                scope: "example/repo".to_string(),
                source: "AGENTS.md".to_string(),
                text: "raw chat transcript with ghp_exampleSecret".to_string(),
                safe_to_store: false,
                review_note: "unsafe".to_string(),
            },
            crate::governance_audit_report::GovernanceMemoryProposal {
                scope: "example/repo".to_string(),
                source: "package.json".to_string(),
                text: "release gate: npm run release:check".to_string(),
                safe_to_store: true,
                review_note: "safe".to_string(),
            },
        ],
        mcp_review: Some(GovernanceMcpReview {
            status: "passed".to_string(),
            gate_status: Some("pass".to_string()),
            files_reviewed: Some(42),
            scan_truncated: Some(true),
            errors: Some(0),
            warnings: Some(1),
            violation_count: Some(1),
            detail: "reviewed /Users/example/private/repo with npm_SECRET".to_string(),
        }),
        findings: vec![
            GovernanceFinding {
                severity: GovernanceFindingSeverity::Error,
                category: "security".to_string(),
                code: "GOV_SECRET_SHAPED_CONFIG".to_string(),
                message: "Found npm_SECRET in /Users/example/private/repo/.env".to_string(),
                evidence: "/Users/example/private/repo/.env".to_string(),
                next_action: "Remove npm_SECRET from /Users/example/private/repo/.env".to_string(),
            },
            GovernanceFinding {
                severity: GovernanceFindingSeverity::Warning,
                category: "mcp-review".to_string(),
                code: "GOV_MCP_REVIEW_FAILED".to_string(),
                message: "MCP repo review reported warnings.".to_string(),
                evidence: "full MCP detail should stay local".to_string(),
                next_action: "Inspect review locally before release claims.".to_string(),
            },
        ],
    }
}

fn gate(name: &str, command: &str, required_for_release: bool) -> GovernanceGateEvidence {
    GovernanceGateEvidence {
        name: name.to_string(),
        kind: "deterministic".to_string(),
        command: command.to_string(),
        required_for_release,
        source: "package.json".to_string(),
    }
}
