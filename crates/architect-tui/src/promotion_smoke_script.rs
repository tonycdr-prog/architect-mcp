use anyhow::Result;

use crate::interactive::InteractiveWorkflowEngine;
use crate::promotion_smoke_report::{PromotionSmokeVerificationReport, session_changed_files};

pub(crate) fn setup_commands(adapter: &str) -> Vec<String> {
    vec![
        "new app ready documentation-only Codex adapter promotion smoke that creates docs/codex-adapter-smoke.md only".to_string(),
        "answer users=architect-mcp maintainer validating real adapter promotion".to_string(),
        "answer coreFlows=create docs/codex-adapter-smoke.md only; do not create app code".to_string(),
        "answer stack=Markdown documentation artifact in an isolated git worktree".to_string(),
        "answer storage=no runtime storage; one markdown file committed only after promotion".to_string(),
        "answer enforcement=manual TUI approval gates before adapter execution and before promotion".to_string(),
        "answer repoLayout=docs/codex-adapter-smoke.md is the only intended changed file".to_string(),
        "answer constraints=create only docs/codex-adapter-smoke.md; include the exact text codex-adapter-smoke; do not change README, source files, package files, configs, CI, or AGENTS.md".to_string(),
        "answer dataEntities=none".to_string(),
        "answer risk=unwanted mutation outside the isolated worktree".to_string(),
        "answer verification=npm test; review_repo_structure".to_string(),
        "grill".to_string(),
        "contract".to_string(),
        "review plan".to_string(),
        "review files".to_string(),
        format!("approve run {adapter} adapter in isolated worktree"),
        "run adapter".to_string(),
        "diff summary".to_string(),
        "diff file docs/codex-adapter-smoke.md".to_string(),
        "verification status".to_string(),
    ]
}

pub(crate) fn finish_commands(
    engine: &InteractiveWorkflowEngine,
    verification: &[PromotionSmokeVerificationReport],
) -> Result<Vec<String>> {
    let session = engine.active()?;
    let files = session_changed_files(session);
    let file_summary = if files.is_empty() {
        "none".to_string()
    } else {
        files.join(", ")
    };
    let verification_summary = verification
        .iter()
        .map(|record| format!("{} {}", record.check, record.status))
        .collect::<Vec<_>>()
        .join("; ");
    Ok(vec![
        format!(
            "final review Changed files: {file_summary}. Verification: {verification_summary}. Assumptions: real adapter executed in a disposable isolated worktree. Not done: no product app was created by this smoke."
        ),
        "session review".to_string(),
        "promotion status".to_string(),
        "approve promote real adapter smoke reviewed diff and verification".to_string(),
        "promote".to_string(),
    ])
}
