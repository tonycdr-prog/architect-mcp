pub(crate) fn sanitize_note(note: &str) -> String {
    let first_line = note.lines().next().unwrap_or_default().replace('\\', "/");
    first_line
        .split_whitespace()
        .map(|part| {
            if looks_like_local_path(part) {
                "[redacted-path]"
            } else if looks_like_secret(part) {
                "[redacted-secret]"
            } else if looks_like_repo_reference(part) {
                "[redacted-repo]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}

fn looks_like_local_path(part: &str) -> bool {
    let lower = public_token(part);
    lower.starts_with("/")
        || lower.contains("/users/")
        || lower.contains("/home/")
        || (lower.len() >= 3 && lower.as_bytes()[1] == b':' && lower.as_bytes()[2] == b'/')
}

fn looks_like_secret(part: &str) -> bool {
    let lower = public_token(part);
    lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("gho_")
        || lower.contains("ghu_")
        || lower.contains("ghs_")
        || lower.contains("ghr_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
}

fn looks_like_repo_reference(part: &str) -> bool {
    let lower = public_token(part);
    if lower.starts_with("git@") || lower.starts_with("ssh://git@") {
        return true;
    }
    if !(lower.contains("github.com/")
        || lower.contains("gitlab.com/")
        || lower.contains("bitbucket.org/"))
    {
        return false;
    }
    !(lower.contains("/issues/") || lower.contains("/pull/") || lower.contains("/pulls/"))
}

fn public_token(part: &str) -> String {
    part.trim_matches(|ch: char| {
        matches!(
            ch,
            ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
        )
    })
    .to_ascii_lowercase()
}
