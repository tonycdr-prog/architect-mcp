use std::fs;
use std::path::Path;

use crate::governance_audit_report::GovernanceMemoryProposal;
use crate::governance_audit_support::{package_scripts, read_to_string};
use crate::governance_secret_scan::looks_secret_like;

pub(crate) fn memory_proposals(workspace: &Path) -> Vec<GovernanceMemoryProposal> {
    let scope = repo_scope(workspace);
    let mut proposals = Vec::new();
    let scripts = package_scripts(workspace).unwrap_or_default();

    if scripts.contains_key("release:check") {
        proposals.push(proposal(
            &scope,
            "package.json",
            &format!(
                "{scope} release gate: npm run release:check is the release-sensitive gate. Verify current package.json before relying on it. Review when release workflow changes."
            ),
        ));
    }

    let agents = read_to_string(workspace, "AGENTS.md");
    let agents_lower = agents.to_ascii_lowercase();
    if !agents.is_empty() && agents_lower.contains("memory") {
        proposals.push(proposal(
            &scope,
            "AGENTS.md",
            &format!(
                "{scope} memory policy lives in AGENTS.md. Store only durable project context and re-read AGENTS.md before writing memory."
            ),
        ));
    }

    if workspace.join("docs/goal-ai-software-foundry.md").is_file() {
        proposals.push(proposal(
            &scope,
            "docs/goal-ai-software-foundry.md",
            &format!(
                "{scope} tracks its long-running AI software delivery goal in docs/goal-ai-software-foundry.md. Verify the latest roadmap and judge status before implementation."
            ),
        ));
    } else if workspace.join("docs/architecture-contract.md").is_file() {
        proposals.push(proposal(
            &scope,
            "docs/architecture-contract.md",
            &format!(
                "{scope} keeps an architecture contract in docs/architecture-contract.md. Verify it before implementation and update it through reviewed changes."
            ),
        ));
    }

    filter_memory_proposals(proposals)
}

pub(crate) fn filter_memory_proposals(
    proposals: Vec<GovernanceMemoryProposal>,
) -> Vec<GovernanceMemoryProposal> {
    proposals
        .into_iter()
        .map(|mut proposal| {
            let lower = proposal.text.to_ascii_lowercase();
            let unsafe_reason = if looks_secret_like(&proposal.text) {
                Some("secret-shaped text")
            } else if lower.contains("raw chat")
                || lower.contains("conversation log")
                || lower.contains("full chat transcript")
                || lower.contains("chat transcript")
            {
                Some("raw conversation log or transcript")
            } else if lower.contains("transient task") || lower.contains("temporary detail") {
                Some("transient task detail")
            } else if lower.contains("customer data")
                || lower.contains("private customer")
                || lower.contains("customer record")
            {
                Some("private customer data")
            } else if lower.contains("speculative guess")
                || lower.contains("speculative detail")
                || lower.contains("not yet confirmed")
            {
                Some("speculative content")
            } else if lower.contains("sensitive security finding")
                || lower.contains("exploit detail")
                || lower.contains("vulnerability detail")
            {
                Some("sensitive security finding")
            } else {
                None
            };
            if let Some(reason) = unsafe_reason {
                proposal.safe_to_store = false;
                proposal.review_note = format!("rejected: {reason}");
            }
            proposal
        })
        .filter(|proposal| proposal.safe_to_store)
        .collect()
}

fn proposal(scope: &str, source: &str, text: &str) -> GovernanceMemoryProposal {
    GovernanceMemoryProposal {
        scope: scope.to_string(),
        source: source.to_string(),
        text: text.to_string(),
        safe_to_store: true,
        review_note: "safe durable project context; prohibited data omitted".to_string(),
    }
}

fn repo_scope(workspace: &Path) -> String {
    package_name(workspace)
        .or_else(|| git_remote_scope(workspace))
        .or_else(|| {
            workspace
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "workspace".to_string())
}

fn package_name(workspace: &Path) -> Option<String> {
    let package = fs::read_to_string(workspace.join("package.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&package).ok()?;
    value
        .get("name")
        .and_then(serde_json::Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .map(ToString::to_string)
}

fn git_remote_scope(workspace: &Path) -> Option<String> {
    let config = fs::read_to_string(workspace.join(".git").join("config")).ok()?;
    config.lines().find_map(|line| {
        let value = line.trim().strip_prefix("url = ")?;
        parse_github_repo(value)
    })
}

fn parse_github_repo(value: &str) -> Option<String> {
    let normalized = value.trim().trim_end_matches(".git");
    let repo = normalized
        .strip_prefix("https://github.com/")
        .or_else(|| normalized.strip_prefix("git@github.com:"))?;
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    if owner.is_empty() || name.is_empty() {
        return None;
    }
    Some(format!("{owner}/{name}"))
}
