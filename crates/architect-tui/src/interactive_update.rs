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
        format!("verification: {}", session.verification.len()),
        format!("approval: {:?}", session.approval_status),
        format!("changed files: {}", session.changed_files.len()),
        format!("arena candidates: {}", session.arena_candidates.len()),
    ];
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
    lines
}

pub(crate) fn help_update() -> WorkflowUpdate {
    update(
        vec![
            "commands: new app <idea>, resume <session-id>, answer key=value, grill, contract, review plan, review files, run adapter, diff summary, diff file <path>, approve [reason], reject [reason], override [reason], promote, arena run <adapter[,adapter]>, arena rank, record verification check=status, final review <text>, session review".to_string(),
        ],
        Vec::new(),
        None,
    )
}
