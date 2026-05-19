use std::path::Path;
use std::process::Command;

use serde_json::Value;

pub(crate) fn public_text(value: &str, max_len: usize) -> String {
    let normalized = value
        .replace('\\', "/")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if is_private_key_material(&normalized) {
        return "[redacted-secret]".to_string();
    }
    let mut text = normalized
        .split_whitespace()
        .map(redact_token)
        .collect::<Vec<_>>()
        .join(" ");
    if text.chars().count() > max_len {
        text = text.chars().take(max_len).collect::<String>();
        text.push_str(" [truncated]");
    }
    text
}

pub(crate) fn run_gh_json(workspace: &Path, args: &[String]) -> Result<Value, String> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(workspace)
        .output()
        .map_err(|error| {
            format!(
                "gh command could not run: {}",
                public_text(&error.to_string(), 240)
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "gh command failed: {}",
            public_text(stderr.trim(), 240)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("gh JSON could not be parsed: {error}"))
}

fn redact_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    if lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
        || is_private_key_material(token)
    {
        "[redacted-secret]".to_string()
    } else if contains_local_path(&lower) {
        "[redacted-local-path]".to_string()
    } else {
        token.to_string()
    }
}

fn contains_local_path(value: &str) -> bool {
    if value.starts_with('/') {
        return true;
    }

    let bytes = value.as_bytes();
    for index in 0..bytes.len() {
        if is_embedded_unix_path(bytes, index) || is_embedded_windows_path(bytes, index) {
            return true;
        }
    }
    false
}

fn is_embedded_unix_path(bytes: &[u8], index: usize) -> bool {
    if bytes[index] != b'/' {
        return false;
    }
    if index + 1 < bytes.len() && bytes[index + 1] == b'/' {
        return false;
    }
    path_prefix_boundary(bytes, index) && slash_starts_path(bytes, index)
}

fn is_embedded_windows_path(bytes: &[u8], index: usize) -> bool {
    if index + 2 >= bytes.len()
        || !bytes[index].is_ascii_alphabetic()
        || bytes[index + 1] != b':'
        || bytes[index + 2] != b'/'
    {
        return false;
    }
    if index + 3 < bytes.len() && bytes[index + 3] == b'/' {
        return false;
    }
    path_prefix_boundary(bytes, index)
}

fn path_prefix_boundary(bytes: &[u8], index: usize) -> bool {
    index == 0
        || matches!(
            bytes[index - 1],
            b'=' | b':' | b'(' | b'[' | b'{' | b'<' | b'"' | b'\'' | b'`' | b',' | b';'
        )
}

fn slash_starts_path(bytes: &[u8], index: usize) -> bool {
    index + 1 == bytes.len()
        || matches!(
            bytes[index + 1],
            b'.' | b'_' | b'-' | b'~' | b'a'..=b'z' | b'0'..=b'9'
        )
}

fn is_private_key_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("begin") && lower.contains("private") && lower.contains("key")
}
