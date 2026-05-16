use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::interactive::InteractiveWorkflowEngine;
use crate::interactive_update::{WorkflowUpdate, inspector_for, update};
use crate::session::{ApprovalStatus, TuiSession};

impl InteractiveWorkflowEngine {
    pub(crate) fn diff_summary(&self) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        let mut lines = vec![
            "diff summary".to_string(),
            format!("approval: {}", approval_status(&session.approval_status)),
            format!(
                "worktree: {}",
                session
                    .worktree
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "not recorded".to_string())
            ),
        ];
        if let Some(diff_stat) = &session.diff_stat {
            lines.push("stat:".to_string());
            lines.extend(diff_stat.lines().map(|line| format!("  {line}")));
        }
        if session.changed_files.is_empty() {
            lines.push("changed files: none recorded".to_string());
        } else {
            lines.push("changed files:".to_string());
            lines.extend(session.changed_files.iter().filter_map(|file| {
                let path = file.get("path")?.as_str()?;
                let lines = file
                    .get("lines")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0);
                Some(format!("  {path} ({lines} lines changed)"))
            }));
        }
        Ok(update(lines, inspector_for(session), Some(session.clone())))
    }

    pub(crate) fn diff_file(&self, requested_path: &str) -> Result<WorkflowUpdate> {
        let session = self.active()?;
        let safe_path = safe_relative_path(requested_path)?;
        ensure_changed_file(session, &safe_path)?;
        let worktree = session
            .worktree
            .as_ref()
            .context("run adapter before requesting a focused diff")?;
        let diff = focused_diff(worktree, &safe_path)?;
        let lines = if diff.trim().is_empty() {
            vec![format!(
                "diff file {}: no diff content",
                safe_path.display()
            )]
        } else {
            std::iter::once(format!("diff file {}", safe_path.display()))
                .chain(diff.lines().map(ToString::to_string))
                .collect()
        };
        Ok(update(lines, inspector_for(session), Some(session.clone())))
    }
}

fn approval_status(status: &ApprovalStatus) -> &'static str {
    match status {
        ApprovalStatus::Pending => "pending",
        ApprovalStatus::Approved => "approved",
        ApprovalStatus::Rejected => "rejected",
        ApprovalStatus::Promoted => "promoted",
        ApprovalStatus::Override => "override",
    }
}

fn safe_relative_path(path: &str) -> Result<PathBuf> {
    let normalized = path.replace('\\', "/");
    if normalized
        .as_bytes()
        .get(1)
        .is_some_and(|byte| *byte == b':')
    {
        anyhow::bail!("diff file path must be a safe relative workspace path");
    }
    let candidate = Path::new(&normalized);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        anyhow::bail!("diff file path must be a safe relative workspace path");
    }
    Ok(candidate.to_path_buf())
}

fn ensure_changed_file(session: &TuiSession, path: &Path) -> Result<()> {
    let requested = normalized_workspace_path(path);
    let listed = session.changed_files.iter().any(|file| {
        file.get("path")
            .and_then(|value| value.as_str())
            .is_some_and(|changed| normalize_path_text(changed) == requested)
    });
    if listed {
        return Ok(());
    }
    anyhow::bail!("{requested} is not in the recorded changed-file set");
}

fn focused_diff(worktree: &Path, path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .args(["diff", "--"])
        .arg(path)
        .output()
        .context("failed to run git diff for focused file")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            "git diff exited without stderr".to_string()
        } else {
            stderr
        };
        anyhow::bail!("git diff failed for {}: {detail}", path.display());
    }
    let diff = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !diff.is_empty() {
        return Ok(diff);
    }
    let full_path = worktree.join(path);
    if full_path.exists() {
        let content = std::fs::read_to_string(&full_path)
            .with_context(|| format!("failed to read {}", full_path.display()))?;
        return Ok(format!(
            "untracked file content:\n{}",
            truncate_for_terminal(&content)
        ));
    }
    Ok(String::new())
}

fn truncate_for_terminal(content: &str) -> String {
    const LIMIT: usize = 8192;
    if content.len() <= LIMIT {
        content.to_string()
    } else {
        let end = content
            .char_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index <= LIMIT)
            .last()
            .unwrap_or(0);
        format!("{}\n[truncated after {LIMIT} bytes]", &content[..end])
    }
}

fn normalized_workspace_path(path: &Path) -> String {
    normalize_path_text(&path.to_string_lossy())
}

fn normalize_path_text(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TuiConfig;
    use crate::orchestrator::Orchestrator;
    use serde_json::json;
    use std::process::Command;

    #[test]
    fn rejects_unsafe_diff_paths() {
        assert!(safe_relative_path("../secret").is_err());
        assert!(safe_relative_path("/tmp/secret").is_err());
        assert!(safe_relative_path("C:\\secret").is_err());
        assert_eq!(
            safe_relative_path("docs/live-qa.md").expect("safe"),
            PathBuf::from("docs/live-qa.md")
        );
        assert_eq!(
            safe_relative_path("docs\\live-qa.md").expect("safe"),
            PathBuf::from("docs/live-qa.md")
        );
    }

    #[test]
    fn diff_commands_show_summary_and_focused_file_content() {
        let temp = tempfile::tempdir().expect("tempdir");
        run_git(temp.path(), ["init"]);
        run_git(temp.path(), ["config", "user.email", "test@example.com"]);
        run_git(temp.path(), ["config", "user.name", "architect mcp test"]);
        std::fs::create_dir_all(temp.path().join("docs")).expect("docs");
        std::fs::write(temp.path().join("docs/live-qa.md"), "before\n").expect("file");
        run_git(temp.path(), ["add", "docs/live-qa.md"]);
        run_git(temp.path(), ["commit", "-m", "init"]);
        std::fs::write(temp.path().join("docs/live-qa.md"), "after\n").expect("file");

        let mut session = TuiSession::new("build", "shell");
        session.worktree = Some(temp.path().to_path_buf());
        session.diff_stat = Some("docs/live-qa.md | 2 +-".to_string());
        session.changed_files = vec![json!({ "path": "docs/live-qa.md", "lines": 2 })];
        let mut engine =
            InteractiveWorkflowEngine::new(Orchestrator::new(temp.path(), TuiConfig::default()));
        engine.active = Some(session);

        let summary = engine
            .diff_summary()
            .expect("summary")
            .transcript
            .join("\n");
        assert!(summary.contains("diff summary"));
        assert!(summary.contains("docs/live-qa.md"));

        let diff = engine
            .diff_file("docs\\live-qa.md")
            .expect("diff")
            .transcript
            .join("\n");
        assert!(diff.contains("diff file docs/live-qa.md"));
        assert!(diff.contains("-before") || diff.contains("+after"));
    }

    #[test]
    fn truncates_untracked_unicode_content_on_valid_boundary() {
        let content = format!("{}étail", "a".repeat(8191));
        let truncated = truncate_for_terminal(&content);
        assert!(truncated.contains("[truncated after 8192 bytes]"));
        assert!(truncated.is_char_boundary(truncated.find('\n').expect("marker line")));
    }

    #[test]
    fn focused_diff_reports_git_failures() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(temp.path().join("docs")).expect("docs");
        std::fs::write(temp.path().join("docs/live-qa.md"), "content\n").expect("file");
        let error = focused_diff(temp.path(), Path::new("docs/live-qa.md"))
            .expect_err("non-git worktree should fail");
        assert!(error.to_string().contains("git diff failed"));
    }

    fn run_git<const N: usize>(path: &Path, args: [&str; N]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("git command");
        assert!(
            output.status.success(),
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
