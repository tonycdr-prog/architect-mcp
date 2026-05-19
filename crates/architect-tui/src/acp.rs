use anyhow::Result;
use serde_json::{Map, Value, json};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::acp_state::{ACP_MAX_CONCURRENCY, ACP_MIN_CONCURRENCY, ACP_MODES, AcpState};
use crate::config::TuiConfig;
use crate::mcp::CORE_WORK_GATE_TOOLS;

pub use crate::acp_state::{AcpSession, AcpSessionStatus};

const SESSION_PROMPT_PARAMS: [&str; 2] = ["sessionId", "prompt"];
const SESSION_ID_PARAMS: [&str; 1] = ["sessionId"];

struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    params: Value,
}

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
    let request = match validate_json_rpc_request(request) {
        Ok(request) => request,
        Err((id, message)) => return Some(error_response(id, -32600, &message)),
    };
    let id = request.id.clone()?;
    let result = match request.method.as_str() {
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
            let session = match state.create_session(config, &request.params) {
                Ok(session) => session,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            json!({ "session": session })
        }
        "session/prompt" => {
            let params =
                match validate_params("session/prompt", &request.params, &SESSION_PROMPT_PARAMS) {
                    Ok(params) => params,
                    Err(error) => return Some(error_response(id, -32602, &error.to_string())),
                };
            let session_id = match required_string_param("session/prompt", params, "sessionId") {
                Ok(session_id) => session_id,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            let prompt = match required_string_param("session/prompt", params, "prompt") {
                Ok(prompt) => prompt,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            if prompt.trim().is_empty() {
                return Some(error_response(
                    id,
                    -32602,
                    "ACP parameter 'prompt' must not be blank for method 'session/prompt'",
                ));
            }
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
            let params = match validate_params("session/get", &request.params, &SESSION_ID_PARAMS) {
                Ok(params) => params,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            let session_id = match required_string_param("session/get", params, "sessionId") {
                Ok(session_id) => session_id,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            let session = match state.session(session_id) {
                Ok(session) => session,
                Err(error) => return Some(error_response(id, -32004, &error.to_string())),
            };
            json!({ "session": session })
        }
        "session/events" => {
            let params =
                match validate_params("session/events", &request.params, &SESSION_ID_PARAMS) {
                    Ok(params) => params,
                    Err(error) => return Some(error_response(id, -32602, &error.to_string())),
                };
            let session_id = match required_string_param("session/events", params, "sessionId") {
                Ok(session_id) => session_id,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
            };
            let session = match state.session(session_id) {
                Ok(session) => session,
                Err(error) => return Some(error_response(id, -32004, &error.to_string())),
            };
            json!({ "sessionId": session_id, "events": session.events })
        }
        "session/cancel" => {
            let params =
                match validate_params("session/cancel", &request.params, &SESSION_ID_PARAMS) {
                    Ok(params) => params,
                    Err(error) => return Some(error_response(id, -32602, &error.to_string())),
                };
            let session_id = match required_string_param("session/cancel", params, "sessionId") {
                Ok(session_id) => session_id,
                Err(error) => return Some(error_response(id, -32602, &error.to_string())),
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
                &format!("unknown method '{}'", request.method),
            ));
        }
    };

    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}

fn validate_json_rpc_request(
    request: Value,
) -> std::result::Result<JsonRpcRequest, (Value, String)> {
    let Some(object) = request.as_object() else {
        return Err((
            Value::Null,
            "ACP JSON-RPC request must be an object".to_string(),
        ));
    };

    let id = match object.get("id") {
        Some(id) if is_valid_json_rpc_id(id) => Some(id.clone()),
        Some(_) => {
            return Err((
                Value::Null,
                "ACP JSON-RPC request id must be a string, number, or null".to_string(),
            ));
        }
        None => None,
    };

    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err((
            id.unwrap_or(Value::Null),
            "ACP JSON-RPC request must include jsonrpc=\"2.0\"".to_string(),
        ));
    }

    let method = match object.get("method") {
        Some(Value::String(method)) if !method.trim().is_empty() => method.clone(),
        Some(Value::String(_)) => {
            return Err((
                id.unwrap_or(Value::Null),
                "ACP JSON-RPC method must not be blank".to_string(),
            ));
        }
        Some(_) => {
            return Err((
                id.unwrap_or(Value::Null),
                "ACP JSON-RPC method must be a string".to_string(),
            ));
        }
        None => {
            return Err((
                id.unwrap_or(Value::Null),
                "ACP JSON-RPC method is required".to_string(),
            ));
        }
    };

    Ok(JsonRpcRequest {
        id,
        method,
        params: object.get("params").cloned().unwrap_or_else(|| json!({})),
    })
}

fn is_valid_json_rpc_id(id: &Value) -> bool {
    id.is_string() || id.is_number() || id.is_null()
}

fn validate_params<'a>(
    method: &str,
    params: &'a Value,
    allowed: &[&str],
) -> Result<&'a Map<String, Value>> {
    let Some(object) = params.as_object() else {
        anyhow::bail!("ACP method '{method}' params must be an object");
    };
    for name in object.keys() {
        if !allowed.contains(&name.as_str()) {
            anyhow::bail!("unsupported ACP parameter '{name}' for method '{method}'");
        }
    }
    Ok(object)
}

fn required_string_param<'a>(
    method: &str,
    params: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str> {
    let Some(value) = params.get(name) else {
        anyhow::bail!("ACP parameter '{name}' is required for method '{method}'");
    };
    let Some(value) = value.as_str() else {
        anyhow::bail!("ACP parameter '{name}' must be a string for method '{method}'");
    };
    Ok(value)
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
