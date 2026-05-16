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
    vec![
        format!("session: {}", session.id),
        format!("phase: {:?}", session.phase),
        format!("adapter: {}", session.adapter),
        format!("gates: {}", session.gates.len()),
        format!("verification: {}", session.verification.len()),
    ]
}

pub(crate) fn help_update() -> WorkflowUpdate {
    update(
        vec![
            "commands: new app <idea>, answer key=value, grill, contract, review plan, review files, run adapter, record verification check=status, final review <text>, session review".to_string(),
        ],
        Vec::new(),
        None,
    )
}
