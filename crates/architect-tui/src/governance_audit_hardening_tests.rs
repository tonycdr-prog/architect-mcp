use std::fs;

use tempfile::tempdir;

use crate::config::TuiConfig;
use crate::governance_audit::{
    GovernanceAuditOptions, build_governance_audit_report, push_mcp_review_findings,
};
use crate::governance_audit_report::{
    GovernanceAuditStatus, GovernanceFinding, GovernanceMcpReview,
};
use crate::governance_audit_tests::write_fixture_repo;

#[tokio::test]
async fn governance_audit_detects_common_secret_shaped_local_config() {
    let temp = tempdir().expect("temp");
    let root = temp.path();
    write_fixture_repo(root);
    fs::write(
        root.join(".env.local"),
        "SESSION_SECRET=super-secret-value\nDATABASE_URL=postgres://user:pass@localhost/app\n",
    )
    .expect("env");
    fs::write(
        root.join(".mcp.json"),
        r#"{"servers":{"custom":{"env":{"API_KEY":"abcdefghijklmnopqrstuvwxyz123456"}}}}"#,
    )
    .expect("mcp config");

    let report = build_governance_audit_report(
        root.to_path_buf(),
        TuiConfig::default(),
        &GovernanceAuditOptions {
            json: true,
            public_summary: false,
            skip_mcp: true,
            max_files: 100,
            profile: None,
        },
    )
    .await;

    assert_eq!(report.status, GovernanceAuditStatus::Failed);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "GOV_SECRET_SHAPED_CONFIG")
    );
}

#[tokio::test]
async fn governance_audit_detects_secret_shaped_mcp_config_without_env_file() {
    let temp = tempdir().expect("temp");
    let root = temp.path();
    write_fixture_repo(root);
    fs::write(
        root.join(".mcp.json"),
        r#"{"servers":{"custom":{"env":{"API_KEY":"abcdefghijklmnopqrstuvwxyz123456"}}}}"#,
    )
    .expect("mcp config");

    let report = build_governance_audit_report(
        root.to_path_buf(),
        TuiConfig::default(),
        &GovernanceAuditOptions {
            json: true,
            public_summary: false,
            skip_mcp: true,
            max_files: 100,
            profile: None,
        },
    )
    .await;

    assert_eq!(report.status, GovernanceAuditStatus::Failed);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "GOV_SECRET_SHAPED_CONFIG")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn governance_audit_skips_symlinked_config_paths() {
    let temp = tempdir().expect("temp");
    let root = temp.path().join("workspace");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&root).expect("workspace");
    fs::create_dir_all(&outside).expect("outside");
    write_fixture_repo(&root);
    fs::write(
        outside.join(".env.local"),
        "SESSION_SECRET=super-secret-value\n",
    )
    .expect("outside env");
    std::os::unix::fs::symlink(outside.join(".env.local"), root.join(".env.local"))
        .expect("config symlink");
    fs::create_dir_all(outside.join("configs")).expect("outside configs");
    fs::write(
        outside.join("configs").join(".mcp.json"),
        r#"{"env":{"API_KEY":"abcdefghijklmnopqrstuvwxyz123456"}}"#,
    )
    .expect("outside mcp");
    std::os::unix::fs::symlink(outside.join("configs"), root.join("configs"))
        .expect("directory symlink");

    let report = build_governance_audit_report(
        root,
        TuiConfig::default(),
        &GovernanceAuditOptions {
            json: true,
            public_summary: false,
            skip_mcp: true,
            max_files: 100,
            profile: None,
        },
    )
    .await;

    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.code != "GOV_SECRET_SHAPED_CONFIG")
    );
}

#[test]
fn governance_audit_reports_mcp_warn_and_truncation_findings() {
    let mut findings: Vec<GovernanceFinding> = Vec::new();
    push_mcp_review_findings(
        &mut findings,
        &GovernanceMcpReview {
            status: "warn".to_string(),
            gate_status: Some("warn".to_string()),
            files_reviewed: Some(5000),
            scan_truncated: Some(true),
            errors: Some(0),
            warnings: Some(8),
            violation_count: Some(0),
            detail: "review_local_workspace mode=audit completed without writing files".to_string(),
        },
    );

    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "GOV_MCP_REVIEW_NOT_CLEAN")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "GOV_MCP_REVIEW_TRUNCATED")
    );
}
