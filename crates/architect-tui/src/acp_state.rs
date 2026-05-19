use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::config::TuiConfig;
use crate::mcp::CORE_WORK_GATE_TOOLS;

pub const ACP_MODES: [&str; 4] = ["new-app", "coding-task", "automation", "arena"];
pub const ACP_MIN_CONCURRENCY: usize = 1;
pub const ACP_MAX_CONCURRENCY: usize = 8;
const ACP_SESSION_PARAMS: [&str; 5] = [
    "adapter",
    "mode",
    "concurrency",
    "worktreeIsolation",
    "approvalPolicy",
];

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
    pub fn create_session(&mut self, config: &TuiConfig, params: &Value) -> Result<AcpSession> {
        if !params.is_object() {
            anyhow::bail!("ACP session params must be an object");
        }
        reject_unknown_params(params, &ACP_SESSION_PARAMS)?;

        let adapter = optional_string(params, "adapter")?
            .unwrap_or(&config.agents.default_adapter)
            .to_string();
        if !config.adapters.contains_key(&adapter) {
            anyhow::bail!("unknown ACP adapter '{adapter}'");
        }

        let mode = optional_string(params, "mode")?
            .unwrap_or(ACP_MODES[0])
            .to_string();
        if !ACP_MODES.contains(&mode.as_str()) {
            anyhow::bail!("unsupported ACP mode '{mode}'");
        }

        let concurrency = optional_usize(params, "concurrency")?.unwrap_or(ACP_MIN_CONCURRENCY);
        if !(ACP_MIN_CONCURRENCY..=ACP_MAX_CONCURRENCY).contains(&concurrency) {
            anyhow::bail!(
                "ACP concurrency must be between {ACP_MIN_CONCURRENCY} and {ACP_MAX_CONCURRENCY}"
            );
        }

        if let Some(worktree_isolation) = optional_bool(params, "worktreeIsolation")?
            && worktree_isolation != config.agents.worktree_isolation
        {
            anyhow::bail!(
                "ACP sessions cannot override configured worktreeIsolation={}",
                config.agents.worktree_isolation
            );
        }

        if let Some(approval_policy) = optional_string(params, "approvalPolicy")?
            && approval_policy != config.agents.approval_policy
        {
            anyhow::bail!(
                "ACP sessions cannot override configured approvalPolicy={}",
                config.agents.approval_policy
            );
        }

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
        Ok(session)
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

fn optional_string<'a>(params: &'a Value, name: &str) -> Result<Option<&'a str>> {
    match params.get(name) {
        Some(value) => value
            .as_str()
            .map(Some)
            .with_context(|| format!("ACP parameter '{name}' must be a string")),
        None => Ok(None),
    }
}

fn optional_usize(params: &Value, name: &str) -> Result<Option<usize>> {
    match params.get(name) {
        Some(value) => value
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .map(Some)
            .with_context(|| format!("ACP parameter '{name}' must be a positive integer")),
        None => Ok(None),
    }
}

fn optional_bool(params: &Value, name: &str) -> Result<Option<bool>> {
    match params.get(name) {
        Some(value) => value
            .as_bool()
            .map(Some)
            .with_context(|| format!("ACP parameter '{name}' must be a boolean")),
        None => Ok(None),
    }
}

fn reject_unknown_params(params: &Value, allowed: &[&str]) -> Result<()> {
    let Some(object) = params.as_object() else {
        return Ok(());
    };
    for name in object.keys() {
        if !allowed.contains(&name.as_str()) {
            anyhow::bail!("unsupported ACP session parameter '{name}'");
        }
    }
    Ok(())
}
