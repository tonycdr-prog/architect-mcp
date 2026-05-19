use anyhow::Result;

pub(crate) fn validate_issue_url(issue_url: &str) -> Result<String> {
    let trimmed = issue_url.trim().trim_end_matches('/');
    if trimmed.is_empty()
        || trimmed.contains(char::is_whitespace)
        || trimmed.contains('?')
        || trimmed.contains('#')
    {
        anyhow::bail!("--issue-url must be a plain GitHub issue URL");
    }
    let Some(path) = trimmed.strip_prefix("https://github.com/") else {
        anyhow::bail!("--issue-url must be a plain GitHub issue URL");
    };
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() != 4 || parts[2] != "issues" {
        anyhow::bail!("--issue-url must be a plain GitHub issue URL");
    }
    if !is_safe_github_path_part(parts[0])
        || !is_safe_github_path_part(parts[1])
        || parts[3].is_empty()
        || !parts[3].chars().all(|ch| ch.is_ascii_digit())
    {
        anyhow::bail!("--issue-url must be a plain GitHub issue URL");
    }
    Ok(trimmed.to_string())
}

fn is_safe_github_path_part(part: &str) -> bool {
    !part.is_empty()
        && part
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}
