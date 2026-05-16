use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::config::TuiConfig;
use crate::mcp::CORE_WORK_GATE_TOOLS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcpSessionStatus {
    Active,
    CancelRequested,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcpSession {
    pub id: String,
    pub adapter: String,
    pub mode: String,
    pub concurrency: usize,
    pub worktree_isolation: bool,
    pub approval_policy: String,
    pub status: AcpSessionStatus,
    pub cancellation_requested: bool,
    pub events: Vec<Value>,
}

#[derive(Debug, Default)]
pub struct AcpState {
    sessions: BTreeMap<String, AcpSession>,
}

impl AcpState {
    pub fn create_session(&mut self, config: &TuiConfig, params: &Value) -> AcpSession {
        let adapter = params
            .get("adapter")
            .and_then(Value::as_str)
            .unwrap_or(&config.agents.default_adapter)
            .to_string();
        let mode = params
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("new-app")
            .to_string();
        let concurrency = params
            .get("concurrency")
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        let session = AcpSession {
            id: Uuid::new_v4().to_string(),
            adapter,
            mode,
            concurrency,
            worktree_isolation: config.agents.worktree_isolation,
            approval_policy: config.agents.approval_policy.clone(),
            status: AcpSessionStatus::Active,
            cancellation_requested: false,
            events: vec![json!({
                "type": "session_created",
                "message": "session created"
            })],
        };
        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    pub fn session(&self, id: &str) -> Result<&AcpSession> {
        self.sessions
            .get(id)
            .with_context(|| format!("unknown ACP session '{id}'"))
    }

    pub fn record_prompt_turn(&mut self, id: &str, prompt: &str) -> Result<Vec<Value>> {
        let session = self
            .sessions
            .get_mut(id)
            .with_context(|| format!("unknown ACP session '{id}'"))?;
        if session.cancellation_requested {
            anyhow::bail!("ACP session '{id}' is cancelled");
        }
        let events = vec![
            json!({
                "type": "plan_update",
                "sessionId": id,
                "message": "grill and contract required before edits"
            }),
            json!({
                "type": "tool",
                "sessionId": id,
                "name": CORE_WORK_GATE_TOOLS[0]
            }),
            json!({
                "type": "terminal",
                "sessionId": id,
                "stream": "status",
                "text": format!("prompt accepted for {} adapter; execution pending approval", session.adapter)
            }),
            json!({
                "type": "prompt",
                "sessionId": id,
                "text": prompt
            }),
        ];
        session.events.extend(events.clone());
        Ok(events)
    }

    pub fn cancel(&mut self, id: &str) -> Result<Value> {
        let session = self
            .sessions
            .get_mut(id)
            .with_context(|| format!("unknown ACP session '{id}'"))?;
        session.status = AcpSessionStatus::Cancelled;
        session.cancellation_requested = true;
        let event = json!({
            "type": "cancelled",
            "sessionId": id,
            "message": "session cancellation requested"
        });
        session.events.push(event.clone());
        Ok(event)
    }
}
