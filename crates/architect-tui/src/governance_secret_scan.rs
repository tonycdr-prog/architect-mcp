use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn governance_config_files(workspace: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let canonical_root = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    collect_config_files(workspace, &canonical_root, workspace, 0, &mut files);
    files
}

fn collect_config_files(
    root: &Path,
    canonical_root: &Path,
    current: &Path,
    depth: usize,
    files: &mut Vec<PathBuf>,
) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if ignored_dir(&name) {
                continue;
            }
            collect_config_files(root, canonical_root, &path, depth + 1, files);
        } else if file_type.is_file()
            && is_sensitive_config_name(&name)
            && path_stays_inside(&path, canonical_root)
        {
            files.push(path);
        }
    }
    files.sort_by(|left, right| {
        let left = left.strip_prefix(root).unwrap_or(left);
        let right = right.strip_prefix(root).unwrap_or(right);
        left.cmp(right)
    });
}

fn path_stays_inside(path: &Path, canonical_root: &Path) -> bool {
    path.canonicalize()
        .is_ok_and(|canonical| canonical.starts_with(canonical_root))
}

fn ignored_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "dist" | ".next" | ".vitepress"
    )
}

fn is_sensitive_config_name(name: &str) -> bool {
    name == ".mcp.json" || (name.starts_with(".env") && name != ".env.example")
}

pub(crate) fn looks_secret_like(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    content.contains("npm_")
        || content.contains("ghp_")
        || content.contains("github_pat_")
        || content.contains("AKIA")
        || content.contains("BEGIN PRIVATE KEY")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
        || lower.lines().any(line_has_secret_assignment)
        || lower
            .split(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\'')
            .any(credentialed_service_url)
}

fn line_has_secret_assignment(line: &str) -> bool {
    let Some((key, value)) = split_assignment(line) else {
        return line_has_inline_secret_assignment(line);
    };
    (secret_key_name(key) && secret_value(value)) || line_has_inline_secret_assignment(line)
}

fn split_assignment(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
        return None;
    }
    if let Some((key, value)) = trimmed.split_once('=') {
        return Some((clean_key(key), clean_value(value)));
    }
    let (key, value) = trimmed.split_once(':')?;
    Some((clean_key(key), clean_value(value)))
}

fn clean_key(value: &str) -> &str {
    value
        .trim()
        .trim_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
}

fn clean_value(value: &str) -> &str {
    value
        .trim()
        .trim_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
}

fn secret_key_name(key: &str) -> bool {
    secret_key_patterns()
        .iter()
        .any(|pattern| key.contains(pattern))
}

fn secret_key_patterns() -> &'static [&'static str] {
    &[
        "api_key",
        "apikey",
        "access_key",
        "secret",
        "session_secret",
        "password",
        "passwd",
        "token",
        "database_url",
        "db_url",
        "private_key",
    ]
}

fn line_has_inline_secret_assignment(line: &str) -> bool {
    secret_key_patterns().iter().any(|pattern| {
        line.match_indices(pattern)
            .any(|(index, _)| inline_secret_value(line, index + pattern.len()))
    })
}

fn inline_secret_value(line: &str, start: usize) -> bool {
    let after_key = &line[start..];
    let Some(separator_index) = after_key.find([':', '=']) else {
        return false;
    };
    if separator_index > 8 {
        return false;
    }
    let value = secret_value_token(&after_key[separator_index + 1..]);
    secret_value(&value)
}

fn secret_value_token(value: &str) -> String {
    let trimmed = value.trim_start();
    if let Some(quote) = trimmed
        .chars()
        .next()
        .filter(|ch| *ch == '"' || *ch == '\'')
    {
        let rest = &trimmed[quote.len_utf8()..];
        let Some(end) = rest.find(quote) else {
            return clean_value(trimmed).to_string();
        };
        return rest[..end].to_string();
    }
    let end = trimmed
        .find(|ch: char| ch.is_whitespace() || matches!(ch, ',' | '}' | ']'))
        .unwrap_or(trimmed.len());
    clean_value(&trimmed[..end]).to_string()
}

fn secret_value(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || placeholder_value(value) {
        return false;
    }
    credentialed_service_url(value) || value.len() >= 16
}

fn placeholder_value(value: &str) -> bool {
    let value = value.trim();
    value.starts_with('<')
        || value.starts_with("${")
        || value.contains("replace_me")
        || value.contains("placeholder")
        || value.contains("changeme")
        || value.contains("example")
        || value.contains("dummy")
        || value.contains("your_")
}

fn credentialed_service_url(value: &str) -> bool {
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };
    matches!(
        scheme,
        "postgres" | "postgresql" | "mysql" | "mongodb" | "redis" | "amqp"
    ) && rest.contains('@')
}
