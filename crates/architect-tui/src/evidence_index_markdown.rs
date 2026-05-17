use crate::evidence_index_report::EvidenceIndexReport;
use crate::governance_audit_report::GovernanceAuditStatus;
use crate::launch_judge_report::LaunchJudgeResult;

pub(crate) fn render_markdown(report: &EvidenceIndexReport) -> String {
    let mut out = String::new();
    out.push_str("# Release Evidence Index\n\n");
    out.push_str(&format!("- Result: `{}`\n", result_label(&report.result)));
    out.push_str(&format!("- Read-only: `{}`\n", report.read_only));
    if let Some(repository) = &report.repository {
        out.push_str(&format!("- Repository: `{repository}`\n"));
    }
    out.push_str("\n## Sections\n\n");
    out.push_str("| Section | Result | Summary |\n");
    out.push_str("| --- | --- | --- |\n");
    for section in &report.sections {
        out.push_str(&format!(
            "| {} | `{}` | {} |\n",
            table_text(&section.name),
            result_label(&section.result),
            table_text(&section.summary)
        ));
    }

    out.push_str("\n## Launch Readiness\n\n");
    out.push_str(&format!(
        "- Pull requests: `{}`\n",
        report.launch_readiness.launch_stack.pull_request_count
    ));
    out.push_str(&format!(
        "- PR status: `{}` passed, `{}` waived, `{}` warnings, `{}` failed\n",
        report
            .launch_readiness
            .launch_stack
            .pull_request_status
            .passed,
        report
            .launch_readiness
            .launch_stack
            .pull_request_status
            .waived,
        report
            .launch_readiness
            .launch_stack
            .pull_request_status
            .warning,
        report
            .launch_readiness
            .launch_stack
            .pull_request_status
            .failed
    ));
    out.push_str(&format!(
        "- Blockers: `{}`\n",
        report.launch_readiness.launch_stack.blocker_issues.len()
    ));
    for blocker in &report.launch_readiness.launch_stack.blocker_issues {
        out.push_str(&format!(
            "  - #{}: `{}` state `{}` waived `{}`\n",
            blocker.number,
            format!("{:?}", blocker.status).to_ascii_lowercase(),
            blocker.state,
            blocker.waived
        ));
    }
    out.push_str(&format!(
        "- Terminal evidence reports: `{}`\n",
        report.launch_readiness.terminal_evidence.report_count
    ));
    if let Some(issue) = &report.launch_readiness.terminal_evidence.issue {
        out.push_str(&format!(
            "- Terminal evidence issue: #{} `{}` with `{}` extracted blocks\n",
            issue.number,
            result_label(&issue.result),
            issue.extracted_block_count
        ));
    }
    if !report
        .launch_readiness
        .terminal_evidence
        .platforms
        .is_empty()
    {
        out.push_str(&format!(
            "- Platforms: {}\n",
            inline_list(&report.launch_readiness.terminal_evidence.platforms)
        ));
    }
    if let Some(waiver) = &report.launch_readiness.terminal_evidence_waiver {
        out.push_str(&format!(
            "- Terminal evidence waiver: issue #{} applied `{}` reason: {}\n",
            waiver.issue, waiver.applied, waiver.reason
        ));
    }

    out.push_str("\n## Governance Audit\n\n");
    out.push_str(&format!(
        "- Status: `{}`\n",
        governance_status(&report.governance_audit.status)
    ));
    out.push_str(&format!(
        "- Categories: `{}`\n",
        report.governance_audit.categories.len()
    ));
    out.push_str(&format!(
        "- Deterministic gates: `{}` total, `{}` release-required\n",
        report.governance_audit.deterministic_gates.count,
        report
            .governance_audit
            .deterministic_gates
            .required_for_release_count
    ));
    out.push_str(&format!(
        "- Smoke evidence: `{}` entries\n",
        report.governance_audit.smoke_evidence.count
    ));
    out.push_str(&format!(
        "- Memory proposals: `{}` safe, `{}` unsafe\n",
        report.governance_audit.memory.safe_to_store_count,
        report.governance_audit.memory.unsafe_proposal_count
    ));
    if let Some(review) = &report.governance_audit.mcp_review {
        out.push_str(&format!("- MCP review: `{}`", review.status));
        if let Some(gate_status) = &review.gate_status {
            out.push_str(&format!(" gate `{gate_status}`"));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "- Findings: `{}` errors, `{}` warnings, `{}` info\n",
        report.governance_audit.finding_counts.errors,
        report.governance_audit.finding_counts.warnings,
        report.governance_audit.finding_counts.info
    ));

    append_list(&mut out, "Findings", &report.findings);
    append_list(&mut out, "Next Actions", &report.next_actions);
    out
}

fn append_list(out: &mut String, title: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    out.push_str(&format!("\n## {title}\n\n"));
    for value in values {
        out.push_str(&format!("- {value}\n"));
    }
}

fn inline_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn table_text(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn result_label(result: &LaunchJudgeResult) -> &'static str {
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
