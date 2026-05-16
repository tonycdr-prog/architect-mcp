use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::adapter::AdapterConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthStatus {
    Authenticated,
    NeedsLogin,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdapterHealth {
    pub name: String,
    pub command: String,
    pub installed: bool,
    pub version: Option<String>,
    pub auth_status: AuthStatus,
    pub ready: bool,
    pub detail: String,
}

pub fn probe_adapter_health(name: &str, config: &AdapterConfig) -> AdapterHealth {
    if config.available == Some(false) {
        return AdapterHealth {
            name: name.to_string(),
            command: config.command.clone(),
            installed: false,
            version: None,
            auth_status: AuthStatus::Unknown,
            ready: false,
            detail: "disabled by configuration".to_string(),
        };
    }

    let output = Command::new(&config.command).arg("--version").output();
    match output {
        Ok(output) if output.status.success() => {
            let version = first_output_line(&output.stdout, &output.stderr);
            let (auth_status, auth_detail) = probe_adapter_auth(name, config);
            let ready = matches!(
                auth_status,
                AuthStatus::Authenticated | AuthStatus::NotApplicable
            );
            AdapterHealth {
                name: name.to_string(),
                command: config.command.clone(),
                installed: true,
                version,
                auth_status,
                ready,
                detail: auth_detail.unwrap_or_else(|| "installed".to_string()),
            }
        }
        _ => AdapterHealth {
            name: name.to_string(),
            command: config.command.clone(),
            installed: false,
            version: None,
            auth_status: AuthStatus::Unknown,
            ready: false,
            detail: "command not found or --version failed".to_string(),
        },
    }
}

pub fn adapter_healths(adapters: &BTreeMap<String, AdapterConfig>) -> Vec<AdapterHealth> {
    adapters
        .iter()
        .map(|(name, adapter)| probe_adapter_health(name, adapter))
        .collect()
}

pub fn print_adapter_table(adapters: &BTreeMap<String, AdapterConfig>, json: bool) -> Result<()> {
    let healths = adapter_healths(adapters);
    if json {
        println!("{}", serde_json::to_string_pretty(&healths)?);
        return Ok(());
    }

    for (name, adapter) in adapters {
        let health = healths
            .iter()
            .find(|health| health.name == *name)
            .expect("health generated for adapter");
        let version = health.version.as_deref().unwrap_or("-");
        println!(
            "{name}\t{}\tinstalled={}\tauth={:?}\tready={}\t{version}\t{}",
            adapter.command, health.installed, health.auth_status, health.ready, health.detail
        );
    }
    Ok(())
}

fn probe_adapter_auth(name: &str, config: &AdapterConfig) -> (AuthStatus, Option<String>) {
    if name == "shell" {
        return (AuthStatus::NotApplicable, Some("shell adapter".to_string()));
    }
    if is_codex_adapter(name, &config.command) {
        return probe_codex_auth(&config.command);
    }
    (
        AuthStatus::Unknown,
        Some("auth probe not implemented for this adapter".to_string()),
    )
}

fn is_codex_adapter(name: &str, command: &str) -> bool {
    if name.eq_ignore_ascii_case("codex") {
        return true;
    }
    Path::new(command)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| {
            file_name.eq_ignore_ascii_case("codex") || file_name.eq_ignore_ascii_case("codex.exe")
        })
        .unwrap_or(false)
}

fn probe_codex_auth(command: &str) -> (AuthStatus, Option<String>) {
    match Command::new(command).args(["login", "status"]).output() {
        Ok(output) => {
            let text = joined_output(&output.stdout, &output.stderr);
            let detail = text
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .unwrap_or_else(|| "codex login status returned no output".to_string());
            let status = codex_auth_status_from_output(output.status.success(), &text);
            (status, Some(detail))
        }
        Err(error) => (
            AuthStatus::Unknown,
            Some(format!("could not run codex login status: {error}")),
        ),
    }
}

pub fn codex_auth_status_from_output(success: bool, output: &str) -> AuthStatus {
    if success && output.contains("Logged in") {
        AuthStatus::Authenticated
    } else {
        AuthStatus::NeedsLogin
    }
}

fn first_output_line(stdout: &[u8], stderr: &[u8]) -> Option<String> {
    let output = joined_output(stdout, stderr);
    output
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
}

fn joined_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    format!("{stdout}{stderr}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_auth_probe_requires_logged_in_success() {
        assert_eq!(
            codex_auth_status_from_output(true, "Logged in as user@example.com"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            codex_auth_status_from_output(true, "Not logged in"),
            AuthStatus::NeedsLogin
        );
        assert_eq!(
            codex_auth_status_from_output(false, "Logged in"),
            AuthStatus::NeedsLogin
        );
    }

    #[test]
    fn adapter_health_json_uses_public_field_names() {
        let health = AdapterHealth {
            name: "codex".to_string(),
            command: "codex".to_string(),
            installed: true,
            version: Some("codex 1.0.0".to_string()),
            auth_status: AuthStatus::Authenticated,
            ready: true,
            detail: "Logged in".to_string(),
        };
        let json = serde_json::to_value(&health).expect("json");
        assert_eq!(json["authStatus"], "authenticated");
        assert_eq!(json["ready"], true);
    }
}
