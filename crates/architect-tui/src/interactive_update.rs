use crate::session::TuiSession;

#[derive(Debug, Clone)]
pub struct WorkflowUpdate {
    pub transcript: Vec<String>,
    pub inspector: Vec<String>,
    pub session: Option<TuiSession>,
}

pub(crate) fn update(
    transcript: Vec<String>,
    inspector: Vec<String>,
    session: Option<TuiSession>,
) -> WorkflowUpdate {
    WorkflowUpdate {
        transcript,
        inspector,
        session,
    }
}

pub(crate) fn gate_line(name: &str, ready: bool) -> String {
    format!("{name}: {}", if ready { "ready" } else { "needs input" })
}

pub(crate) fn inspector_for(session: &TuiSession) -> Vec<String> {
    let mut lines = vec![
        format!("session: {}", session.id),
        format!("phase: {:?}", session.phase),
        format!("adapter: {}", session.adapter),
        format!("gates: {}", session.gates.len()),
        format!(
            "verification: {}/{}",
            session.verification.len(),
            session.required_verification.len()
        ),
        format!("execution approved: {}", session.execution_approved),
        format!("approval: {:?}", session.approval_status),
        format!("promotion receipt: {}", session.promotion_receipt.is_some()),
        format!("changed files: {}", session.changed_files.len()),
        format!("adapter issues: {}", session.adapter_run_issues.len()),
        format!("arena candidates: {}", session.arena_candidates.len()),
        format!(
            "mcp recommendation: {}",
            session
                .mcp_recommendation
                .as_ref()
                .and_then(|value| value.get("status"))
                .and_then(|value| value.as_str())
                .unwrap_or("none")
        ),
        format!("mcp install approved: {}", session.mcp_install_approved),
        format!(
            "foundry plan: {}",
            session
                .foundry_plan
                .as_ref()
                .map(|plan| plan.repo_name.as_str())
                .unwrap_or("none")
        ),
        format!("foundry approved: {}", session.foundry_approved),
    ];
    if !session.adapter_run_issues.is_empty() {
        lines.push("adapter issue detail:".to_string());
        lines.extend(
            session
                .adapter_run_issues
                .iter()
                .take(3)
                .map(|issue| format!("  {issue}")),
        );
    }
    if let Some(diff_stat) = &session.diff_stat {
        lines.push("diff stat:".to_string());
        lines.extend(diff_stat.lines().take(4).map(|line| format!("  {line}")));
    }
    if !session.arena_candidates.is_empty() {
        lines.push("arena:".to_string());
        lines.extend(session.arena_candidates.iter().map(|candidate| {
            format!(
                "  {} {:?} files={}",
                candidate.adapter,
                candidate.review_status,
                candidate.changed_files.len()
            )
        }));
    }
    if let Some(plan) = &session.mcp_install_plan {
        lines.push(format!(
            "mcp plan: {} -> {}",
            plan.get("serverId")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown"),
            plan.get("targetClient")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
        ));
    }
    if let Some(plan) = &session.foundry_plan {
        lines.push(format!("foundry repo: {}", plan.repo_name));
        lines.push("foundry visibility: private".to_string());
    }
    lines
}

pub(crate) fn help_update() -> WorkflowUpdate {
    update(
        vec![
            "commands: new app <idea>, resume <session-id>, answer key=value, grill, contract, review plan, review files, run adapter, diff summary, diff file <path>, approve [reason], reject [reason], override <reason>, promotion status, promote, arena run <adapter[,adapter]>, arena rank, arena select <adapter>, integrations recommend [context], integrations plan <server> [target], integrations review, integrations apply [path], integrations approve <reason>, integrations write [path], foundry plan <repo> [owner=name], foundry status, foundry approve <reason>, foundry stage, foundry create, foundry create --execute, verification status, record verification <required check>=passed, final review <text>, session review".to_string(),
        ],
        Vec::new(),
        None,
    )
}
