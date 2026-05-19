use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::gate_calls::call_gate;
use crate::headless_support::verification_checks;
use crate::interactive::InteractiveWorkflowEngine;
use crate::mcp::StdioMcpClient;
use crate::session::TuiSession;
use crate::untrusted_input::adapter_output_label;

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
        Some("crashed") => session.record_adapter_run_issue(
            agent_event
                .get("message")
                .and_then(Value::as_str)
                .map(|message| format!("adapter crashed: {message}"))
                .unwrap_or_else(|| "adapter crashed".to_string()),
        ),
        Some("timed_out") => session.record_adapter_run_issue("adapter timed out"),
        Some("cancelled") => session.record_adapter_run_issue("adapter cancelled"),
        Some("completed") => {
            if let Some(exit_code) = agent_event.get("exit_code").and_then(Value::as_i64)
                && exit_code != 0
            {
                session.record_adapter_run_issue(format!("adapter exited with code {exit_code}"));
            }
        }
        Some("output") => {
            session.add_untrusted_inputs([adapter_output_label()]);
            if agent_event.get("truncated").and_then(Value::as_bool) == Some(true) {
                session.record_adapter_run_issue("adapter output truncated");
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
        assert_eq!(session.adapter_run_issues, vec!["adapter crashed: boom"]);
    }

    #[test]
    fn run_evidence_records_timeout_exit_and_truncation_issues() {
        let mut session = TuiSession::new("build", "shell");
        apply_run_evidence(
            &mut session,
            r#"{"type":"agent_event","event":{"type":"timed_out"}}
{"type":"agent_event","event":{"type":"completed","exit_code":2}}
{"type":"agent_event","event":{"type":"output","stream":"pty","text":"...","truncated":true}}"#,
        );

        assert_eq!(
            session.adapter_run_issues,
            vec![
                "adapter timed out",
                "adapter exited with code 2",
                "adapter output truncated"
            ]
        );
        assert!(
            session
                .untrusted_inputs
                .iter()
                .any(|input| input.label == "adapter output")
        );
    }

    #[test]
    fn run_evidence_labels_mcp_results_as_untrusted_tool_data() {
        let mut session = TuiSession::new("build", "shell");
        apply_run_evidence(
            &mut session,
            r#"{"type":"mcp_result","name":"review_agent_session","result":{"ok":true}}"#,
        );

        assert!(
            session
                .untrusted_inputs
                .iter()
                .any(|input| input.label == "mcp response")
        );
    }
}
