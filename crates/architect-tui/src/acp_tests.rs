use serde_json::{Value, json};

use crate::acp::{handle_json_rpc_line, handle_json_rpc_value};
use crate::acp_state::AcpState;
use crate::config::TuiConfig;

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
    assert_eq!(response["result"]["concurrency"]["min"], 1);
    assert_eq!(response["result"]["concurrency"]["max"], 8);
    assert_eq!(response["result"]["modes"][0], "new-app");
}

#[test]
fn acp_session_new_enforces_advertised_config_bounds() {
    let mut state = AcpState::default();
    let config = TuiConfig::default();

    let valid = handle_json_rpc_value(
        &mut state,
        &config,
        json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": "session/new",
            "params": {
                "adapter": "codex",
                "mode": "automation",
                "concurrency": 2,
                "worktreeIsolation": true,
                "approvalPolicy": "manual"
            }
        }),
    )
    .expect("valid response");
    assert_eq!(valid["result"]["session"]["adapter"], "codex");
    assert_eq!(valid["result"]["session"]["mode"], "automation");
    assert_eq!(valid["result"]["session"]["concurrency"], 2);

    for (id, params, message) in [
        (
            42,
            json!({ "adapter": "ghost" }),
            "unknown ACP adapter 'ghost'",
        ),
        (
            43,
            json!({ "mode": "unsafe-mode" }),
            "unsupported ACP mode 'unsafe-mode'",
        ),
        (
            44,
            json!({ "concurrency": 0 }),
            "ACP concurrency must be between 1 and 8",
        ),
        (
            45,
            json!({ "concurrency": 9 }),
            "ACP concurrency must be between 1 and 8",
        ),
        (
            46,
            json!({ "worktreeIsolation": false }),
            "cannot override configured worktreeIsolation=true",
        ),
        (
            47,
            json!({ "approvalPolicy": "auto" }),
            "cannot override configured approvalPolicy=manual",
        ),
    ] {
        let response = handle_json_rpc_value(
            &mut state,
            &config,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "session/new",
                "params": params
            }),
        )
        .expect("error response");
        assert_eq!(response["error"]["code"], -32602);
        assert!(
            response["error"]["message"]
                .as_str()
                .unwrap()
                .contains(message)
        );
    }
}

#[test]
fn acp_session_new_rejects_invalid_param_types() {
    let mut state = AcpState::default();
    for (id, params, message) in [
        (50, json!(["codex"]), "ACP session params must be an object"),
        (56, json!("codex"), "ACP session params must be an object"),
        (
            51,
            json!({ "adapter": ["codex"] }),
            "ACP parameter 'adapter' must be a string",
        ),
        (
            52,
            json!({ "mode": false }),
            "ACP parameter 'mode' must be a string",
        ),
        (
            53,
            json!({ "concurrency": "2" }),
            "ACP parameter 'concurrency' must be a positive integer",
        ),
        (
            54,
            json!({ "worktreeIsolation": "true" }),
            "ACP parameter 'worktreeIsolation' must be a boolean",
        ),
        (
            55,
            json!({ "approvalPolicy": true }),
            "ACP parameter 'approvalPolicy' must be a string",
        ),
        (
            57,
            json!({ "approval_policy": "auto" }),
            "unsupported ACP session parameter 'approval_policy'",
        ),
        (
            58,
            json!({ "workspace": "/tmp/example" }),
            "unsupported ACP session parameter 'workspace'",
        ),
    ] {
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "session/new",
                "params": params
            }),
        )
        .expect("error response");
        assert_eq!(response["error"]["code"], -32602);
        assert_eq!(response["error"]["message"], message);
    }
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
    let parse_error = handle_json_rpc_line(&mut state, &TuiConfig::default(), "{bad json").unwrap();
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
