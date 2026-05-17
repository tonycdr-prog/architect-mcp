use std::path::Path;

use crate::governance_audit_report::GovernanceMemoryProposal;
use crate::governance_audit_support::looks_secret_like;

pub(crate) fn memory_proposals(_workspace: &Path) -> Vec<GovernanceMemoryProposal> {
    filter_memory_proposals(vec![
        proposal(
            "architect-mcp",
            "docs/goal-ai-software-foundry.md",
            "architect-mcp goal: local-first AI software delivery control layer; human owns decisions, agents propose and execute, architect-mcp governs. Review by 2026-08-17.",
        ),
        proposal(
            "architect-mcp",
            "package.json",
            "architect-mcp release gate: npm run release:check is the clean-checkout release-sensitive gate. Review when release workflow changes.",
        ),
        proposal(
            "architect-mcp",
            "AGENTS.md",
            "architect-mcp memory policy: durable project context only; credentials, conversation transcripts, customer data, and temporary run notes stay out of memory.",
        ),
    ])
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
            } else if lower.contains("raw chat") || lower.contains("conversation log") {
                Some("raw conversation log")
            } else if lower.contains("transient task") || lower.contains("temporary detail") {
                Some("transient task detail")
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
