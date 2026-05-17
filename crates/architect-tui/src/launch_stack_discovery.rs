use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::launch_stack::LaunchStackPullRequest;
use crate::launch_stack_github::{append_repo_args, pr_from_value, public_text, run_gh_json};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStackDiscovery {
    pub from_pr: u64,
    pub pull_requests: Vec<u64>,
    pub stopped_at_base: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LaunchStackPrRecord {
    pub pr: LaunchStackPullRequest,
    pub head_ref_name: String,
    pub base_ref_name: String,
    pub state: String,
}

pub(crate) fn resolve_launch_stack_pr_numbers(
    workspace: &Path,
    repo: Option<&str>,
    stack_from_pr: Option<u64>,
    explicit_prs: &[u64],
) -> (Vec<u64>, Option<LaunchStackDiscovery>, Vec<String>) {
    let mut findings = Vec::new();
    let mut stack_discovery = None;
    let mut pr_numbers = Vec::new();
    if let Some(head_pr) = stack_from_pr {
        match discover_stack_from_pr(workspace, repo, head_pr) {
            Ok(discovery) => {
                pr_numbers.extend(discovery.pull_requests.iter().copied());
                stack_discovery = Some(discovery);
            }
            Err(error) => findings.push(error),
        }
    }
    pr_numbers.extend(explicit_prs.iter().copied());
    dedupe_numbers(&mut pr_numbers);
    (pr_numbers, stack_discovery, findings)
}

fn discover_stack_from_pr(
    workspace: &Path,
    repo: Option<&str>,
    number: u64,
) -> Result<LaunchStackDiscovery, String> {
    let mut args = vec![
        "pr".to_string(),
        "list".to_string(),
        "--state".to_string(),
        "open".to_string(),
        "--limit".to_string(),
        "200".to_string(),
        "--json".to_string(),
        "number,title,url,isDraft,mergeStateStatus,statusCheckRollup,headRefName,baseRefName,state"
            .to_string(),
    ];
    append_repo_args(&mut args, repo);
    let value =
        run_gh_json(workspace, &args).map_err(|error| format!("stack discovery: {error}"))?;
    let Some(records) = value.as_array() else {
        return Err("stack discovery: gh PR list JSON was not an array".to_string());
    };
    let records = records.iter().map(pr_record_from_value).collect::<Vec<_>>();
    discover_stack_from_records(number, &records)
}

pub(crate) fn discover_stack_from_records(
    number: u64,
    records: &[LaunchStackPrRecord],
) -> Result<LaunchStackDiscovery, String> {
    let mut by_number: BTreeMap<u64, &LaunchStackPrRecord> = BTreeMap::new();
    let mut by_head: BTreeMap<&str, &LaunchStackPrRecord> = BTreeMap::new();
    for record in records {
        if record.state == "OPEN" {
            by_number.insert(record.pr.number, record);
            by_head.insert(record.head_ref_name.as_str(), record);
        }
    }

    let Some(mut current) = by_number.get(&number).copied() else {
        return Err(format!(
            "stack discovery: PR #{number} is not an open PR in the selected repository"
        ));
    };

    let mut seen = BTreeSet::new();
    let mut pull_requests = Vec::new();
    loop {
        if !seen.insert(current.pr.number) {
            return Err(format!(
                "stack discovery: cycle detected while following PR #{}",
                current.pr.number
            ));
        }
        pull_requests.push(current.pr.number);
        let base = current.base_ref_name.clone();
        let Some(next) = by_head.get(base.as_str()).copied() else {
            pull_requests.reverse();
            return Ok(LaunchStackDiscovery {
                from_pr: number,
                pull_requests,
                stopped_at_base: public_text(&base, 120),
            });
        };
        current = next;
    }
}

pub(crate) fn pr_record_from_value(value: &Value) -> LaunchStackPrRecord {
    LaunchStackPrRecord {
        pr: pr_from_value(0, value),
        head_ref_name: public_text(
            value
                .get("headRefName")
                .and_then(Value::as_str)
                .unwrap_or(""),
            180,
        ),
        base_ref_name: public_text(
            value
                .get("baseRefName")
                .and_then(Value::as_str)
                .unwrap_or(""),
            180,
        ),
        state: public_text(
            &value
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN")
                .to_ascii_uppercase(),
            80,
        ),
    }
}

fn dedupe_numbers(numbers: &mut Vec<u64>) {
    let mut seen = HashSet::new();
    numbers.retain(|number| seen.insert(*number));
}
