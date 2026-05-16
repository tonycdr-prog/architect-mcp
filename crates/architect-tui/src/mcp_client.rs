use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

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
    stdout_rx: Receiver<std::result::Result<Value, String>>,
    next_id: u64,
    request_timeout: Duration,
}

impl StdioMcpClient {
    pub fn connect(bridge: &ArchitectMcpBridge) -> Result<Self> {
        Self::connect_with_timeout(bridge, Duration::from_secs(30))
    }

    pub fn connect_with_timeout(
        bridge: &ArchitectMcpBridge,
        request_timeout: Duration,
    ) -> Result<Self> {
        let mut child = bridge.spawn()?;
        let stdin = child.stdin.take().context("architect-mcp stdin missing")?;
        let stdout = child
            .stdout
            .take()
            .context("architect-mcp stdout missing")?;
        let stdout_rx = spawn_stdout_reader(stdout);
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || drain_stderr(stderr));
        }

        let mut client = Self {
            child,
            stdin,
            stdout_rx,
            next_id: 1,
            request_timeout,
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
            let response = self.recv_response(method)?;
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

    fn recv_response(&self, method: &str) -> Result<Value> {
        match self.stdout_rx.recv_timeout(self.request_timeout) {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => anyhow::bail!("invalid JSON-RPC response for {method}: {error}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                anyhow::bail!("timed out waiting for architect-mcp response to {method}")
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("architect-mcp exited before responding to {method}")
            }
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

fn spawn_stdout_reader(stdout: ChildStdout) -> Receiver<std::result::Result<Value, String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let parsed =
                        serde_json::from_str(line.trim()).map_err(|error| error.to_string());
                    if tx.send(parsed).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.send(Err(error.to_string()));
                    break;
                }
            }
        }
    });
    rx
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
#[path = "mcp_client_tests.rs"]
mod tests;
