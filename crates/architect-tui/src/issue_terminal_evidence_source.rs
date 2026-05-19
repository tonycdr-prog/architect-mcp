use std::path::Path;
use std::process::Command;

use serde_json::Value;

#[derive(Debug)]
pub(crate) struct IssueContent {
    number: u64,
    title: String,
    url: String,
    bodies: Vec<BodySource>,
}

#[derive(Debug)]
pub(crate) struct BodySource {
    pub(crate) source: String,
    pub(crate) body: String,
}

impl IssueContent {
    pub(crate) fn number(&self) -> u64 {
        self.number
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn url(&self) -> &str {
        &self.url
    }

    pub(crate) fn bodies(&self) -> &[BodySource] {
        &self.bodies
    }
}

pub(crate) fn extract_json_blocks(body: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let trimmed = body.trim();
    if is_terminal_evidence_json(trimmed) {
        blocks.push(trimmed.to_string());
    }

    let mut in_json = false;
    let mut current = Vec::new();
    for line in body.lines() {
        let marker = line.trim();
        if marker.starts_with("```") {
            if in_json {
                blocks.push(current.join("\n"));
                current.clear();
                in_json = false;
            } else {
                let lang = marker.trim_start_matches('`').trim();
                in_json = lang.eq_ignore_ascii_case("json");
            }
            continue;
        }
        if in_json {
            current.push(line);
        }
    }

    blocks
}

fn is_terminal_evidence_json(text: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    value.get("schemaVersion").is_some() && value.get("reports").is_some()
}

pub(crate) fn fetch_issue_content(
    workspace: &Path,
    repo: Option<&str>,
    issue: u64,
) -> Result<IssueContent, String> {
    let mut args = issue_view_args(issue);
    if let Some(repo) = repo.filter(|repo| !repo.trim().is_empty()) {
        args.push("--repo".to_string());
        args.push(repo.to_string());
    }
    let value =
        run_gh_json(workspace, &args).map_err(|error| format!("issue #{issue}: {error}"))?;
    issue_content_from_value(issue, &value)
}

pub(crate) fn issue_content_from_value(
    fallback_issue: u64,
    value: &Value,
) -> Result<IssueContent, String> {
    let number = value
        .get("number")
        .and_then(Value::as_u64)
        .unwrap_or(fallback_issue);
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut bodies = Vec::new();
    if let Some(body) = value.get("body").and_then(Value::as_str)
        && !body.trim().is_empty()
    {
        bodies.push(BodySource {
            source: "issue body".to_string(),
            body: body.to_string(),
        });
    }
    if let Some(comments) = value.get("comments").and_then(Value::as_array) {
        for (index, comment) in comments.iter().enumerate() {
            if let Some(body) = comment.get("body").and_then(Value::as_str)
                && !body.trim().is_empty()
            {
                bodies.push(BodySource {
                    source: format!("comment {}", index + 1),
                    body: body.to_string(),
                });
            }
        }
    }
    Ok(IssueContent {
        number,
        title,
        url,
        bodies,
    })
}

pub(crate) fn issue_view_args(issue: u64) -> Vec<String> {
    vec![
        "issue".to_string(),
        "view".to_string(),
        issue.to_string(),
        "--json".to_string(),
        "number,title,url,body,comments".to_string(),
    ]
}

pub(crate) fn issue_terminal_evidence_source_path(
    issue_number: u64,
    has_issue_body_blocks: bool,
    has_comment_blocks: bool,
) -> Option<String> {
    match (has_issue_body_blocks, has_comment_blocks) {
        (false, false) => None,
        (true, false) => Some(format!("issue #{issue_number} body")),
        (false, true) => Some(format!("issue #{issue_number} comments")),
        (true, true) => Some(format!("issue #{issue_number} body + comments")),
    }
}

fn run_gh_json(workspace: &Path, args: &[String]) -> Result<Value, String> {
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

pub(crate) fn public_text(value: &str, max_len: usize) -> String {
    let normalized = value
        .replace('\\', "/")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
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

fn redact_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    if contains_secret_token(&lower) {
        "[redacted-secret]".to_string()
    } else if contains_local_path(&lower) {
        "[redacted-local-path]".to_string()
    } else {
        token.to_string()
    }
}

fn contains_secret_token(value: &str) -> bool {
    if value.contains("begin") && value.contains("private") && value.contains("key") {
        return true;
    }
    value
        .split(secret_segment_boundary)
        .any(is_secret_like_segment)
}

fn secret_segment_boundary(ch: char) -> bool {
    matches!(
        ch,
        '=' | ':' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '`' | ',' | ';'
    )
}

fn is_secret_like_segment(segment: &str) -> bool {
    let segment = segment.trim_matches(|ch: char| matches!(ch, '.' | '!' | '?'));
    has_credential_tail(segment, "github_pat_", 16)
        || has_credential_tail(segment, "npm_", 16)
        || has_credential_tail(segment, "ghp_", 16)
        || has_credential_tail(segment, "sk-", 16)
        || has_credential_tail(segment, "xoxb-", 16)
}

fn has_credential_tail(segment: &str, prefix: &str, min_tail_len: usize) -> bool {
    let Some(tail) = segment.strip_prefix(prefix) else {
        return false;
    };
    tail.chars().count() >= min_tail_len
        && tail
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
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
