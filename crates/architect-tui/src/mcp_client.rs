use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::thread;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::mcp::ArchitectMcpBridge;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpResponseError {
    pub code: i64,
    pub message: String,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum McpToolOutcome {
    Ok { value: Value },
    Error { error: McpResponseError },
}

pub struct StdioMcpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl StdioMcpClient {
    pub fn connect(bridge: &ArchitectMcpBridge) -> Result<Self> {
        let mut child = bridge.spawn()?;
        let stdin = child.stdin.take().context("architect-mcp stdin missing")?;
        let stdout = child
            .stdout
            .take()
            .context("architect-mcp stdout missing")?;
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || drain_stderr(stderr));
        }

        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        let initialize = client.request(
            "initialize",
            json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "architect-mcp-tui",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )?;
        if let Err(error) = initialize {
            anyhow::bail!("architect-mcp initialize failed: {}", error.message);
        }
        client.notify("notifications/initialized", json!({}))?;
        Ok(client)
    }

    pub fn list_tools(&mut self) -> Result<Value> {
        match self.request("tools/list", json!({}))? {
            Ok(result) => Ok(result),
            Err(error) => anyhow::bail!("tools/list failed: {}", error.message),
        }
    }

    pub fn call_tool(&mut self, name: &str, args: Value) -> Result<McpToolOutcome> {
        match self.request(
            "tools/call",
            json!({
                "name": name,
                "arguments": args
            }),
        )? {
            Ok(result) => Ok(normalize_tool_result(result)),
            Err(error) => Ok(McpToolOutcome::Error { error }),
        }
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<std::result::Result<Value, McpResponseError>> {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        writeln!(self.stdin, "{request}")?;
        self.stdin.flush()?;

        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line)?;
            if read == 0 {
                anyhow::bail!("architect-mcp exited before responding to {method}");
            }
            let response: Value = serde_json::from_str(line.trim())
                .with_context(|| format!("invalid JSON-RPC response for {method}"))?;
            if response.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = response.get("error") {
                return Ok(Err(parse_mcp_error(error)));
            }
            if let Some(result) = response.get("result") {
                return Ok(Ok(result.clone()));
            }
            anyhow::bail!("JSON-RPC response for {method} had no result or error");
        }
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        writeln!(self.stdin, "{notification}")?;
        self.stdin.flush()?;
        Ok(())
    }
}

impl Drop for StdioMcpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn drain_stderr(stderr: std::process::ChildStderr) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        line.clear();
    }
}

fn normalize_tool_result(result: Value) -> McpToolOutcome {
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return McpToolOutcome::Error {
            error: McpResponseError {
                code: -32000,
                message: extract_text_content(&result)
                    .unwrap_or_else(|| "tool returned an MCP error".to_string()),
                details: Some(result),
            },
        };
    }
    if let Some(value) = result.get("structuredContent") {
        return McpToolOutcome::Ok {
            value: value.clone(),
        };
    }
    if let Some(text) = extract_text_content(&result) {
        let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "text": text }));
        return McpToolOutcome::Ok { value };
    }
    McpToolOutcome::Ok { value: result }
}

fn extract_text_content(result: &Value) -> Option<String> {
    result
        .get("content")
        .and_then(Value::as_array)
        .and_then(|content| {
            content.iter().find_map(|item| {
                if item.get("type").and_then(Value::as_str) == Some("text") {
                    item.get("text")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                } else {
                    None
                }
            })
        })
}

fn parse_mcp_error(error: &Value) -> McpResponseError {
    McpResponseError {
        code: error.get("code").and_then(Value::as_i64).unwrap_or(-32000),
        message: error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("MCP error")
            .to_string(),
        details: error.get("data").cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::TuiConfig, mcp::ArchitectMcpBridge};
    use std::process::Command;

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
    fn fake_client(temp: &tempfile::TempDir) -> StdioMcpClient {
        let server_path = temp.path().join("fake-mcp.mjs");
        std::fs::write(&server_path, fake_mcp_server_script()).expect("write fake server");
        let mut config = TuiConfig::default();
        config.architect_mcp.command = Some("node".to_string());
        config.architect_mcp.args = vec![server_path.display().to_string()];
        StdioMcpClient::connect(&ArchitectMcpBridge::new(temp.path(), config)).expect("connect")
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
  else if (msg.method === 'tools/call') console.log(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result: { content: [{ type: 'text', text: JSON.stringify({ ok: true, name: msg.params.name }) }], structuredContent: { ok: true, name: msg.params.name } } }));
});
"#
    }
}
