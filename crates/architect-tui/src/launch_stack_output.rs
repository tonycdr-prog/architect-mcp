use crate::launch_stack::LaunchStackReport;

pub(crate) fn print_launch_stack_text_report(report: &LaunchStackReport) {
    println!("architect-mcp-tui launch stack: {:?}", report.result);
    if let Some(discovery) = &report.stack_discovery {
        let prs = discovery
            .pull_requests
            .iter()
            .map(|number| format!("#{number}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "- discovered from PR #{}: {} (stopped at {})",
            discovery.from_pr, prs, discovery.stopped_at_base
        );
    }
    for pr in &report.pull_requests {
        println!(
            "- PR #{}: {:?} draft={} review={} merge={} mergeable={} checks passed={} pending={} failed={} missing_required={}",
            pr.number,
            pr.status,
            pr.is_draft,
            pr.review_decision.as_deref().unwrap_or("none"),
            pr.merge_state_status,
            pr.mergeable.as_deref().unwrap_or("unknown"),
            pr.checks.passed,
            pr.checks.pending,
            pr.checks.failed,
            pr.checks.missing_required_names.len()
        );
        if !pr.checks.missing_required_names.is_empty() {
            println!(
                "  missing required checks: {}",
                pr.checks.missing_required_names.join(", ")
            );
        }
    }
    for issue in &report.blocker_issues {
        print!(
            "- issue #{}: {:?} state={}",
            issue.number, issue.status, issue.state
        );
        if let Some(reason) = &issue.waiver_reason {
            print!(" waiver=\"{reason}\"");
        }
        println!();
    }
    for finding in &report.findings {
        println!("- finding: {finding}");
    }
    if !report.next_actions.is_empty() {
        println!("next actions:");
        for action in &report.next_actions {
            println!("- {action}");
        }
    }
}
