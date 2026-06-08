use std::fs;
use std::path::Path;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum LaunchScope {
    LocalDemo,
    InternalV0,
    PublicCli,
    Production,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchScopeSource {
    Supplied,
    Inferred,
    Defaulted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchScopeSummary {
    pub name: LaunchScope,
    pub resolution: LaunchScopeSource,
    pub signals: Vec<String>,
}

impl LaunchScope {
    pub(crate) fn treats_missing_terminal_evidence_as_future(self) -> bool {
        matches!(self, LaunchScope::LocalDemo | LaunchScope::InternalV0)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            LaunchScope::LocalDemo => "local-demo",
            LaunchScope::InternalV0 => "internal-v0",
            LaunchScope::PublicCli => "public-cli",
            LaunchScope::Production => "production",
        }
    }
}

pub(crate) fn resolve_launch_scope(
    workspace: &Path,
    supplied: Option<LaunchScope>,
) -> LaunchScopeSummary {
    if let Some(name) = supplied {
        return LaunchScopeSummary {
            name,
            resolution: LaunchScopeSource::Supplied,
            signals: vec!["scope supplied by CLI flag".to_string()],
        };
    }

    infer_launch_scope(workspace)
}

fn infer_launch_scope(workspace: &Path) -> LaunchScopeSummary {
    let text = readiness_text(workspace);
    let normalized = text.to_ascii_lowercase();

    if normalized.contains("production ready")
        || normalized.contains("production-ready")
        || normalized.contains("customer ready")
        || normalized.contains("customer-ready")
    {
        return summary(
            LaunchScope::Production,
            LaunchScopeSource::Inferred,
            vec!["readiness docs contain production/customer-ready language".to_string()],
        );
    }

    if normalized.contains("internal v0") || normalized.contains("internal-v0") {
        return summary(
            LaunchScope::InternalV0,
            LaunchScopeSource::Inferred,
            vec!["readiness docs contain internal-v0 language".to_string()],
        );
    }

    let local_demo_claim = normalized.contains("local synthetic v0 demo")
        || normalized.contains("local demo ready")
        || normalized.contains("local-demo");
    let explicit_not_public = normalized.contains("not public launch")
        || normalized.contains("not public-launch")
        || normalized.contains("not production")
        || normalized.contains("not customer");
    if local_demo_claim && explicit_not_public {
        return summary(
            LaunchScope::LocalDemo,
            LaunchScopeSource::Inferred,
            vec![
                "readiness docs contain local-demo readiness language".to_string(),
                "readiness docs explicitly avoid production/customer/public launch claims"
                    .to_string(),
            ],
        );
    }

    summary(
        LaunchScope::PublicCli,
        LaunchScopeSource::Defaulted,
        vec!["no narrower launch scope was supplied or confidently inferred".to_string()],
    )
}

fn readiness_text(workspace: &Path) -> String {
    let mut parts = Vec::new();
    for relative in [
        "README.md",
        "docs/architecture-contract.md",
        "docs/build-plan.md",
        "docs/release-readiness.md",
        "docs/readiness.md",
    ] {
        if let Ok(text) = fs::read_to_string(workspace.join(relative)) {
            parts.push(text);
        }
    }
    if let Ok(entries) = fs::read_dir(workspace.join("docs/readiness")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "md")
                && let Ok(text) = fs::read_to_string(path)
            {
                parts.push(text);
            }
        }
    }
    parts.join("\n")
}

fn summary(
    name: LaunchScope,
    source: LaunchScopeSource,
    signals: Vec<String>,
) -> LaunchScopeSummary {
    LaunchScopeSummary {
        name,
        resolution: source,
        signals,
    }
}
