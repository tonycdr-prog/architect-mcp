use super::*;
use crate::{config::TuiConfig, mcp::ArchitectMcpBridge};
use std::process::Command;
use std::time::Duration;

#[test]
fn normalizes_mcp_text_content_as_json() {
    let outcome = normalize_tool_result(json!({
        "content": [
            { "type": "text", "text": "{\"ready\":true}" }
        ]
    }));
    assert_eq!(
        outcome,
        McpToolOutcome::Ok {
            value: json!({ "ready": true })
        }
    );
}

#[cfg(unix)]
#[test]
fn stdio_mcp_client_calls_tool_and_normalizes_structured_content() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let mut client = fake_client(&temp);

    let tools = client.list_tools().expect("list tools");
    assert_eq!(tools["tools"][0]["name"], "grill_me");
    let result = client
        .call_tool("grill_me", json!({ "brief": { "idea": "build a thing" } }))
        .expect("call");
    assert_eq!(
        result,
        McpToolOutcome::Ok {
            value: json!({ "ok": true, "name": "grill_me" })
        }
    );
}

#[cfg(unix)]
#[test]
fn stdio_mcp_client_returns_structured_tool_errors() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let mut client = fake_client(&temp);

    let result = client.call_tool("boom", json!({})).expect("call");
    assert_eq!(
        result,
        McpToolOutcome::Error {
            error: McpResponseError {
                code: -32602,
                message: "bad tool".to_string(),
                details: None
            }
        }
    );
}

#[cfg(unix)]
#[test]
fn stdio_mcp_client_times_out_stalled_tool_calls() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let mut client = fake_client_with_timeout(&temp, Duration::from_millis(50));

    let error = client.call_tool("stall", json!({})).expect_err("timeout");
    assert!(error.to_string().contains("timed out waiting"));
}

#[cfg(unix)]
fn fake_client(temp: &tempfile::TempDir) -> StdioMcpClient {
    fake_client_with_timeout(temp, Duration::from_secs(30))
}

#[cfg(unix)]
fn fake_client_with_timeout(temp: &tempfile::TempDir, timeout: Duration) -> StdioMcpClient {
    let server_path = temp.path().join("fake-mcp.mjs");
    std::fs::write(&server_path, fake_mcp_server_script()).expect("write fake server");
    let mut config = TuiConfig::default();
    config.architect_mcp.command = Some("node".to_string());
    config.architect_mcp.args = vec![server_path.display().to_string()];
    StdioMcpClient::connect_with_timeout(&ArchitectMcpBridge::new(temp.path(), config), timeout)
        .expect("connect")
}

#[cfg(unix)]
fn fake_mcp_server_script() -> &'static str {
    r#"
import readline from 'node:readline';
const rl = readline.createInterface({ input: process.stdin });
rl.on('line', (line) => {
  const msg = JSON.parse(line);
  if (!msg.id) return;
  if (msg.method === 'initialize') console.log(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result: { protocolVersion: '2025-11-25', capabilities: {}, serverInfo: { name: 'fake', version: '0.0.0' } } }));
  else if (msg.method === 'tools/list') console.log(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result: { tools: [{ name: 'grill_me' }] } }));
  else if (msg.method === 'tools/call' && msg.params.name === 'boom') console.log(JSON.stringify({ jsonrpc: '2.0', id: msg.id, error: { code: -32602, message: 'bad tool' } }));
  else if (msg.method === 'tools/call' && msg.params.name === 'stall') {}
  else if (msg.method === 'tools/call') console.log(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result: { content: [{ type: 'text', text: JSON.stringify({ ok: true, name: msg.params.name }) }], structuredContent: { ok: true, name: msg.params.name } } }));
});
"#
}
