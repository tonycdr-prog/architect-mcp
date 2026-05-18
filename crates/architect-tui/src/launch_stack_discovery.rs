use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::launch_stack::LaunchStackPullRequest;
use crate::launch_stack_github::{append_repo_args, pr_from_value, public_text, run_gh_json};

pub(crate) const STACK_DISCOVERY_PR_FIELDS: &str = "number,headRefName,baseRefName,state";

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
    discover_stack_from_pr_with_runner(workspace, repo, number, &run_gh_json)
}

pub(crate) fn discover_stack_from_pr_with_runner<R>(
    workspace: &Path,
    repo: Option<&str>,
    number: u64,
    run_gh: &R,
) -> Result<LaunchStackDiscovery, String>
where
    R: Fn(&Path, &[String]) -> Result<Value, String>,
{
    let mut current = fetch_stack_pr_by_number(workspace, repo, number, run_gh)?;
    if current.state != "OPEN" {
        return Err(format!(
            "stack discovery: PR #{number} is not an open PR in the selected repository"
        ));
    }

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
        let matches = fetch_stack_prs_by_head(workspace, repo, &base, run_gh)?;
        let matching_refs = matches.iter().collect::<Vec<_>>();
        let Some(next) = choose_open_record_for_base(&base, &matching_refs)? else {
            pull_requests.reverse();
            return Ok(LaunchStackDiscovery {
                from_pr: number,
                pull_requests,
                stopped_at_base: public_text(&base, 120),
            });
        };
        current = next.clone();
    }
}

fn fetch_stack_pr_by_number(
    workspace: &Path,
    repo: Option<&str>,
    number: u64,
    run_gh: &impl Fn(&Path, &[String]) -> Result<Value, String>,
) -> Result<LaunchStackPrRecord, String> {
    let mut args = vec![
        "pr".to_string(),
        "view".to_string(),
        number.to_string(),
        "--json".to_string(),
        STACK_DISCOVERY_PR_FIELDS.to_string(),
    ];
    append_repo_args(&mut args, repo);
    let value = run_gh(workspace, &args).map_err(|error| format!("stack discovery: {error}"))?;
    Ok(pr_record_from_value(&value))
}

fn fetch_stack_prs_by_head(
    workspace: &Path,
    repo: Option<&str>,
    head_ref_name: &str,
    run_gh: &impl Fn(&Path, &[String]) -> Result<Value, String>,
) -> Result<Vec<LaunchStackPrRecord>, String> {
    let mut args = vec![
        "pr".to_string(),
        "list".to_string(),
        "--state".to_string(),
        "all".to_string(),
        "--head".to_string(),
        head_ref_name.to_string(),
        "--limit".to_string(),
        "100".to_string(),
        "--json".to_string(),
        STACK_DISCOVERY_PR_FIELDS.to_string(),
    ];
    append_repo_args(&mut args, repo);
    let value = run_gh(workspace, &args).map_err(|error| format!("stack discovery: {error}"))?;
    let Some(records) = value.as_array() else {
        return Err("stack discovery: gh PR head lookup JSON was not an array".to_string());
    };
    Ok(records.iter().map(pr_record_from_value).collect::<Vec<_>>())
}

#[cfg(test)]
pub(crate) fn discover_stack_from_records(
    number: u64,
    records: &[LaunchStackPrRecord],
) -> Result<LaunchStackDiscovery, String> {
    let mut by_number: std::collections::BTreeMap<u64, &LaunchStackPrRecord> =
        std::collections::BTreeMap::new();
    let mut by_head: std::collections::BTreeMap<&str, Vec<&LaunchStackPrRecord>> =
        std::collections::BTreeMap::new();
    for record in records {
        by_head
            .entry(record.head_ref_name.as_str())
            .or_default()
            .push(record);
        if record.state == "OPEN" {
            by_number.insert(record.pr.number, record);
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
        let matches = by_head.get(base.as_str()).cloned().unwrap_or_default();
        let Some(next) = choose_open_record_for_base(&base, &matches)? else {
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

fn choose_open_record_for_base<'a>(
    base: &str,
    records: &[&'a LaunchStackPrRecord],
) -> Result<Option<&'a LaunchStackPrRecord>, String> {
    let open_records = records
        .iter()
        .copied()
        .filter(|record| record.state == "OPEN")
        .collect::<Vec<_>>();
    if open_records.len() > 1 {
        let numbers = open_records
            .iter()
            .map(|record| format!("#{}", record.pr.number))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "stack discovery: branch {} matches multiple open PR heads ({numbers})",
            public_text(base, 120)
        ));
    }
    if let Some(record) = open_records.first().copied() {
        return Ok(Some(record));
    }
    if !records.is_empty() {
        return Err(format!(
            "stack discovery: branch {} matches a PR head that is not open",
            public_text(base, 120)
        ));
    }
    Ok(None)
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
