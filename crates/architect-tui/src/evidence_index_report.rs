use serde::Serialize;

use crate::governance_audit_public_summary::GovernanceAuditPublicSummary;
use crate::governance_audit_report::GovernanceAuditStatus;
use crate::launch_judge_report::LaunchJudgeResult;
use crate::launch_readiness_public_summary::LaunchReadinessPublicSummary;
use crate::launch_stack_github::public_text;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceIndexReport {
    pub schema_version: u8,
    pub generated_at_unix_seconds: u64,
    pub result: LaunchJudgeResult,
    pub repository: Option<String>,
    pub read_only: bool,
    pub sections: Vec<EvidenceIndexSection>,
    pub launch_readiness: LaunchReadinessPublicSummary,
    pub governance_audit: GovernanceAuditPublicSummary,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceIndexSection {
    pub name: String,
    pub result: LaunchJudgeResult,
    pub status: String,
    pub summary: String,
}

pub(crate) fn build_evidence_index_report_from_public_summaries_at(
    launch_readiness: LaunchReadinessPublicSummary,
    governance_audit: GovernanceAuditPublicSummary,
    generated_at_unix_seconds: u64,
) -> EvidenceIndexReport {
    let launch_readiness = sanitize_launch_readiness_summary(launch_readiness);
    let governance_audit = sanitize_governance_summary(governance_audit);
    let launch_result = launch_readiness.result.clone();
    let governance_result = governance_result(&governance_audit.status);
    let result = combine_results(&[launch_result.clone(), governance_result.clone()]);
    let repository = launch_readiness
        .repository
        .as_deref()
        .map(|repository| public_text(repository, 160));
    let read_only = launch_readiness.read_only && governance_audit.read_only;
    let sections = vec![
        EvidenceIndexSection {
            name: "launch readiness".to_string(),
            result: launch_result,
            status: launch_result_status(&launch_readiness.result).to_string(),
            summary: launch_summary(&launch_readiness),
        },
        EvidenceIndexSection {
            name: "governance audit".to_string(),
            result: governance_result,
            status: governance_status(&governance_audit.status).to_string(),
            summary: governance_summary(&governance_audit),
        },
    ];
    let findings = public_strings(
        &launch_readiness
            .findings
            .iter()
            .cloned()
            .chain(governance_audit.findings.iter().map(|finding| {
                format!(
                    "{} {}: {}",
                    format!("{:?}", finding.severity).to_ascii_lowercase(),
                    finding.code,
                    finding.message
                )
            }))
            .collect::<Vec<_>>(),
        320,
    );
    let mut next_actions = public_strings(
        &launch_readiness
            .next_actions
            .iter()
            .cloned()
            .chain(governance_audit.next_actions.iter().cloned())
            .collect::<Vec<_>>(),
        320,
    );
    dedupe(&mut next_actions);

    EvidenceIndexReport {
        schema_version: 1,
        generated_at_unix_seconds,
        result,
        repository,
        read_only,
        sections,
        launch_readiness,
        governance_audit,
        findings,
        next_actions,
    }
}

fn sanitize_launch_readiness_summary(
    mut summary: LaunchReadinessPublicSummary,
) -> LaunchReadinessPublicSummary {
    summary.repository = summary
        .repository
        .as_deref()
        .map(|repository| public_text(repository, 160));
    summary.launch_stack.stopped_at_base = summary
        .launch_stack
        .stopped_at_base
        .as_deref()
        .map(|base| public_text(base, 120));
    for blocker in &mut summary.launch_stack.blocker_issues {
        blocker.state = public_text(&blocker.state, 80);
    }
    summary.terminal_evidence.platforms = public_strings(&summary.terminal_evidence.platforms, 80);
    summary.terminal_evidence.issues = public_strings(&summary.terminal_evidence.issues, 320);
    if let Some(waiver) = &mut summary.terminal_evidence_waiver {
        waiver.reason = public_text(&waiver.reason, 240);
    }
    summary.findings = public_strings(&summary.findings, 320);
    summary.next_actions = public_strings(&summary.next_actions, 320);
    summary
}

fn sanitize_governance_summary(
    mut summary: GovernanceAuditPublicSummary,
) -> GovernanceAuditPublicSummary {
    for category in &mut summary.categories {
        category.name = public_text(&category.name, 120);
        category.summary = public_text(&category.summary, 240);
    }
    summary.deterministic_gates.names = public_strings(&summary.deterministic_gates.names, 120);
    summary.smoke_evidence.names = public_strings(&summary.smoke_evidence.names, 120);
    if let Some(review) = &mut summary.mcp_review {
        review.status = public_text(&review.status, 80);
        review.gate_status = review
            .gate_status
            .as_deref()
            .map(|status| public_text(status, 80));
    }
    for finding in &mut summary.findings {
        finding.category = public_text(&finding.category, 120);
        finding.code = public_text(&finding.code, 120);
        finding.message = public_text(&finding.message, 240);
        finding.next_action = public_text(&finding.next_action, 240);
    }
    summary.next_actions = public_strings(&summary.next_actions, 240);
    summary
}

fn launch_result_status(result: &LaunchJudgeResult) -> &'static str {
    match result {
        LaunchJudgeResult::Go => "go",
        LaunchJudgeResult::ConditionalGo => "conditional_go",
        LaunchJudgeResult::NoGo => "no_go",
    }
}

fn governance_status(status: &GovernanceAuditStatus) -> &'static str {
    match status {
        GovernanceAuditStatus::Passed => "passed",
        GovernanceAuditStatus::PassedWithWarnings => "passed_with_warnings",
        GovernanceAuditStatus::Failed => "failed",
    }
}

fn governance_result(status: &GovernanceAuditStatus) -> LaunchJudgeResult {
    match status {
        GovernanceAuditStatus::Passed => LaunchJudgeResult::Go,
        GovernanceAuditStatus::PassedWithWarnings => LaunchJudgeResult::ConditionalGo,
        GovernanceAuditStatus::Failed => LaunchJudgeResult::NoGo,
    }
}

fn combine_results(results: &[LaunchJudgeResult]) -> LaunchJudgeResult {
    if results
        .iter()
        .any(|result| result == &LaunchJudgeResult::NoGo)
    {
        LaunchJudgeResult::NoGo
    } else if results
        .iter()
        .any(|result| result == &LaunchJudgeResult::ConditionalGo)
    {
        LaunchJudgeResult::ConditionalGo
    } else {
        LaunchJudgeResult::Go
    }
}

fn launch_summary(summary: &LaunchReadinessPublicSummary) -> String {
    public_text(
        &format!(
            "{} PRs, {} blockers, {} terminal reports",
            summary.launch_stack.pull_request_count,
            summary.launch_stack.blocker_issues.len(),
            summary.terminal_evidence.report_count
        ),
        240,
    )
}

fn governance_summary(summary: &GovernanceAuditPublicSummary) -> String {
    public_text(
        &format!(
            "{} categories, {} deterministic gates, {} smoke evidence entries, {} findings",
            summary.categories.len(),
            summary.deterministic_gates.count,
            summary.smoke_evidence.count,
            summary.findings.len()
        ),
        240,
    )
}

fn public_strings(values: &[String], max_len: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| public_text(value, max_len))
        .collect()
}

fn dedupe(values: &mut Vec<String>) {
    let mut seen = Vec::new();
    values.retain(|value| {
        if seen.contains(value) {
            false
        } else {
            seen.push(value.clone());
            true
        }
    });
}
