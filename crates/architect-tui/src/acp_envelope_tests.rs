use serde_json::json;

use crate::acp::{handle_json_rpc_line, handle_json_rpc_value};
use crate::acp_state::AcpState;
use crate::config::TuiConfig;

#[test]
fn acp_rejects_invalid_json_rpc_envelopes_before_dispatch() {
    let mut state = AcpState::default();

    for (request, expected_id, message) in [
        (
            json!(["not-object"]),
            json!(null),
            "ACP JSON-RPC request must be an object",
        ),
        (
            json!({ "id": 80, "method": "initialize" }),
            json!(80),
            "ACP JSON-RPC request must include jsonrpc=\"2.0\"",
        ),
        (
            json!({ "jsonrpc": "1.0", "id": 81, "method": "initialize" }),
            json!(81),
            "ACP JSON-RPC request must include jsonrpc=\"2.0\"",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": 82 }),
            json!(82),
            "ACP JSON-RPC method is required",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": 83, "method": false }),
            json!(83),
            "ACP JSON-RPC method must be a string",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": 84, "method": "  " }),
            json!(84),
            "ACP JSON-RPC method must not be blank",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": { "bad": true }, "method": "initialize" }),
            json!(null),
            "ACP JSON-RPC request id must be a string, number, or null",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": ["bad"], "method": "initialize" }),
            json!(null),
            "ACP JSON-RPC request id must be a string, number, or null",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": false, "method": "initialize" }),
            json!(null),
            "ACP JSON-RPC request id must be a string, number, or null",
        ),
    ] {
        let response = handle_json_rpc_value(&mut state, &TuiConfig::default(), request)
            .expect("error response");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], expected_id);
        assert_eq!(response["error"]["code"], -32600);
        assert_eq!(response["error"]["message"], message);
    }
}

#[test]
fn acp_preserves_notifications_and_valid_request_errors_after_envelope_validation() {
    let mut state = AcpState::default();

    let notification = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    );
    assert!(notification.is_none());

    let unknown_notification = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({ "jsonrpc": "2.0", "method": "not/a-real-method" }),
    );
    assert!(unknown_notification.is_none());

    let unknown_method = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({ "jsonrpc": "2.0", "id": "abc", "method": "not/a-real-method" }),
    )
    .expect("unknown method response");
    assert_eq!(unknown_method["id"], "abc");
    assert_eq!(unknown_method["error"]["code"], -32601);
    assert_eq!(
        unknown_method["error"]["message"],
        "unknown method 'not/a-real-method'"
    );

    let valid_initialize = handle_json_rpc_value(
        &mut state,
        &TuiConfig::default(),
        json!({ "jsonrpc": "2.0", "id": null, "method": "initialize" }),
    )
    .expect("initialize response");
    assert_eq!(valid_initialize["id"], json!(null));
    assert_eq!(
        valid_initialize["result"]["serverInfo"]["name"],
        "architect-mcp-tui"
    );
}

#[test]
fn acp_malformed_json_and_valid_line_request_still_recover() {
    let mut state = AcpState::default();
    let parse_error = handle_json_rpc_line(&mut state, &TuiConfig::default(), "{bad json").unwrap();
    assert_eq!(parse_error["error"]["code"], -32700);
    assert_eq!(parse_error["id"], json!(null));

    let response = handle_json_rpc_line(
        &mut state,
        &TuiConfig::default(),
        r#"{ "jsonrpc": "2.0", "id": 85, "method": "initialize" }"#,
    )
    .unwrap();
    assert_eq!(response["id"], 85);
    assert_eq!(
        response["result"]["serverInfo"]["name"],
        "architect-mcp-tui"
    );
}
