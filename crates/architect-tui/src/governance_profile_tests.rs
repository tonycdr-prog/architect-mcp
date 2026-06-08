use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::config::TuiConfig;
use crate::governance_audit::{GovernanceAuditOptions, build_governance_audit_report};
use crate::governance_audit_report::GovernanceAuditStatus;
use crate::governance_profile::GovernanceRepoProfile;

#[tokio::test]
async fn governance_audit_does_not_require_rust_tui_or_publish_artifacts_for_static_apps() {
    let temp = tempdir().expect("temp");
    let root = temp.path();
    write_static_web_app_fixture(root);

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

    assert_eq!(report.status, GovernanceAuditStatus::Passed);
    assert_eq!(
        report.repo_profile.name,
        GovernanceRepoProfile::StaticWebApp
    );
    assert!(
        report
            .deterministic_gates
            .iter()
            .all(|gate| gate.command != "npm run rust:check")
    );
    for forbidden in [
        "Cargo.lock",
        "rust:check",
        "tui:live-qa",
        ".github/workflows/npm-publish.yml",
        "check:v10",
    ] {
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.message.contains(forbidden)
                    && !finding.evidence.contains(forbidden)),
            "unexpected static-app governance finding for {forbidden}: {:?}",
            report.findings
        );
    }
}

fn write_static_web_app_fixture(root: &Path) {
    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::create_dir_all(root.join(".github/workflows")).expect("workflows");
    fs::write(
        root.join("AGENTS.md"),
        "Use memory for durable context. Do not store secrets, raw conversation logs, or transient task details.",
    )
    .expect("agents");
    fs::write(root.join("docs/architecture-contract.md"), "contract").expect("contract");
    fs::write(root.join("docs/build-plan.md"), "plan").expect("plan");
    fs::write(root.join("README.md"), "readme").expect("readme");
    fs::write(root.join("llms.txt"), "llms").expect("llms");
    fs::write(root.join(".env.example"), "VITE_PUBLIC_BASE_URL=").expect("env");
    fs::write(root.join("package-lock.json"), "{}").expect("package lock");
    fs::write(root.join(".github/dependabot.yml"), "version: 2").expect("dependabot");
    fs::write(
        root.join(".github/workflows/ci.yml"),
        "run: npm run typecheck\nrun: npm test\nrun: npm run build\nrun: npm run docs:build\nrun: npm run lint\nrun: npm run release:check\n",
    )
    .expect("ci");
    fs::write(
        root.join("package.json"),
        r#"{
          "private": true,
          "type": "module",
          "scripts": {
            "release:check": "npm run typecheck && npm test && npm run docs:build && npm run build && npm run lint",
            "typecheck": "tsc --noEmit",
            "test": "vitest run",
            "build": "vite build",
            "docs:build": "vitepress build docs",
            "lint": "eslint ."
          },
          "dependencies": {
            "@vitejs/plugin-react": "latest",
            "vite": "latest",
            "react": "latest",
            "react-dom": "latest"
          },
          "devDependencies": {
            "typescript": "latest",
            "vitest": "latest",
            "eslint": "latest"
          }
        }"#,
    )
    .expect("package");
}
