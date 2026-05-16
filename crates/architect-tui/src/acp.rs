use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::acp_state::AcpState;
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
            "modes": ["new-app", "coding-task", "automation", "arena"],
            "concurrency": { "min": 1, "max": 8, "default": 1 },
            "worktreeIsolation": config.agents.worktree_isolation,
            "approvalPolicy": config.agents.approval_policy
        }),
        "session/new" => {
            let session = state.create_session(config, &params);
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
        let session = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "session/new" }),
        )
        .expect("session response")["result"]["session"]["id"]
            .as_str()
            .unwrap()
            .to_string();
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "session/prompt",
                "params": { "sessionId": session, "prompt": "build an app" }
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

    #[test]
    fn acp_rejects_unknown_session_and_tracks_cancellation() {
        let mut state = AcpState::default();
        let missing = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "session/prompt",
                "params": { "sessionId": "missing", "prompt": "build" }
            }),
        )
        .expect("error response");
        assert_eq!(missing["error"]["code"], -32004);

        let session = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({ "jsonrpc": "2.0", "id": 6, "method": "session/new" }),
        )
        .expect("session response")["result"]["session"]["id"]
            .as_str()
            .unwrap()
            .to_string();
        let cancel = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "session/cancel",
                "params": { "sessionId": session }
            }),
        )
        .expect("cancel response");
        assert_eq!(cancel["result"]["cancelled"], true);
        assert_eq!(cancel["result"]["event"]["type"], "cancelled");
    }

    #[test]
    fn acp_notifications_do_not_emit_responses() {
        let mut state = AcpState::default();
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        );
        assert!(response.is_none());
    }

    #[test]
    fn acp_malformed_json_returns_parse_error_without_poisoning_state() {
        let mut state = AcpState::default();
        let parse_error =
            handle_json_rpc_line(&mut state, &TuiConfig::default(), "{bad json").unwrap();
        assert_eq!(parse_error["error"]["code"], -32700);
        assert_eq!(parse_error["id"], Value::Null);

        let response = handle_json_rpc_line(
            &mut state,
            &TuiConfig::default(),
            r#"{ "jsonrpc": "2.0", "id": 8, "method": "initialize" }"#,
        )
        .unwrap();
        assert_eq!(
            response["result"]["serverInfo"]["name"],
            "architect-mcp-tui"
        );
    }
}
