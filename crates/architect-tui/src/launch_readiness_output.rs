use crate::launch_readiness::LaunchReadinessReport;

pub(crate) fn print_text_report(report: &LaunchReadinessReport) {
    println!("architect-mcp-tui launch readiness: {:?}", report.result);
    println!("- read-only: {}", report.read_only);
    println!("- stack: {:?}", report.launch_stack.result);
    match &report.terminal_evidence_issue {
        Some(evidence) => {
            println!("- terminal evidence issue: {:?}", evidence.result);
            println!(
                "- terminal evidence reports: {}",
                evidence.terminal_evidence.reports.len()
            );
        }
        None => {
            println!("- terminal evidence issue: not supplied");
        }
    }
    if let Some(waiver) = &report.terminal_evidence_waiver {
        let status = if waiver.applied {
            "applied"
        } else {
            "recorded"
        };
        println!(
            "- terminal evidence waiver: issue #{} {status}: {}",
            waiver.issue, waiver.reason
        );
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
