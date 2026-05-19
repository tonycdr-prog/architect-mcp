use std::collections::BTreeMap;

use crate::launch_stack::{LaunchStackIssue, LaunchStackItemStatus};
use crate::launch_stack_github::public_text;

pub(crate) fn apply_waivers(
    blocker_issues: &mut [LaunchStackIssue],
    waivers: &BTreeMap<u64, String>,
) {
    for issue in blocker_issues {
        if issue.status == LaunchStackItemStatus::Warning
            && issue.state == "OPEN"
            && let Some(reason) = waivers.get(&issue.number)
        {
            issue.status = LaunchStackItemStatus::Waived;
            issue.waiver_reason = Some(reason.clone());
            issue.next_action = None;
        }
    }
}

pub(crate) fn parse_waivers(raw_waivers: &[String]) -> (BTreeMap<u64, String>, Vec<String>) {
    let mut waivers = BTreeMap::new();
    let mut findings = Vec::new();
    for raw in raw_waivers {
        match parse_waiver(raw) {
            Ok((number, reason)) => {
                if waivers.insert(number, reason).is_some() {
                    findings.push(format!("duplicate waiver supplied for issue #{number}"));
                }
            }
            Err(error) => findings.push(error),
        }
    }
    (waivers, findings)
}

fn parse_waiver(raw: &str) -> Result<(u64, String), String> {
    let Some((number, reason)) = raw.split_once('=').or_else(|| raw.split_once(':')) else {
        return Err(
            "blocker waiver must use ISSUE=reason or ISSUE:reason with a public reason".to_string(),
        );
    };
    let number = number
        .trim()
        .trim_start_matches('#')
        .parse::<u64>()
        .map_err(|_| "blocker waiver issue number must be numeric".to_string())?;
    let reason = public_text(reason, 240);
    if reason.is_empty() {
        return Err(format!(
            "blocker waiver for issue #{number} needs a public reason"
        ));
    }
    Ok((number, reason))
}
