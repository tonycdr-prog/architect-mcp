use crate::issue_terminal_evidence::IssueTerminalEvidenceReport;

pub(crate) fn print_text_report(report: &IssueTerminalEvidenceReport) {
    println!(
        "architect-mcp-tui issue terminal evidence: {:?}",
        report.result
    );
    println!("- issue #{}: {}", report.issue.number, report.issue.title);
    println!("- extracted blocks: {}", report.extracted_blocks.len());
    println!("- reports: {}", report.terminal_evidence.reports.len());
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
