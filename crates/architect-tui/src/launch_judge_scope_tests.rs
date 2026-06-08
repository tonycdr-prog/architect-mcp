use std::fs;

use crate::governance_audit_report::GovernanceAuditStatus;
use crate::governance_profile::{
    GovernanceProfileConfidence, GovernanceRepoProfile, GovernanceRepoProfileSummary,
};
use crate::launch_judge_evidence::{
    build_terminal_evidence_summary_for_scope, read_terminal_evidence_for_scope,
};
use crate::launch_judge_report::{
    LaunchJudgeCheckStatus, LaunchJudgeCommandEvidence, LaunchJudgeResult,
    LaunchJudgeTerminalEvidenceReport, LaunchJudgeTerminalEvidenceStatus, build_report, check,
};
use crate::launch_scope::{
    LaunchScope, LaunchScopeSource, LaunchScopeSummary, resolve_launch_scope,
};
use crate::smoke_types::{
    AdapterSummary, BinarySummary, CommandCheck, EnvironmentSummary, GateSmoke, SmokeReport,
    SmokeStatus,
};

#[test]
fn missing_terminal_evidence_is_future_evidence_for_local_demo_scope() {
    let (summary, check) = read_terminal_evidence_for_scope(&[], LaunchScope::LocalDemo);

    assert!(!summary.supplied);
    assert_eq!(check.status, LaunchJudgeCheckStatus::Info);
    assert!(check.detail.contains("local-demo"));
}

#[test]
fn incomplete_terminal_evidence_is_future_evidence_for_internal_v0_scope() {
    let reports = vec![LaunchJudgeTerminalEvidenceReport {
        platform: "linux".to_string(),
        status: LaunchJudgeTerminalEvidenceStatus::Passed,
        environment: None,
        source: "issue #136 linux public-safe summary".to_string(),
        command_summary: "terminal evidence passed on linux".to_string(),
        collected_at: None,
        notes: None,
    }];

    let (summary, check) = build_terminal_evidence_summary_for_scope(
        Some("issue #136".to_string()),
        reports,
        Vec::new(),
        false,
        LaunchScope::InternalV0,
    );

    assert_eq!(check.status, LaunchJudgeCheckStatus::Info);
    assert!(summary.issues.iter().any(|issue| issue.contains("windows")));
}

#[test]
fn local_demo_scope_can_go_with_future_terminal_evidence_gap() {
    let (terminal_evidence, terminal_evidence_check) =
        read_terminal_evidence_for_scope(&[], LaunchScope::LocalDemo);

    let report = build_report(
        std::path::Path::new("/tmp/workspace"),
        scope_summary(LaunchScope::LocalDemo),
        governance_report(),
        Some(passed_smoke_report()),
        LaunchJudgeCommandEvidence {
            command: "npm run release:check".to_string(),
            attempted: true,
            ok: Some(true),
            exit_code: Some(0),
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
        (terminal_evidence, terminal_evidence_check),
    );

    assert_eq!(report.result, LaunchJudgeResult::Go);
    assert_eq!(
        report.future_launch_readiness.public_cli,
        LaunchJudgeResult::ConditionalGo
    );
    assert!(
        report
            .future_launch_readiness
            .missing_evidence
            .iter()
            .any(|issue| issue.contains("Windows and Linux"))
    );
}

#[test]
fn launch_scope_infers_local_demo_from_explicit_narrow_readiness_claims() {
    let temp = tempfile::tempdir().expect("temp");
    let docs = temp.path().join("docs");
    fs::create_dir_all(&docs).expect("docs");
    fs::write(
        docs.join("readiness.md"),
        "Status: local synthetic v0 demo ready. This is not production-proven, not customer-validated, and not public launch-ready.",
    )
    .expect("readiness");

    let scope = resolve_launch_scope(temp.path(), None);

    assert_eq!(scope.name, LaunchScope::LocalDemo);
    assert_eq!(scope.resolution, LaunchScopeSource::Inferred);
}

fn governance_report() -> crate::governance_audit_report::GovernanceAuditReport {
    crate::governance_audit_report::GovernanceAuditReport {
        schema_version: 1,
        status: GovernanceAuditStatus::Passed,
        workspace: "/tmp/workspace".to_string(),
        read_only: true,
        repo_profile: GovernanceRepoProfileSummary {
            name: GovernanceRepoProfile::StaticWebApp,
            confidence: GovernanceProfileConfidence::High,
            signals: vec!["test fixture".to_string()],
        },
        categories: Vec::new(),
        deterministic_gates: Vec::new(),
        smoke_evidence: Vec::new(),
        memory_proposals: Vec::new(),
        mcp_review: None,
        findings: Vec::new(),
    }
}

fn scope_summary(name: LaunchScope) -> LaunchScopeSummary {
    LaunchScopeSummary {
        name,
        resolution: LaunchScopeSource::Supplied,
        signals: vec!["test fixture".to_string()],
    }
}

fn passed_smoke_report() -> SmokeReport {
    SmokeReport {
        schema_version: 1,
        status: SmokeStatus::Passed,
        workspace: "/tmp/workspace".to_string(),
        environment: EnvironmentSummary {
            os: "linux".to_string(),
            arch: "x64".to_string(),
            family: "unix".to_string(),
            current_dir: "/tmp/workspace".to_string(),
            terminal: Default::default(),
            tools: Default::default(),
        },
        binary: BinarySummary {
            path: None,
            source: "test".to_string(),
            sha256: None,
            npm_cache_root: None,
            npm_cache_env_override: None,
        },
        architect_mcp: crate::mcp::McpProcessSpec {
            command: "node".to_string(),
            args: Vec::new(),
            tool_surface: "advanced".to_string(),
        },
        help: command_check("architect-mcp-tui --help"),
        adapters: AdapterSummary {
            total: 0,
            ready: 0,
            health: Vec::new(),
        },
        gate_only: GateSmoke {
            attempted: true,
            command: "architect-mcp-tui run --jsonl".to_string(),
            ok: true,
            status: Some("approval_required".to_string()),
            event_types: Vec::new(),
            jsonl_line_count: 0,
            jsonl_tail: Vec::new(),
            error: None,
        },
    }
}

fn command_check(command: &str) -> CommandCheck {
    CommandCheck {
        command: command.to_string(),
        ok: true,
        exit_code: Some(0),
        timed_out: false,
        first_line: None,
    }
}
