use std::fs;
use std::path::Path;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum GovernanceRepoProfile {
    ArchitectMcpSelf,
    StaticWebApp,
    NodePackage,
    RustWorkspace,
    TuiApp,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceProfileConfidence {
    Supplied,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceRepoProfileSummary {
    pub name: GovernanceRepoProfile,
    pub confidence: GovernanceProfileConfidence,
    pub signals: Vec<String>,
}

impl GovernanceRepoProfile {
    pub(crate) fn requires_cargo_lock(self) -> bool {
        matches!(
            self,
            GovernanceRepoProfile::ArchitectMcpSelf
                | GovernanceRepoProfile::RustWorkspace
                | GovernanceRepoProfile::TuiApp
        )
    }

    pub(crate) fn requires_rust_script(self) -> bool {
        self.requires_cargo_lock()
    }

    pub(crate) fn requires_tui_live_qa(self) -> bool {
        matches!(
            self,
            GovernanceRepoProfile::ArchitectMcpSelf | GovernanceRepoProfile::TuiApp
        )
    }

    pub(crate) fn requires_npm_publish_workflow(self) -> bool {
        matches!(
            self,
            GovernanceRepoProfile::ArchitectMcpSelf | GovernanceRepoProfile::NodePackage
        )
    }

    pub(crate) fn requires_check_v10(self) -> bool {
        self == GovernanceRepoProfile::ArchitectMcpSelf
    }
}

pub(crate) fn resolve_governance_profile(
    workspace: &Path,
    supplied: Option<GovernanceRepoProfile>,
) -> GovernanceRepoProfileSummary {
    if let Some(name) = supplied {
        return GovernanceRepoProfileSummary {
            name,
            confidence: GovernanceProfileConfidence::Supplied,
            signals: vec!["profile supplied by CLI flag".to_string()],
        };
    }

    infer_governance_profile(workspace)
}

fn infer_governance_profile(workspace: &Path) -> GovernanceRepoProfileSummary {
    let package = package_json(workspace);
    let scripts = package_scripts(package.as_ref());
    let dependencies = dependency_names(package.as_ref());
    let package_name = package
        .as_ref()
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let private = package
        .as_ref()
        .and_then(|value| value.get("private"))
        .and_then(Value::as_bool);
    let has_cargo = workspace.join("Cargo.toml").is_file();
    let has_architect_tui_crate = workspace.join("crates/architect-tui").is_dir();
    let has_tui_script = scripts.iter().any(|script| script == "tui:live-qa");
    let has_tui_workflow = workspace
        .join(".github/workflows/tui-live-qa.yml")
        .is_file();
    let has_npm_publish_workflow = workspace
        .join(".github/workflows/npm-publish.yml")
        .is_file();
    let has_publish_metadata = package.as_ref().is_some_and(|value| {
        value.get("publishConfig").is_some()
            || value.get("bin").is_some()
            || value.get("exports").is_some()
            || value.get("main").is_some()
    });
    let has_static_web_stack = dependencies.iter().any(|name| {
        matches!(
            name.as_str(),
            "vite" | "@vitejs/plugin-react" | "react" | "react-dom" | "next" | "astro"
        )
    });

    if (package_name == "@tonycdr-prog/architect-mcp" || package_name == "architect-mcp")
        && has_architect_tui_crate
    {
        return summary(
            GovernanceRepoProfile::ArchitectMcpSelf,
            GovernanceProfileConfidence::High,
            [
                format!("package name is {package_name}"),
                "crates/architect-tui exists".to_string(),
            ],
        );
    }

    if has_tui_script || has_tui_workflow || has_architect_tui_crate {
        return summary(
            GovernanceRepoProfile::TuiApp,
            GovernanceProfileConfidence::High,
            compact_signals([
                has_tui_script.then_some("package script tui:live-qa exists".to_string()),
                has_tui_workflow.then_some(".github/workflows/tui-live-qa.yml exists".to_string()),
                has_architect_tui_crate.then_some("crates/architect-tui exists".to_string()),
            ]),
        );
    }

    if has_cargo {
        return summary(
            GovernanceRepoProfile::RustWorkspace,
            GovernanceProfileConfidence::High,
            ["Cargo.toml exists".to_string()],
        );
    }

    if has_static_web_stack && private != Some(false) {
        return summary(
            GovernanceRepoProfile::StaticWebApp,
            if private == Some(true) {
                GovernanceProfileConfidence::High
            } else {
                GovernanceProfileConfidence::Medium
            },
            compact_signals([
                Some("static web dependency detected".to_string()),
                (private == Some(true)).then_some("package.json private=true".to_string()),
                (!has_cargo).then_some("no Cargo.toml".to_string()),
            ]),
        );
    }

    if has_npm_publish_workflow || (private == Some(false) && has_publish_metadata) {
        return summary(
            GovernanceRepoProfile::NodePackage,
            if has_npm_publish_workflow {
                GovernanceProfileConfidence::High
            } else {
                GovernanceProfileConfidence::Medium
            },
            compact_signals([
                has_npm_publish_workflow
                    .then_some(".github/workflows/npm-publish.yml exists".to_string()),
                (private == Some(false)).then_some("package.json private=false".to_string()),
                has_publish_metadata.then_some("package publish metadata exists".to_string()),
            ]),
        );
    }

    summary(
        GovernanceRepoProfile::Unknown,
        GovernanceProfileConfidence::Low,
        ["no high-confidence governance profile signals found".to_string()],
    )
}

fn package_json(workspace: &Path) -> Option<Value> {
    let package = fs::read_to_string(workspace.join("package.json")).ok()?;
    serde_json::from_str(&package).ok()
}

fn package_scripts(package: Option<&Value>) -> Vec<String> {
    package
        .and_then(|value| value.get("scripts"))
        .and_then(Value::as_object)
        .map(|scripts| scripts.keys().cloned().collect())
        .unwrap_or_default()
}

fn dependency_names(package: Option<&Value>) -> Vec<String> {
    let mut names = Vec::new();
    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(dependencies) = package
            .and_then(|value| value.get(section))
            .and_then(Value::as_object)
        {
            names.extend(dependencies.keys().cloned());
        }
    }
    names
}

fn summary<I>(
    name: GovernanceRepoProfile,
    confidence: GovernanceProfileConfidence,
    signals: I,
) -> GovernanceRepoProfileSummary
where
    I: IntoIterator<Item = String>,
{
    GovernanceRepoProfileSummary {
        name,
        confidence,
        signals: signals.into_iter().collect(),
    }
}

fn compact_signals<const N: usize>(signals: [Option<String>; N]) -> Vec<String> {
    signals.into_iter().flatten().collect()
}
