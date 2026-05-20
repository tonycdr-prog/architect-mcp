use crate::foundry_audit_public_summary::build_public_summary;
use crate::foundry_audit_report::{
    FoundryAuditActionabilitySummary, FoundryAuditConstitutionSummary, FoundryAuditDecision,
    FoundryAuditEvidenceSummary, FoundryAuditForgeSummary, FoundryAuditLedgerSummary,
    FoundryAuditMcpSummary, FoundryAuditPublicSafety, FoundryAuditReport, FoundryAuditStatus,
};

#[test]
fn public_summary_omits_paths_tokens_and_raw_details_from_decisions() {
    let summary = build_public_summary(&FoundryAuditReport {
        schema_version: 1,
        status: FoundryAuditStatus::PassedWithWarnings,
        workspace: "/Users/alice/private/repo".to_string(),
        read_only: true,
        mutation_boundary: "preview_only_no_repository_mutation".to_string(),
        approval_required_before_mutation: true,
        server_writes_performed: 0,
        mcp: FoundryAuditMcpSummary {
            tools_called: vec!["review_local_workspace".to_string()],
            files_reviewed: Some(7),
            scan_truncated: false,
            review_gate_status: Some("warn".to_string()),
        },
        constitution: FoundryAuditConstitutionSummary {
            repo_shape: Some("rust_cli".to_string()),
            ..FoundryAuditConstitutionSummary::default()
        },
        evidence: FoundryAuditEvidenceSummary {
            total_evidence: 1,
            redacted: 1,
            omitted_raw_payloads: 1,
            suppression_candidates: 0,
            coverage_caveats: 0,
        },
        actionability: FoundryAuditActionabilitySummary::default(),
        ledger: FoundryAuditLedgerSummary::default(),
        forge: FoundryAuditForgeSummary::default(),
        decisions: vec![FoundryAuditDecision {
            id: "fdl-001-pr_preview".to_string(),
            route: "pr_preview".to_string(),
            score: 76,
            evidence_count: 2,
            risk: "redacted".to_string(),
            approval_state: "approval_required".to_string(),
            next_action:
                "Do not paste stdout: secret lines from /Users/alice/private/repo with ghp_123456"
                    .to_string(),
            preview_kind: Some("pull_request".to_string()),
            preview_title: Some("[foundry-preview] /Users/alice/private/repo".to_string()),
            warning_count: 1,
        }],
        public_safety: FoundryAuditPublicSafety::default(),
        error: None,
    });

    let json = serde_json::to_string(&summary).expect("summary json");
    assert!(!json.contains("/Users/alice"));
    assert!(!json.contains("ghp_123456"));
    assert!(!json.contains("stdout:"));
    assert!(json.contains("[redacted-local-path]"));
    assert!(json.contains("[redacted-secret]"));
    assert!(summary.read_only);
    assert_eq!(summary.server_writes_performed, 0);
}
