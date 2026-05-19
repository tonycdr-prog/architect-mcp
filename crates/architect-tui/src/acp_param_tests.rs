use serde_json::json;

use crate::acp::handle_json_rpc_value;
use crate::acp_state::AcpState;
use crate::config::TuiConfig;

#[test]
fn acp_session_methods_reject_invalid_params_before_state_lookup() {
    let mut state = AcpState::default();
    let session = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({ "jsonrpc": "2.0", "id": 60, "method": "session/new" }),
    )
    .expect("session response")["result"]["session"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    for (id, method, params, message) in [
        (
            61,
            "session/prompt",
            json!(["not-object"]),
            "ACP method 'session/prompt' params must be an object",
        ),
        (
            62,
            "session/prompt",
            json!({ "sessionId": session.clone(), "prompt": "build", "workspace": "/tmp/example" }),
            "unsupported ACP parameter 'workspace' for method 'session/prompt'",
        ),
        (
            63,
            "session/prompt",
            json!({ "prompt": "build" }),
            "ACP parameter 'sessionId' is required for method 'session/prompt'",
        ),
        (
            64,
            "session/prompt",
            json!({ "sessionId": 7, "prompt": "build" }),
            "ACP parameter 'sessionId' must be a string for method 'session/prompt'",
        ),
        (
            65,
            "session/prompt",
            json!({ "sessionId": session.clone() }),
            "ACP parameter 'prompt' is required for method 'session/prompt'",
        ),
        (
            66,
            "session/prompt",
            json!({ "sessionId": session.clone(), "prompt": false }),
            "ACP parameter 'prompt' must be a string for method 'session/prompt'",
        ),
        (
            67,
            "session/prompt",
            json!({ "sessionId": session.clone(), "prompt": "  " }),
            "ACP parameter 'prompt' must not be blank for method 'session/prompt'",
        ),
        (
            68,
            "session/get",
            json!("not-object"),
            "ACP method 'session/get' params must be an object",
        ),
        (
            69,
            "session/get",
            json!({ "sessionId": session.clone(), "includeEvents": true }),
            "unsupported ACP parameter 'includeEvents' for method 'session/get'",
        ),
        (
            70,
            "session/events",
            json!({ "sessionId": false }),
            "ACP parameter 'sessionId' must be a string for method 'session/events'",
        ),
        (
            71,
            "session/cancel",
            json!({}),
            "ACP parameter 'sessionId' is required for method 'session/cancel'",
        ),
    ] {
        let response = handle_json_rpc_value(
            &mut state,
            &TuiConfig::default(),
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": params
            }),
        )
        .expect("error response");
        assert_eq!(response["error"]["code"], -32602);
        assert_eq!(response["error"]["message"], message);
    }
}

#[test]
fn acp_session_get_and_events_accept_strict_valid_params() {
    let mut state = AcpState::default();
    let session = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({ "jsonrpc": "2.0", "id": 72, "method": "session/new" }),
    )
    .expect("session response")["result"]["session"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_response = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({
            "jsonrpc": "2.0",
            "id": 73,
            "method": "session/get",
            "params": { "sessionId": session.clone() }
        }),
    )
    .expect("get response");
    assert_eq!(get_response["result"]["session"]["id"], session);

    let events_response = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({
            "jsonrpc": "2.0",
            "id": 74,
            "method": "session/events",
            "params": { "sessionId": session.clone() }
        }),
    )
    .expect("events response");
    assert_eq!(events_response["result"]["sessionId"], session);
    assert_eq!(
        events_response["result"]["events"][0]["type"],
        "session_created"
    );
}
