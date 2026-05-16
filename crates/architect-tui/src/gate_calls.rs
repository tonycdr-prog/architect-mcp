use anyhow::Result;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::headless_events::{JsonlEvent, emit};
use crate::mcp::{McpToolOutcome, StdioMcpClient};

pub(crate) async fn call_gate<W: AsyncWriteExt + Unpin>(
    client: &mut StdioMcpClient,
    output: &mut W,
    jsonl: bool,
    name: &str,
    purpose: &str,
    args: Value,
) -> Result<Option<Value>> {
    emit(output, jsonl, &JsonlEvent::McpCall { name, purpose }).await?;
    match client.call_tool(name, args)? {
        McpToolOutcome::Ok { value } => {
            emit(
                output,
                jsonl,
                &JsonlEvent::McpResult {
                    name,
                    result: &value,
                },
            )
            .await?;
            Ok(Some(value))
        }
        McpToolOutcome::Error { error } => {
            emit(
                output,
                jsonl,
                &JsonlEvent::McpError {
                    name,
                    error: &error,
                },
            )
            .await?;
            Ok(None)
        }
    }
}
