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
    if trimmed.starts_with('{') && trimmed.contains("\"reports\"") {
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

pub(crate) fn fetch_issue_content(
    workspace: &Path,
    repo: Option<&str>,
    issue: u64,
) -> Result<IssueContent, String> {
    let mut args = vec![
        "issue".to_string(),
        "view".to_string(),
        issue.to_string(),
        "--json".to_string(),
        "number,title,url,body,comments".to_string(),
    ];
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
    if lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
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
