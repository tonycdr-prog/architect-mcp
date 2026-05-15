use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

use crate::config::TuiConfig;
use crate::mcp::CORE_WORK_GATE_TOOLS;

pub fn acp_sdk_marker() -> &'static str {
    std::any::type_name::<agent_client_protocol::schema::ProtocolVersion>()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpSession {
    pub id: String,
    pub adapter: String,
    pub mode: String,
    pub concurrency: usize,
    pub worktree_isolation: bool,
    pub approval_policy: String,
}

#[derive(Debug, Default)]
pub struct AcpState {
    sessions: BTreeMap<String, AcpSession>,
}

pub async fn run_acp_stdio(config: TuiConfig) -> Result<()> {
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = io::stdout();
    let mut state = AcpState::default();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line)?;
        if let Some(response) = handle_json_rpc_value(&mut state, &config, request) {
            stdout
                .write_all(serde_json::to_string(&response)?.as_bytes())
                .await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

pub fn handle_json_rpc_value(
    state: &mut AcpState,
    config: &TuiConfig,
    request: Value,
) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

    let result = match method {
        "initialize" => json!({
            "protocolVersion": "1",
            "serverInfo": {
                "name": "architect-mcp-tui",
                "version": env!("CARGO_PKG_VERSION"),
                "sdk": acp_sdk_marker()
            },
            "capabilities": {
                "sessions": true,
                "terminal": true,
                "tools": true,
                "cancellation": true,
                "config": true
            }
        }),
        "config/options" => json!({
            "adapters": config.adapters.keys().collect::<Vec<_>>(),
            "modes": ["new-app", "coding-task", "automation", "arena"],
            "concurrency": { "min": 1, "max": 8, "default": 1 },
            "worktreeIsolation": config.agents.worktree_isolation,
            "approvalPolicy": config.agents.approval_policy
        }),
        "session/new" => {
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
            };
            state.sessions.insert(session.id.clone(), session.clone());
            json!({ "session": session })
        }
        "session/prompt" => {
            let prompt = params
                .get("prompt")
                .and_then(Value::as_str)
                .unwrap_or_default();
            json!({
                "accepted": true,
                "prompt": prompt,
                "plan": CORE_WORK_GATE_TOOLS,
                "events": [
                    { "type": "plan_update", "message": "grill and contract required before edits" },
                    { "type": "tool", "name": "grill_me" },
                    { "type": "terminal", "stream": "status", "text": "adapter execution pending approval" }
                ]
            })
        }
        "session/cancel" => json!({ "cancelled": true }),
        "shutdown" => json!({ "ok": true }),
        _ => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("unknown method '{method}'")
                }
            }));
        }
    };

    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acp_initialize_mentions_sdk_and_capabilities() {
        let mut state = AcpState::default();
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }),
        )
        .expect("response");
        assert_eq!(
            response["result"]["serverInfo"]["name"],
            "architect-mcp-tui"
        );
        assert!(
            response["result"]["serverInfo"]["sdk"]
                .as_str()
                .unwrap()
                .contains("ProtocolVersion")
        );
        assert_eq!(response["result"]["capabilities"]["terminal"], true);
    }

    #[test]
    fn acp_prompt_returns_golden_work_gate_plan() {
        let mut state = AcpState::default();
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "session/prompt",
                "params": { "prompt": "build an app" }
            }),
        )
        .expect("response");
        assert_eq!(response["result"]["plan"][0], "grill_me");
        assert_eq!(response["result"]["events"][1]["name"], "grill_me");
    }

    #[test]
    fn acp_config_options_expose_adapter_and_approval_policy() {
        let mut state = AcpState::default();
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({ "jsonrpc": "2.0", "id": 4, "method": "config/options" }),
        )
        .expect("response");
        assert_eq!(response["result"]["worktreeIsolation"], true);
        assert_eq!(response["result"]["approvalPolicy"], "manual");
    }
}
