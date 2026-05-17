use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::launch_judge_report::LaunchJudgeTerminalEvidenceReport;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchJudgeTerminalEvidenceEnvironment {
    LocalTerminal,
    VmOrCloudTerminal,
    Container,
    HostedCi,
    Unknown,
}

pub(crate) fn resolve_environment(
    environment: Option<&str>,
) -> Result<LaunchJudgeTerminalEvidenceEnvironment> {
    match environment.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => normalize_environment(value),
        None => Ok(auto_detect_environment()),
    }
}

pub(crate) fn validate_report_environment(
    report: &LaunchJudgeTerminalEvidenceReport,
    issues: &mut Vec<String>,
) {
    match report.environment {
        Some(LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal)
        | Some(LaunchJudgeTerminalEvidenceEnvironment::VmOrCloudTerminal) => {}
        Some(LaunchJudgeTerminalEvidenceEnvironment::Container) => issues.push(format!(
            "terminal evidence for '{}' came from a container; useful smoke evidence, but final launch go requires a local terminal or VM/cloud terminal report",
            report.platform
        )),
        Some(LaunchJudgeTerminalEvidenceEnvironment::HostedCi) => issues.push(format!(
            "terminal evidence for '{}' came from hosted CI; hosted runner evidence does not satisfy manual terminal QA",
            report.platform
        )),
        Some(LaunchJudgeTerminalEvidenceEnvironment::Unknown) => issues.push(format!(
            "terminal evidence for '{}' has unknown environment provenance; use local_terminal or vm_or_cloud_terminal for final launch go",
            report.platform
        )),
        None => issues.push(format!(
            "terminal evidence for '{}' needs environment provenance; use local_terminal or vm_or_cloud_terminal for final launch go",
            report.platform
        )),
    }
}

fn normalize_environment(value: &str) -> Result<LaunchJudgeTerminalEvidenceEnvironment> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "local" | "local_terminal" | "manual" | "manual_terminal" => {
            Ok(LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal)
        }
        "vm"
        | "cloud"
        | "cloud_terminal"
        | "devbox"
        | "dev_box"
        | "vm_or_cloud"
        | "vm_or_cloud_terminal" => Ok(LaunchJudgeTerminalEvidenceEnvironment::VmOrCloudTerminal),
        "container" | "docker" | "podman" | "lxc" | "devcontainer" => {
            Ok(LaunchJudgeTerminalEvidenceEnvironment::Container)
        }
        "ci" | "hosted_ci" | "github_actions" | "gha" | "runner" => {
            Ok(LaunchJudgeTerminalEvidenceEnvironment::HostedCi)
        }
        "unknown" => Ok(LaunchJudgeTerminalEvidenceEnvironment::Unknown),
        other => anyhow::bail!(
            "unsupported terminal evidence environment '{other}'; use local-terminal, vm-or-cloud-terminal, container, hosted-ci, or unknown"
        ),
    }
}

fn auto_detect_environment() -> LaunchJudgeTerminalEvidenceEnvironment {
    if env_flag("GITHUB_ACTIONS") || env_flag("CI") || std::env::var_os("BUILD_BUILDID").is_some() {
        return LaunchJudgeTerminalEvidenceEnvironment::HostedCi;
    }
    if std::path::Path::new("/.dockerenv").exists()
        || std::env::var_os("KUBERNETES_SERVICE_HOST").is_some()
        || std::env::var("container")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    {
        return LaunchJudgeTerminalEvidenceEnvironment::Container;
    }
    LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false)
}
