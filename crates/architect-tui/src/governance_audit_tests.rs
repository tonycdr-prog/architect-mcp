use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::config::TuiConfig;
use crate::governance_audit::{GovernanceAuditOptions, build_governance_audit_report};
use crate::governance_audit_report::{GovernanceAuditStatus, GovernanceMemoryProposal};
use crate::governance_memory::{filter_memory_proposals, memory_proposals};

#[tokio::test]
async fn governance_audit_reports_read_only_release_and_memory_evidence() {
    let temp = tempdir().expect("temp");
    let root = temp.path();
    write_fixture_repo(root);

    let report = build_governance_audit_report(
        root.to_path_buf(),
        TuiConfig::default(),
        &GovernanceAuditOptions {
            json: true,
            public_summary: false,
            skip_mcp: true,
            max_files: 100,
        },
    )
    .await;

    assert!(report.read_only);
    assert_ne!(report.status, GovernanceAuditStatus::Failed);
    assert!(
        report
            .deterministic_gates
            .iter()
            .any(|gate| gate.command == "npm run release:check" && gate.required_for_release)
    );
    assert!(
        report
            .smoke_evidence
            .iter()
            .any(|gate| gate.command == "npm run tui:live-qa" && !gate.required_for_release)
    );
    assert!(
        report
            .memory_proposals
            .iter()
            .all(|proposal| proposal.safe_to_store)
    );
    assert!(report.findings.iter().all(|finding| {
        finding.code != "GOV_REQUIRED_FILE_MISSING" && finding.code != "GOV_SECRET_SHAPED_CONFIG"
    }));
}

#[test]
fn memory_proposals_reject_secrets_raw_chat_and_transient_details() {
    let proposals = filter_memory_proposals(vec![
        proposal("release gate: npm run release:check"),
        proposal("raw chat: user said temporary detail"),
        proposal("token npm_secretValueShouldNeverPersist"),
        proposal("transient task detail from this one run"),
    ]);

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].text, "release gate: npm run release:check");
}

#[test]
fn memory_proposals_reject_all_agents_md_prohibited_categories() {
    let keep = "architect-mcp release gate: npm run release:check (clean checkout)";
    let proposals = filter_memory_proposals(vec![
        proposal(keep),
        proposal("full chat transcript from today's session"),
        proposal("chat transcript: agent said hello"),
        proposal("customer data: user email and plan tier"),
        proposal("private customer record with billing details"),
        proposal("speculative guess: maybe the db is postgres"),
        proposal("speculative detail about future architecture"),
        proposal("sensitive security finding: auth bypass in login"),
        proposal("exploit detail: SSRF via redirect"),
        proposal("vulnerability detail from pentest report"),
    ]);

    assert_eq!(proposals.len(), 1, "only the safe proposal should survive");
    assert_eq!(proposals[0].text, keep);
}

#[test]
fn memory_proposals_are_scoped_to_audited_workspace_artifacts() {
    let temp = tempdir().expect("temp");
    let root = temp.path();
    fs::create_dir_all(root.join(".git")).expect("git");
    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::write(
        root.join(".git/config"),
        "[remote \"origin\"]\n\turl = https://github.com/example/sample-app.git\n",
    )
    .expect("git config");
    fs::write(
        root.join("package.json"),
        r#"{"scripts":{"release:check":"npm test"}}"#,
    )
    .expect("package");
    fs::write(
        root.join("AGENTS.md"),
        "Use memory only for durable project context.",
    )
    .expect("agents");
    fs::write(root.join("docs/architecture-contract.md"), "contract").expect("contract");

    let proposals = memory_proposals(root);

    assert_eq!(proposals.len(), 3);
    assert!(
        proposals
            .iter()
            .all(|proposal| proposal.scope == "example/sample-app")
    );
    assert!(
        proposals
            .iter()
            .all(|proposal| !proposal.text.contains("architect-mcp goal"))
    );
    assert!(
        proposals
            .iter()
            .any(|proposal| proposal.source == "package.json")
    );
    assert!(
        proposals
            .iter()
            .any(|proposal| proposal.source == "AGENTS.md")
    );
    assert!(
        proposals
            .iter()
            .any(|proposal| proposal.source == "docs/architecture-contract.md")
    );
}

#[test]
fn memory_proposals_do_not_invent_architect_mcp_context_for_unadopted_repos() {
    let temp = tempdir().expect("temp");
    let root = temp.path();
    fs::write(root.join("README.md"), "external repo").expect("readme");

    let proposals = memory_proposals(root);

    assert!(proposals.is_empty());
}

fn proposal(text: &str) -> GovernanceMemoryProposal {
    GovernanceMemoryProposal {
        scope: "architect-mcp".to_string(),
        source: "test".to_string(),
        text: text.to_string(),
        safe_to_store: true,
        review_note: "safe durable project context; no secrets or raw chat".to_string(),
    }
}

fn write_fixture_repo(root: &Path) {
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
    fs::write(root.join(".env.example"), "TOKEN=replace_me").expect("env");
    fs::write(root.join("package-lock.json"), "{}").expect("package lock");
    fs::write(root.join("Cargo.lock"), "# lock").expect("cargo lock");
    fs::write(root.join(".github/dependabot.yml"), "version: 2").expect("dependabot");
    fs::write(
        root.join(".github/workflows/ci.yml"),
        "run: npm run rust:check\nrun: npm run typecheck\nrun: npm test\nrun: npm run build\n",
    )
    .expect("ci");
    fs::write(
        root.join(".github/workflows/npm-publish.yml"),
        "run: npm run release:check\n",
    )
    .expect("publish");
    fs::write(
        root.join(".github/workflows/tui-live-qa.yml"),
        "run: npm run tui:live-qa\n",
    )
    .expect("qa");
    fs::write(
        root.join("package.json"),
        r#"{
          "scripts": {
            "release:check": "npm run rust:check && npm run check:v10",
            "rust:check": "cargo test --workspace",
            "typecheck": "tsc --noEmit",
            "test": "node --test",
            "build": "tsc",
            "docs:build": "vitepress build docs",
            "tui:live-qa": "architect-mcp-tui smoke --json"
          }
        }"#,
    )
    .expect("package");
}
