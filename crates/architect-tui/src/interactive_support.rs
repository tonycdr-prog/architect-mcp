use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::gate_calls::call_gate;
use crate::headless_support::verification_checks;
use crate::interactive::InteractiveWorkflowEngine;
use crate::mcp::StdioMcpClient;
use crate::session::TuiSession;

impl InteractiveWorkflowEngine {
    pub(crate) async fn call_tool(&self, name: &str, purpose: &str, args: Value) -> Result<Value> {
        let mut output = Vec::new();
        let mut client = StdioMcpClient::connect(&self.orchestrator.bridge)?;
        client.list_tools()?;
        call_gate(&mut client, &mut output, true, name, purpose, args)
            .await?
            .with_context(|| format!("{name} did not return a usable result"))
    }

    pub(crate) fn gate_inputs(&self) -> Result<(String, Value, Vec<String>)> {
        let session = self.active()?;
        let grill = self.grill_gate()?;
        let plan = grill.get("buildPlan").cloned().unwrap_or_else(|| json!({}));
        Ok((
            session.prompt.clone(),
            grill.clone(),
            verification_checks(&grill, &plan),
        ))
    }

    pub(crate) fn grill_gate(&self) -> Result<Value> {
        self.active()?
            .gates
            .get("grill_me")
            .cloned()
            .context("run grill before this command")
    }

    pub(crate) fn active(&self) -> Result<&TuiSession> {
        self.active.as_ref().context("start with: new app <idea>")
    }

    pub(crate) fn update_active<F>(&mut self, change: F) -> Result<&TuiSession>
    where
        F: FnOnce(&mut TuiSession),
    {
        let mut session = self.active.take().context("start with: new app <idea>")?;
        change(&mut session);
        self.store.save(&mut session)?;
        self.active = Some(session);
        self.active()
    }
}

pub(crate) fn apply_run_evidence(session: &mut TuiSession, events: &str) {
    for line in events.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if event.get("type").and_then(Value::as_str) == Some("diff_evidence") {
            if let Some(worktree) = event.get("worktree").and_then(Value::as_str) {
                session.worktree = Some(worktree.into());
            }
            session.diff_stat = event
                .get("diff_stat")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            session.changed_files = event
                .get("changed_files")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
        }
        if event.get("type").and_then(Value::as_str) == Some("mcp_result")
            && let Some(name) = event.get("name").and_then(Value::as_str)
        {
            let result = event.get("result").cloned().unwrap_or(Value::Null);
            session.set_gate(name, result);
        }
        if event.get("type").and_then(Value::as_str) == Some("agent_event") {
            apply_agent_event(session, &event);
        }
    }
}

fn apply_agent_event(session: &mut TuiSession, event: &Value) {
    let Some(agent_event) = event.get("event") else {
        return;
    };
    match agent_event.get("type").and_then(Value::as_str) {
        Some("crashed" | "timed_out" | "cancelled") => session.adapter_crashed = true,
        Some("completed") => {
            if agent_event
                .get("exit_code")
                .is_some_and(|code| !matches!(code.as_i64(), Some(0)))
            {
                session.adapter_crashed = true;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_evidence_marks_crashed_adapter_for_arena_ranking() {
        let mut session = TuiSession::new("build", "shell");
        apply_run_evidence(
            &mut session,
            r#"{"type":"agent_event","event":{"type":"crashed","message":"boom"}}"#,
        );
        assert!(session.adapter_crashed);
    }
}
