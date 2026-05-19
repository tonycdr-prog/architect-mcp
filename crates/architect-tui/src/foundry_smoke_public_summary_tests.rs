use std::path::PathBuf;

use crate::foundry_smoke::{FoundryGithubVerification, FoundrySmokeCommandReport};
use crate::foundry_smoke_public_summary::build_foundry_smoke_public_summary;
use crate::foundry_smoke_report::{FoundrySmokeReport, FoundrySmokeStatus};
use crate::foundry_smoke_retention::FoundrySmokeRetentionDecisionRecord;

#[test]
fn public_summary_reports_live_private_repo_evidence_without_private_names() {
    let report = sample_report(FoundrySmokeStatus::Passed, "live");

    let summary = build_foundry_smoke_public_summary(&report);
    let json = serde_json::to_string(&summary).expect("serialize summary");

    assert_eq!(summary.status, FoundrySmokeStatus::Passed);
    assert!(summary.mutation.github_mutation);
    assert!(summary.mutation.requires_execute);
    assert!(summary.mutation.requires_confirm_private_repo_mutation);
    assert!(summary.github.as_ref().unwrap().private_repo_verified);
    assert!(summary.github.as_ref().unwrap().draft_pr_verified);
    assert_eq!(summary.commands.total, 2);
    assert_eq!(summary.commands.passed, 2);
    assert!(summary.target_redacted);
    assert!(!json.contains("owner/private-proof-repo"));
    assert!(!json.contains("https://github.com/owner/private-proof-repo"));
    assert!(!json.contains("/Users/"));
    assert!(!json.contains("gh repo create"));
}

#[test]
fn public_summary_keeps_dry_run_non_mutating() {
    let mut report = sample_report(FoundrySmokeStatus::Passed, "dry_run");
    report.github = None;

    let summary = build_foundry_smoke_public_summary(&report);

    assert!(!summary.mutation.github_mutation);
    assert!(!summary.mutation.requires_execute);
    assert!(!summary.mutation.requires_confirm_private_repo_mutation);
    assert!(summary.github.is_none());
    assert!(
        summary
            .next_actions
            .iter()
            .any(|action| action.contains("--execute --confirm-private-repo-mutation"))
    );
}

#[test]
fn public_summary_redacts_errors_and_counts_failed_commands() {
    let mut report = sample_report(FoundrySmokeStatus::Failed, "live");
    report.error = Some(
        "failed at /Users/example/project with npm_secret, ghp_secret, owner/private-proof-repo, private-proof-repo, and https://github.com/owner/private-proof-repo/pull/1".to_string(),
    );
    report.commands.push(FoundrySmokeCommandReport {
        command: "gh repo create owner/private-proof-repo".to_string(),
        ok: false,
        transcript: Vec::new(),
        error: Some("boom".to_string()),
    });

    let summary = build_foundry_smoke_public_summary(&report);
    let json = serde_json::to_string(&summary).expect("serialize summary");

    assert_eq!(summary.commands.failed, 1);
    assert_eq!(summary.status, FoundrySmokeStatus::Failed);
    assert!(json.contains("[redacted-local-path]"));
    assert!(json.contains("[redacted-secret]"));
    assert!(!json.contains("/Users/"));
    assert!(!json.contains("npm_secret"));
    assert!(!json.contains("ghp_secret"));
    assert!(!json.contains("owner/private-proof-repo"));
    assert!(!json.contains("private-proof-repo"));
    assert!(!json.contains("https://github.com/owner/private-proof-repo/pull/1"));
}

#[test]
fn public_summary_omits_raw_command_output_and_mcp_payload_details() {
    let mut report = sample_report(FoundrySmokeStatus::Failed, "live");
    report.error = Some(
        "gh repo create owner/private-proof-repo failed: stderr tail: {\"jsonrpc\":\"2.0\",\"method\":\"tools/call\"} transcript: stdout from /Users/example/workspace/staged".to_string(),
    );

    let summary = build_foundry_smoke_public_summary(&report);
    let json = serde_json::to_string(&summary).expect("serialize summary");

    assert!(
        summary
            .findings
            .iter()
            .any(|finding| finding.contains("raw command output omitted"))
    );
    assert!(
        summary
            .findings
            .iter()
            .all(|finding| !finding.to_ascii_lowercase().contains("stdout"))
    );
    assert!(
        summary
            .findings
            .iter()
            .all(|finding| !finding.to_ascii_lowercase().contains("stderr"))
    );
    assert!(!json.contains("gh repo create"));
    assert!(!json.contains("tools/call"));
    assert!(!json.contains("\"jsonrpc\""));
    assert!(!json.contains("/Users/example/workspace/staged"));
}

#[test]
fn public_summary_draft_pr_verification_requires_url() {
    let mut report = sample_report(FoundrySmokeStatus::Passed, "live");
    report.github.as_mut().unwrap().draft_pr_url = None;

    let summary = build_foundry_smoke_public_summary(&report);

    assert!(!summary.github.as_ref().unwrap().draft_pr_verified);
}

#[test]
fn public_summary_reports_retention_decision_without_deletion_claims() {
    let mut report = sample_report(FoundrySmokeStatus::Passed, "live");
    report.retention =
        "delete-later requested after review: remove after maintainer approval".to_string();
    report.retention_decision = Some(FoundrySmokeRetentionDecisionRecord {
        decision: "delete_later_requested".to_string(),
        reason: "remove after maintainer approval".to_string(),
        deletion_performed: false,
    });

    let summary = build_foundry_smoke_public_summary(&report);

    let decision = summary
        .retention_decision
        .as_ref()
        .expect("retention decision");
    assert_eq!(decision.decision, "delete_later_requested");
    assert_eq!(decision.reason, "remove after maintainer approval");
    assert!(!decision.deletion_performed);
    assert!(
        summary
            .next_actions
            .iter()
            .any(|action| action.contains("foundry-smoke did not delete any repository"))
    );
}

#[test]
fn public_summary_redacts_retention_reason_private_details() {
    let mut report = sample_report(FoundrySmokeStatus::Passed, "live");
    report.retention = "retained for maintainer evidence: keep owner/private-proof-repo because /Users/example/workspace has npm_secret".to_string();
    report.retention_decision = Some(FoundrySmokeRetentionDecisionRecord {
        decision: "retained_for_evidence".to_string(),
        reason: "keep owner/private-proof-repo at https://github.com/owner/private-proof-repo because /Users/example/workspace has npm_secret".to_string(),
        deletion_performed: false,
    });

    let summary = build_foundry_smoke_public_summary(&report);
    let json = serde_json::to_string(&summary).expect("serialize summary");

    assert!(json.contains("[redacted-target]"));
    assert!(json.contains("[redacted-local-path]"));
    assert!(json.contains("[redacted-secret]"));
    assert!(!json.contains("owner/private-proof-repo"));
    assert!(!json.contains("private-proof-repo"));
    assert!(!json.contains("https://github.com/owner/private-proof-repo"));
    assert!(!json.contains("/Users/"));
    assert!(!json.contains("npm_secret"));
}

#[test]
fn public_summary_redacts_short_repo_names_and_private_key_markers() {
    let mut report = sample_report(FoundrySmokeStatus::Failed, "live");
    report.target = "owner/xy".to_string();
    report.github.as_mut().unwrap().repo_url = "https://github.com/owner/xy".to_string();
    report.github.as_mut().unwrap().draft_pr_url =
        Some("https://github.com/owner/xy/pull/1".to_string());
    report.error = Some(
        "repo xy failed with -----BEGIN PRIVATE KEY----- abc123 -----END PRIVATE KEY-----"
            .to_string(),
    );

    let summary = build_foundry_smoke_public_summary(&report);
    let json = serde_json::to_string(&summary).expect("serialize summary");

    assert!(json.contains("[redacted-secret]"));
    assert!(!json.contains("owner/xy"));
    assert!(!json.contains("https://github.com/owner/xy"));
    assert!(!json.contains("xy failed"));
    assert!(!json.contains("BEGIN PRIVATE KEY"));
    assert!(!json.contains("abc123"));
}

fn sample_report(status: FoundrySmokeStatus, mode: &str) -> FoundrySmokeReport {
    FoundrySmokeReport {
        schema_version: 1,
        status,
        mode: mode.to_string(),
        target: "owner/private-proof-repo".to_string(),
        workspace: PathBuf::from("/Users/example/workspace"),
        staged_repo: Some(PathBuf::from("/Users/example/workspace/staged")),
        staged_artifacts: 9,
        execution_status: Some("passed".to_string()),
        github: Some(FoundryGithubVerification {
            repo_url: "https://github.com/owner/private-proof-repo".to_string(),
            repo_visibility: "PRIVATE".to_string(),
            is_private: true,
            draft_pr_url: Some("https://github.com/owner/private-proof-repo/pull/1".to_string()),
            draft_pr_number: Some(1),
            draft_pr_is_draft: Some(true),
            draft_pr_state: Some("OPEN".to_string()),
        }),
        commands: vec![
            FoundrySmokeCommandReport {
                command: "foundry plan private-proof-repo".to_string(),
                ok: true,
                transcript: vec!["planned".to_string()],
                error: None,
            },
            FoundrySmokeCommandReport {
                command: "gh repo create owner/private-proof-repo".to_string(),
                ok: true,
                transcript: vec!["created".to_string()],
                error: None,
            },
        ],
        error: None,
        retention: "private GitHub repo retained for maintainer evidence".to_string(),
        retention_decision: None,
    }
}
