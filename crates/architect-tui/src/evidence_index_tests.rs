use std::path::Path;

use crate::evidence_index::{
    enforce_evidence_index_result, validate_output_mode, write_markdown_output,
};
use crate::evidence_index_markdown::render_markdown;
use crate::evidence_index_report::build_evidence_index_report_from_public_summaries_at;
use crate::governance_audit_public_summary::{
    GovernanceAuditPublicCategory, GovernanceAuditPublicFinding,
    GovernanceAuditPublicFindingCounts, GovernanceAuditPublicGateSummary,
    GovernanceAuditPublicMcpReview, GovernanceAuditPublicMemorySummary,
    GovernanceAuditPublicSummary,
};
use crate::governance_audit_report::{GovernanceAuditStatus, GovernanceFindingSeverity};
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness_public_summary::{
    LaunchReadinessPublicBlockerIssue, LaunchReadinessPublicStack,
    LaunchReadinessPublicStatusCounts, LaunchReadinessPublicSummary,
    LaunchReadinessPublicTerminalEvidence, LaunchReadinessPublicTerminalEvidenceIssue,
    LaunchReadinessPublicTerminalEvidenceWaiver,
};
use crate::launch_stack::LaunchStackItemStatus;

#[test]
fn evidence_index_combines_launch_and_governance_public_summaries() {
    let report = build_evidence_index_report_from_public_summaries_at(
        launch_summary(LaunchJudgeResult::Go),
        governance_summary(GovernanceAuditStatus::PassedWithWarnings),
        1_779_000_000,
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.generated_at_unix_seconds, 1_779_000_000);
    assert_eq!(report.result, LaunchJudgeResult::ConditionalGo);
    assert!(report.read_only);
    assert_eq!(report.repository, Some("example/repo".to_string()));
    assert_eq!(report.sections.len(), 2);
    assert_eq!(report.sections[0].name, "launch readiness");
    assert_eq!(report.sections[0].status, "go");
    assert_eq!(report.sections[1].name, "governance audit");
    assert_eq!(report.sections[1].status, "passed_with_warnings");
    assert!(report.sections[0].summary.contains("2 PRs"));
    assert!(report.sections[1].summary.contains("1 categories"));
}

#[test]
fn evidence_index_fails_when_any_section_is_no_go() {
    let report = build_evidence_index_report_from_public_summaries_at(
        launch_summary(LaunchJudgeResult::Go),
        governance_summary(GovernanceAuditStatus::Failed),
        1,
    );

    assert_eq!(report.result, LaunchJudgeResult::NoGo);
    assert_eq!(report.sections[1].result, LaunchJudgeResult::NoGo);
}

#[test]
fn evidence_index_redacts_public_text_and_omits_raw_payloads() {
    let mut launch = launch_summary(LaunchJudgeResult::ConditionalGo);
    launch.repository = Some("/Users/example/private/repo npm_SECRET".to_string());
    launch
        .findings
        .push("manual note referenced /Users/example/private and ghp_example".to_string());
    launch
        .next_actions
        .push("inspect /Users/example/private before publishing npm_SECRET".to_string());

    let report = build_evidence_index_report_from_public_summaries_at(
        launch,
        governance_summary(GovernanceAuditStatus::Passed),
        1,
    );
    let text = serde_json::to_string_pretty(&report).expect("serialize evidence index");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("/Users/example"));
    assert!(!text.contains("npm_SECRET"));
    assert!(!text.contains("ghp_example"));
    assert!(!text.contains("workspace"));
    assert!(!text.contains("commandSummary"));
    assert!(!text.contains("stdout"));
    assert!(!text.contains("stderr"));
    assert!(!text.contains("memory proposal text"));
}

#[test]
fn evidence_index_markdown_renders_public_release_handoff() {
    let mut launch = launch_summary(LaunchJudgeResult::ConditionalGo);
    launch.repository = Some("/Users/example/private/repo npm_SECRET `quoted`".to_string());
    launch.launch_stack.blocker_issues = vec![LaunchReadinessPublicBlockerIssue {
        number: 136,
        state: "OPEN`state".to_string(),
        status: LaunchStackItemStatus::Warning,
        waived: false,
    }];
    launch.terminal_evidence.issue = Some(LaunchReadinessPublicTerminalEvidenceIssue {
        number: 136,
        result: LaunchJudgeResult::ConditionalGo,
        extracted_block_count: 0,
    });
    launch.terminal_evidence.platforms = vec!["linux`runner".to_string()];
    launch.terminal_evidence.issues =
        vec!["terminal evidence reports must include linux and windows".to_string()];
    launch.terminal_evidence_waiver = Some(LaunchReadinessPublicTerminalEvidenceWaiver {
        issue: 136,
        applied: false,
        reason: "maintainer has not accepted a waiver from /Users/example/private with npm_SECRET"
            .to_string(),
    });
    let mut governance = governance_summary(GovernanceAuditStatus::Passed);
    if let Some(review) = &mut governance.mcp_review {
        review.status = "passed`review".to_string();
        review.gate_status = Some("pass`gate".to_string());
    }

    let report = build_evidence_index_report_from_public_summaries_at(launch, governance, 1);
    let markdown = render_markdown(&report);

    assert!(markdown.contains("# Release Evidence Index"));
    assert!(markdown.contains("- Result: `conditional_go`"));
    assert!(
        markdown.contains("- Repository: `` [redacted-local-path] [redacted-secret] `quoted` ``")
    );
    assert!(markdown.contains("| launch readiness | `conditional_go` |"));
    assert!(markdown.contains("- Platforms: `` linux`runner ``"));
    assert!(markdown.contains("- MCP review: `` passed`review `` gate `` pass`gate ``"));
    assert!(markdown.contains("Terminal evidence issue: #136"));
    assert!(markdown.contains("Terminal evidence waiver: issue #136"));
    assert!(markdown.contains("[redacted-local-path]"));
    assert!(markdown.contains("[redacted-secret]"));
    assert!(markdown.contains("## Next Actions"));
    assert!(!markdown.contains("commandSummary"));
    assert!(!markdown.contains("stdout"));
    assert!(!markdown.contains("stderr"));
    assert!(!markdown.contains("/Users/example"));
    assert!(!markdown.contains("npm_SECRET"));
}

#[test]
fn evidence_index_writes_markdown_handoff_inside_workspace() {
    let temp = tempfile::tempdir().expect("tempdir");
    let report = build_evidence_index_report_from_public_summaries_at(
        launch_summary(LaunchJudgeResult::ConditionalGo),
        governance_summary(GovernanceAuditStatus::Passed),
        1,
    );
    let markdown = render_markdown(&report);

    let output = write_markdown_output(
        temp.path(),
        Path::new(".architect-mcp/release/evidence-index.md"),
        &markdown,
    )
    .expect("write markdown handoff");
    let written = std::fs::read_to_string(&output).expect("read markdown handoff");

    assert!(output.starts_with(temp.path()));
    assert_eq!(written, markdown);
    assert!(written.contains("# Release Evidence Index"));
    assert!(!written.contains("stdout"));
    assert!(!written.contains("stderr"));
}

#[test]
fn evidence_index_rejects_markdown_output_outside_workspace() {
    let temp = tempfile::tempdir().expect("tempdir");

    assert!(write_markdown_output(temp.path(), Path::new("../evidence.md"), "safe").is_err());
    assert!(write_markdown_output(temp.path(), Path::new("/tmp/evidence.md"), "safe").is_err());
}

#[test]
fn evidence_index_rejects_ambiguous_output_modes() {
    let error = validate_output_mode(true, true).expect_err("ambiguous mode should fail");

    assert!(
        error
            .to_string()
            .contains("choose only one evidence-index output mode")
    );
}

#[test]
fn evidence_index_require_go_fails_conditional_results_only_when_requested() {
    assert!(enforce_evidence_index_result(&LaunchJudgeResult::ConditionalGo, false).is_ok());

    let error = enforce_evidence_index_result(&LaunchJudgeResult::ConditionalGo, true)
        .expect_err("strict release gate should reject conditional go");
    assert!(error.to_string().contains("--require-go requires go"));
}

#[test]
fn evidence_index_no_go_always_fails() {
    assert!(enforce_evidence_index_result(&LaunchJudgeResult::NoGo, false).is_err());
    assert!(enforce_evidence_index_result(&LaunchJudgeResult::NoGo, true).is_err());
}

fn launch_summary(result: LaunchJudgeResult) -> LaunchReadinessPublicSummary {
    LaunchReadinessPublicSummary {
        schema_version: 1,
        generated_at_unix_seconds: 1,
        result: result.clone(),
        repository: Some("example/repo".to_string()),
        read_only: true,
        launch_stack: LaunchReadinessPublicStack {
            result,
            from_pr: Some(198),
            stopped_at_base: Some("main".to_string()),
            pull_request_count: 2,
            pull_request_order: vec![150, 198],
            pull_request_status: LaunchReadinessPublicStatusCounts {
                passed: 2,
                waived: 0,
                warning: 0,
                failed: 0,
            },
            blocker_issues: Vec::new(),
        },
        terminal_evidence: LaunchReadinessPublicTerminalEvidence {
            issue: None,
            report_count: 0,
            platforms: Vec::new(),
            issues: Vec::new(),
        },
        terminal_evidence_waiver: None,
        findings: Vec::new(),
        next_actions: vec!["collect final release evidence".to_string()],
    }
}

fn governance_summary(status: GovernanceAuditStatus) -> GovernanceAuditPublicSummary {
    let finding_severity = if status == GovernanceAuditStatus::Failed {
        GovernanceFindingSeverity::Error
    } else {
        GovernanceFindingSeverity::Warning
    };

    GovernanceAuditPublicSummary {
        schema_version: 1,
        status,
        read_only: true,
        categories: vec![GovernanceAuditPublicCategory {
            name: "release".to_string(),
            status: GovernanceAuditStatus::PassedWithWarnings,
            summary: "release evidence still needs maintainer review".to_string(),
        }],
        deterministic_gates: GovernanceAuditPublicGateSummary {
            count: 2,
            required_for_release_count: 1,
            names: vec!["release gate".to_string(), "typecheck".to_string()],
        },
        smoke_evidence: GovernanceAuditPublicGateSummary {
            count: 1,
            required_for_release_count: 0,
            names: vec!["tui live qa".to_string()],
        },
        memory: GovernanceAuditPublicMemorySummary {
            proposal_count: 0,
            safe_to_store_count: 0,
            unsafe_proposal_count: 0,
        },
        mcp_review: Some(GovernanceAuditPublicMcpReview {
            status: "passed".to_string(),
            gate_status: Some("pass".to_string()),
            files_reviewed: Some(42),
            errors: Some(0),
            warnings: Some(1),
            violation_count: Some(0),
        }),
        finding_counts: GovernanceAuditPublicFindingCounts {
            info: 0,
            warnings: usize::from(finding_severity == GovernanceFindingSeverity::Warning),
            errors: usize::from(finding_severity == GovernanceFindingSeverity::Error),
        },
        findings: vec![GovernanceAuditPublicFinding {
            severity: finding_severity,
            category: "release".to_string(),
            code: "GOV_RELEASE_EVIDENCE".to_string(),
            message: "release evidence needs maintainer review".to_string(),
            next_action: "collect final release evidence".to_string(),
        }],
        next_actions: vec!["collect final release evidence".to_string()],
    }
}
