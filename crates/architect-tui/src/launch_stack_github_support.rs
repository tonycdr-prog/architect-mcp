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
    } else if lower.starts_with("/")
        || lower.contains("/users/")
        || lower.contains("/home/")
        || (lower.len() >= 3 && lower.as_bytes()[1] == b':' && lower.as_bytes()[2] == b'/')
    {
        "[redacted-local-path]".to_string()
    } else {
        token.to_string()
    }
}

fn is_private_key_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("begin") && lower.contains("private") && lower.contains("key")
}
