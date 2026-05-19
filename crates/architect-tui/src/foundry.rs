use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::foundry_artifacts::artifact_plan;
use crate::session::TuiSession;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepoFoundryPlan {
    pub repo_name: String,
    pub owner: Option<String>,
    pub visibility: RepoVisibility,
    pub default_branch: String,
    pub artifacts: Vec<FoundryArtifact>,
    pub verification: Vec<String>,
    pub first_pr: FoundryPullRequestPlan,
    pub mutation_commands: Vec<String>,
    pub approval_required_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepoVisibility {
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryArtifact {
    pub path: String,
    pub source: String,
    pub purpose: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryPullRequestPlan {
    pub title: String,
    pub body_sections: Vec<String>,
    pub draft: bool,
}

pub fn build_repo_foundry_plan(
    session: &TuiSession,
    repo_name: &str,
    owner: Option<&str>,
) -> Result<RepoFoundryPlan> {
    let repo_name = validate_repo_name(repo_name)?;
    let owner = owner.map(validate_owner).transpose()?;
    let verification = foundry_verification(session);
    let target = repo_target(owner.as_deref(), &repo_name);
    let artifact_paths = artifact_plan(session, &verification);
    let first_pr_title = format!("[architect-mcp] Bootstrap {repo_name}");
    Ok(RepoFoundryPlan {
        repo_name: repo_name.clone(),
        owner,
        visibility: RepoVisibility::Private,
        default_branch: "main".to_string(),
        artifacts: artifact_paths,
        verification,
        first_pr: FoundryPullRequestPlan {
            title: first_pr_title.clone(),
            body_sections: vec![
                "Work gate evidence: grill, contract, build-plan review, and file-plan review passed before repo creation.".to_string(),
                "Verification evidence must be copied from actual command output before the PR is opened.".to_string(),
                "Remaining gaps and assumptions must be stated explicitly; do not claim generated files were tested unless they were.".to_string(),
            ],
            draft: true,
        },
        mutation_commands: vec![
            format!(
                "gh repo create {target} --private --source <staged-repo> --remote origin"
            ),
            "git -C <staged-repo> push -u origin main".to_string(),
            "git -C <staged-repo> push -u origin architect/bootstrap".to_string(),
            format!("gh -R {target} pr create --draft --base main --head architect/bootstrap --title \"{first_pr_title}\" --body-file <staged-repo>/docs/first-pr-draft.md"),
        ],
        approval_required_for: vec![
            "create private GitHub repository".to_string(),
            "push initial branch".to_string(),
            "open first draft pull request".to_string(),
        ],
    })
}

pub fn render_foundry_create_preview(plan: &RepoFoundryPlan) -> Vec<String> {
    let target = repo_target(plan.owner.as_deref(), &plan.repo_name);
    let mut lines = vec![
        "foundry create: dry-run only; no GitHub mutation performed".to_string(),
        format!("repo: {target}"),
        "visibility: private".to_string(),
        format!("default branch: {}", plan.default_branch),
    ];
    lines.extend(
        plan.mutation_commands
            .iter()
            .map(|command| format!("would run: {command}")),
    );
    lines.push(
        "next: foundry stage, then approve again before foundry create --execute".to_string(),
    );
    lines
}

pub fn foundry_plan_lines(plan: &RepoFoundryPlan) -> Vec<String> {
    let target = repo_target(plan.owner.as_deref(), &plan.repo_name);
    let mut lines = vec![
        "foundry plan ready".to_string(),
        format!("repo: {target}"),
        "visibility: private".to_string(),
        format!("artifacts: {}", plan.artifacts.len()),
    ];
    lines.extend(
        plan.artifacts
            .iter()
            .take(8)
            .map(|artifact| format!("artifact: {}", artifact.path)),
    );
    lines.push("next: foundry approve <reason>, then foundry stage".to_string());
    lines
}

pub fn foundry_status_lines(session: &TuiSession) -> Vec<String> {
    let Some(plan) = &session.foundry_plan else {
        return vec![
            "foundry: no plan".to_string(),
            "next: run foundry plan <repo-name> after review files".to_string(),
        ];
    };
    let mut lines = foundry_plan_lines(plan);
    lines.push(format!("foundry approved: {}", session.foundry_approved));
    if let Some(reason) = &session.foundry_approval_reason {
        lines.push(format!("approval reason: {reason}"));
    }
    if let Some(stage) = &session.foundry_stage {
        lines.push(format!("staged repo: {}", stage.path.display()));
        lines.push(format!("staged artifacts: {}", stage.artifacts_written));
    }
    if let Some(execution) = &session.foundry_execution {
        lines.push(format!("github execution: {}", execution.status));
        lines.push(format!("commands run: {}", execution.commands.len()));
    }
    lines
}

fn foundry_verification(session: &TuiSession) -> Vec<String> {
    if !session.required_verification.is_empty() {
        return session.required_verification.clone();
    }
    session
        .brief
        .get("verification")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec!["npm run release:check".to_string()])
}

fn validate_repo_name(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("foundry plan requires a repo name");
    }
    if value == "." || value == ".." || value.contains('/') || value.contains('\\') {
        bail!("repo name must be a single GitHub repository name");
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!("repo name contains unsupported characters");
    }
    Ok(value.to_string())
}

fn validate_owner(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("owner cannot be blank");
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-'))
    {
        bail!("owner contains unsupported characters");
    }
    Ok(value.to_string())
}

pub fn repo_target(owner: Option<&str>, repo_name: &str) -> String {
    owner
        .map(|owner| format!("{owner}/{repo_name}"))
        .unwrap_or_else(|| repo_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::TuiSession;

    #[test]
    fn rejects_path_like_repo_names() {
        let session = TuiSession::new("ready app", "codex");
        let error = build_repo_foundry_plan(&session, "../public", None).expect_err("invalid");
        assert!(error.to_string().contains("single GitHub repository name"));
    }
}
