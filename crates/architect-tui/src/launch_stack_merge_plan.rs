use crate::launch_stack::{LaunchStackItemStatus, LaunchStackReport};
use crate::launch_stack_github::public_text;

pub(crate) fn print_launch_stack_merge_plan(report: &LaunchStackReport) {
    for line in render_launch_stack_merge_plan(report) {
        println!("{line}");
    }
}

pub(crate) fn render_launch_stack_merge_plan(report: &LaunchStackReport) -> Vec<String> {
    let mut lines = vec![
        format!(
            "architect-mcp-tui launch stack merge plan: {:?}",
            report.result
        ),
        "- read-only: true".to_string(),
        "- mutation: none; this checklist does not merge PRs, close issues, or edit branches"
            .to_string(),
    ];
    if let Some(repository) = &report.repository {
        lines.push(format!("- repository: {}", public_text(repository, 160)));
    }
    if let Some(discovery) = &report.stack_discovery {
        lines.push(format!(
            "- discovered from PR #{} and stopped at {}",
            discovery.from_pr,
            public_text(&discovery.stopped_at_base, 120)
        ));
    }

    let discovered_order = has_complete_discovered_order(report);
    if discovered_order {
        lines.push("merge order, base to head:".to_string());
    } else {
        lines.push("pull requests, supplied order:".to_string());
    }
    if report.pull_requests.is_empty() {
        lines.push("- no pull requests supplied".to_string());
    } else {
        for (index, pr) in report.pull_requests.iter().enumerate() {
            let hold = if pr.status == LaunchStackItemStatus::Passed {
                "ready"
            } else {
                "hold"
            };
            lines.push(format!(
                "{}. PR #{} [{}] {} ({:?}, review={}, unresolved_threads={}, merge={}, mergeable={}, checks {}/{}/{})",
                index + 1,
                pr.number,
                hold,
                public_text(&pr.title, 120),
                pr.status,
                pr.review_decision.as_deref().unwrap_or("none"),
                pr.unresolved_review_threads,
                public_text(&pr.merge_state_status, 80),
                pr.mergeable.as_deref().unwrap_or("unknown"),
                pr.checks.passed,
                pr.checks.pending,
                pr.checks.failed
            ));
            if let Some(action) = &pr.next_action {
                lines.push(format!("   next: {}", public_text(action, 180)));
            }
            if !pr.checks.missing_required_names.is_empty() {
                lines.push(format!(
                    "   missing required checks: {}",
                    public_text(&pr.checks.missing_required_names.join(", "), 220)
                ));
            }
        }
    }

    if !report.blocker_issues.is_empty() {
        lines.push("external blockers:".to_string());
        for issue in &report.blocker_issues {
            let waiver = issue
                .waiver_reason
                .as_ref()
                .map(|reason| format!(" waiver=\"{}\"", public_text(reason, 160)))
                .unwrap_or_default();
            lines.push(format!(
                "- issue #{} {:?} state={}{}: {}",
                issue.number,
                issue.status,
                public_text(&issue.state, 80),
                waiver,
                public_text(&issue.title, 120)
            ));
            if let Some(action) = &issue.next_action {
                lines.push(format!("  next: {}", public_text(action, 180)));
            }
        }
    }

    for finding in &report.findings {
        lines.push(format!("- finding: {}", public_text(finding, 240)));
    }

    lines.push("operator checklist:".to_string());
    if discovered_order {
        lines.push("- merge manually from the first ready PR in the listed order".to_string());
    } else {
        lines.push("- verify the true base-to-head order before any manual merge".to_string());
    }
    lines.push(
        "- re-run launch-stack after each manual merge because base branches change".to_string(),
    );
    lines.push(
        "- do not tag, publish, or claim launch go until blockers and terminal evidence are resolved or explicitly waived"
            .to_string(),
    );
    if !report.next_actions.is_empty() {
        lines.push("next actions before final launch go:".to_string());
        for action in &report.next_actions {
            lines.push(format!("- {}", public_text(action, 220)));
        }
    }

    lines
}

fn has_complete_discovered_order(report: &LaunchStackReport) -> bool {
    let listed = report
        .pull_requests
        .iter()
        .map(|pr| pr.number)
        .collect::<Vec<_>>();
    report
        .stack_discovery
        .as_ref()
        .is_some_and(|discovery| discovery.pull_requests == listed)
}
