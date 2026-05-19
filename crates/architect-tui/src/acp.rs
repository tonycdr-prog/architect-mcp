use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::acp_state::{ACP_MAX_CONCURRENCY, ACP_MIN_CONCURRENCY, ACP_MODES, AcpState};
use crate::config::TuiConfig;
use crate::mcp::CORE_WORK_GATE_TOOLS;

pub use crate::acp_state::{AcpSession, AcpSessionStatus};

pub fn acp_sdk_marker() -> &'static str {
    std::any::type_name::<agent_client_protocol::schema::ProtocolVersion>()
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
        if let Some(response) = handle_json_rpc_line(&mut state, &config, &line) {
            stdout
                .write_all(serde_json::to_string(&response)?.as_bytes())
                .await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

pub fn handle_json_rpc_line(state: &mut AcpState, config: &TuiConfig, line: &str) -> Option<Value> {
    match serde_json::from_str::<Value>(line) {
        Ok(request) => handle_json_rpc_value(state, config, request),
        Err(error) => Some(error_response(
            Value::Null,
            -32700,
            &format!("parse error: {error}"),
        )),
    }
}

pub fn handle_json_rpc_value(
    state: &mut AcpState,
    config: &TuiConfig,
    request: Value,
) -> Option<Value> {
    let id = request.get("id").cloned()?;
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
            "modes": ACP_MODES,
            "concurrency": {
                "min": ACP_MIN_CONCURRENCY,
                "max": ACP_MAX_CONCURRENCY,
                "default": ACP_MIN_CONCURRENCY
            },
            "worktreeIsolation": config.agents.worktree_isolation,
            "approvalPolicy": config.agents.approval_policy
        }),
        "session/new" => {
            let session = match state.create_session(config, &params) {
                Ok(session) => session,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            json!({ "session": session })
        }
        "session/prompt" => {
            let Some(session_id) = params.get("sessionId").and_then(Value::as_str) else {
                return Some(error_response(id, -32602, "sessionId is required"));
            };
            let prompt = params
                .get("prompt")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let events = match state.record_prompt_turn(session_id, prompt) {
                Ok(events) => events,
                Err(error) => return Some(error_response(id, -32004, &error.to_string())),
            };
            json!({
                "accepted": true,
                "sessionId": session_id,
                "prompt": prompt,
                "plan": CORE_WORK_GATE_TOOLS,
                "events": events
            })
        }
        "session/get" => {
            let Some(session_id) = params.get("sessionId").and_then(Value::as_str) else {
                return Some(error_response(id, -32602, "sessionId is required"));
            };
            let session = match state.session(session_id) {
                Ok(session) => session,
                Err(error) => return Some(error_response(id, -32004, &error.to_string())),
            };
            json!({ "session": session })
        }
        "session/events" => {
            let Some(session_id) = params.get("sessionId").and_then(Value::as_str) else {
                return Some(error_response(id, -32602, "sessionId is required"));
            };
            let session = match state.session(session_id) {
                Ok(session) => session,
                Err(error) => return Some(error_response(id, -32004, &error.to_string())),
            };
            json!({ "sessionId": session_id, "events": session.events })
        }
        "session/cancel" => {
            let Some(session_id) = params.get("sessionId").and_then(Value::as_str) else {
                return Some(error_response(id, -32602, "sessionId is required"));
            };
            let event = match state.cancel(session_id) {
                Ok(event) => event,
                Err(error) => return Some(error_response(id, -32004, &error.to_string())),
            };
            json!({ "cancelled": true, "sessionId": session_id, "event": event })
        }
        "shutdown" => json!({ "ok": true }),
        _ => {
            return Some(error_response(
                id,
                -32601,
                &format!("unknown method '{method}'"),
            ));
        }
    };

    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}

fn error_response(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}
