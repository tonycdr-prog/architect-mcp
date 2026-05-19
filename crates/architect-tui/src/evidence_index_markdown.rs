use crate::evidence_index_report::EvidenceIndexReport;
use crate::governance_audit_report::GovernanceAuditStatus;
use crate::launch_judge_report::{
    LaunchJudgeResult, LaunchJudgeTerminalEvidenceEnvironment, LaunchJudgeTerminalEvidenceStatus,
};
use crate::launch_stack::LaunchStackItemStatus;

pub(crate) fn render_markdown(report: &EvidenceIndexReport) -> String {
    let mut out = String::new();
    out.push_str("# Release Evidence Index\n\n");
    out.push_str(&format!(
        "- Result: {}\n",
        inline_code(result_label(&report.result))
    ));
    out.push_str(&format!("- Read-only: {}\n", inline_code(report.read_only)));
    if let Some(repository) = &report.repository {
        out.push_str(&format!("- Repository: {}\n", inline_code(repository)));
    }
    out.push_str("\n## Sections\n\n");
    out.push_str("| Section | Result | Summary |\n");
    out.push_str("| --- | --- | --- |\n");
    for section in &report.sections {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            table_text(&section.name),
            inline_code(result_label(&section.result)),
            table_text(&section.summary)
        ));
    }

    let launch_stack = &report.launch_readiness.launch_stack;
    let terminal_evidence = &report.launch_readiness.terminal_evidence;
    let governance_audit = &report.governance_audit;

    out.push_str("\n## Launch Readiness\n\n");
    out.push_str(&format!(
        "- Pull requests: {}\n",
        inline_code(launch_stack.pull_request_count)
    ));
    let pr_status = &launch_stack.pull_request_status;
    out.push_str(&format!(
        "- PR status: {} passed, {} waived, {} warnings, {} failed\n",
        inline_code(pr_status.passed),
        inline_code(pr_status.waived),
        inline_code(pr_status.warning),
        inline_code(pr_status.failed)
    ));
    if launch_stack.unresolved_review_thread_count > 0 {
        out.push_str(&format!(
            "- Unresolved review threads: {}\n",
            inline_code(launch_stack.unresolved_review_thread_count)
        ));
        for entry in &launch_stack.unresolved_review_threads {
            out.push_str(&format!(
                "  - PR #{}: {}\n",
                entry.pull_request,
                inline_code(entry.count)
            ));
        }
    }
    if launch_stack.missing_required_check_count > 0 {
        out.push_str(&format!(
            "- Missing required checks: {}\n",
            inline_code(launch_stack.missing_required_check_count)
        ));
        for missing in &launch_stack.missing_required_checks {
            out.push_str(&format!(
                "  - PR #{}: {}\n",
                missing.pull_request,
                inline_list(&missing.names)
            ));
        }
    }
    out.push_str(&format!(
        "- Blockers: {}\n",
        inline_code(launch_stack.blocker_issues.len())
    ));
    for blocker in &launch_stack.blocker_issues {
        out.push_str(&format!(
            "  - #{}: {} state {} waived {}\n",
            blocker.number,
            inline_code(launch_stack_status(&blocker.status)),
            inline_code(&blocker.state),
            inline_code(blocker.waived)
        ));
    }
    out.push_str(&format!(
        "- Terminal evidence reports: {}\n",
        inline_code(terminal_evidence.report_count)
    ));
    if let Some(issue) = &terminal_evidence.issue {
        out.push_str(&format!(
            "- Terminal evidence issue: #{} {} with {} extracted blocks\n",
            issue.number,
            inline_code(result_label(&issue.result)),
            inline_code(issue.extracted_block_count)
        ));
    }
    if !terminal_evidence.platforms.is_empty() {
        out.push_str(&format!(
            "- Platforms: {}\n",
            inline_list(&terminal_evidence.platforms)
        ));
    }
    if !terminal_evidence.reports.is_empty() {
        out.push_str("- Terminal evidence provenance:\n");
        for evidence in &terminal_evidence.reports {
            let collected_at = evidence.collected_at.as_deref().unwrap_or("missing");
            out.push_str(&format!(
                "  - {}: status {}, environment {}, collectedAt {}\n",
                inline_code(&evidence.platform),
                inline_code(terminal_status(&evidence.status)),
                inline_code(terminal_environment(evidence.environment.as_ref())),
                inline_code(collected_at)
            ));
        }
    }
    if let Some(waiver) = &report.launch_readiness.terminal_evidence_waiver {
        out.push_str(&format!(
            "- Terminal evidence waiver: issue #{} applied {} reason: {}\n",
            waiver.issue,
            inline_code(waiver.applied),
            markdown_text(&waiver.reason)
        ));
    }

    out.push_str("\n## Governance Audit\n\n");
    out.push_str(&format!(
        "- Status: {}\n",
        inline_code(governance_status(&governance_audit.status))
    ));
    out.push_str(&format!(
        "- Categories: {}\n",
        inline_code(governance_audit.categories.len())
    ));
    out.push_str(&format!(
        "- Deterministic gates: {} total, {} release-required\n",
        inline_code(governance_audit.deterministic_gates.count),
        inline_code(
            governance_audit
                .deterministic_gates
                .required_for_release_count
        )
    ));
    out.push_str(&format!(
        "- Smoke evidence: {} entries\n",
        inline_code(governance_audit.smoke_evidence.count)
    ));
    out.push_str(&format!(
        "- Memory proposals: {} safe, {} unsafe\n",
        inline_code(governance_audit.memory.safe_to_store_count),
        inline_code(governance_audit.memory.unsafe_proposal_count)
    ));
    if let Some(review) = &governance_audit.mcp_review {
        out.push_str(&format!("- MCP review: {}", inline_code(&review.status)));
        if let Some(gate_status) = &review.gate_status {
            out.push_str(&format!(" gate {}", inline_code(gate_status)));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "- Findings: {} errors, {} warnings, {} info\n",
        inline_code(governance_audit.finding_counts.errors),
        inline_code(governance_audit.finding_counts.warnings),
        inline_code(governance_audit.finding_counts.info)
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
        out.push_str(&format!("- {}\n", markdown_text(value)));
    }
}

fn inline_list(values: &[String]) -> String {
    values
        .iter()
        .map(inline_code)
        .collect::<Vec<_>>()
        .join(", ")
}

fn inline_code(value: impl ToString) -> String {
    let value = value.to_string().replace('\n', " ");
    let max_backtick_run = max_consecutive_backticks(&value);
    let fence = "`".repeat(max_backtick_run + 1);
    if max_backtick_run > 0 {
        format!("{fence} {value} {fence}")
    } else {
        format!("{fence}{value}{fence}")
    }
}

fn max_consecutive_backticks(value: &str) -> usize {
    let mut max_run = 0;
    let mut current_run = 0;
    for character in value.chars() {
        if character == '`' {
            current_run += 1;
            max_run = max_run.max(current_run);
        } else {
            current_run = 0;
        }
    }
    max_run
}

fn markdown_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('\n', " ")
}

fn table_text(value: &str) -> String {
    markdown_text(value).replace('|', "\\|")
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

fn launch_stack_status(status: &LaunchStackItemStatus) -> &'static str {
    match status {
        LaunchStackItemStatus::Passed => "passed",
        LaunchStackItemStatus::Waived => "waived",
        LaunchStackItemStatus::Warning => "warning",
        LaunchStackItemStatus::Failed => "failed",
    }
}

fn terminal_status(status: &LaunchJudgeTerminalEvidenceStatus) -> &'static str {
    match status {
        LaunchJudgeTerminalEvidenceStatus::Passed => "passed",
        LaunchJudgeTerminalEvidenceStatus::PassedWithWarnings => "passed_with_warnings",
        LaunchJudgeTerminalEvidenceStatus::Failed => "failed",
    }
}

fn terminal_environment(
    environment: Option<&LaunchJudgeTerminalEvidenceEnvironment>,
) -> &'static str {
    match environment {
        Some(LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal) => "local_terminal",
        Some(LaunchJudgeTerminalEvidenceEnvironment::VmOrCloudTerminal) => "vm_or_cloud_terminal",
        Some(LaunchJudgeTerminalEvidenceEnvironment::Container) => "container",
        Some(LaunchJudgeTerminalEvidenceEnvironment::HostedCi) => "hosted_ci",
        Some(LaunchJudgeTerminalEvidenceEnvironment::Unknown) => "unknown",
        None => "missing",
    }
}
