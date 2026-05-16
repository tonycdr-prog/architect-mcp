use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::adapter::AdapterConfig;
use crate::adapter_probe_command::output_with_timeout;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const CMD_PROBE_ARGS: &[&str] = &["/C", "ver"];
const POSIX_PROBE_ARGS: &[&str] = &["-c", "true"];
const POWERSHELL_PROBE_ARGS: &[&str] = &["-NoProfile", "-Command", "exit 0"];

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
    if is_shell_adapter(name, &config.command) {
        return probe_shell_health(name, config);
    }

    let output = output_with_timeout(
        Command::new(&config.command).arg("--version"),
        PROBE_TIMEOUT,
    );
    match output {
        Ok(Some(output)) if output.status.success() => {
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
        Ok(None) => AdapterHealth {
            name: name.to_string(),
            command: config.command.clone(),
            installed: false,
            version: None,
            auth_status: AuthStatus::Unknown,
            ready: false,
            detail: "command timed out during --version probe".to_string(),
        },
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

fn probe_shell_health(name: &str, config: &AdapterConfig) -> AdapterHealth {
    let mut command = Command::new(&config.command);
    command.args(shell_probe_args(&config.command));
    match output_with_timeout(&mut command, PROBE_TIMEOUT) {
        Ok(Some(output)) if output.status.success() => AdapterHealth {
            name: name.to_string(),
            command: config.command.clone(),
            installed: true,
            version: None,
            auth_status: AuthStatus::NotApplicable,
            ready: true,
            detail: "shell adapter".to_string(),
        },
        Ok(None) => AdapterHealth {
            name: name.to_string(),
            command: config.command.clone(),
            installed: false,
            version: None,
            auth_status: AuthStatus::Unknown,
            ready: false,
            detail: "command timed out during shell probe".to_string(),
        },
        _ => AdapterHealth {
            name: name.to_string(),
            command: config.command.clone(),
            installed: false,
            version: None,
            auth_status: AuthStatus::Unknown,
            ready: false,
            detail: "command not found or shell probe failed".to_string(),
        },
    }
}

fn shell_probe_args(command: &str) -> &'static [&'static str] {
    if cfg!(windows) {
        let shell = command_file_name(command).unwrap_or_default();
        if matches!(
            shell.as_str(),
            "pwsh" | "pwsh.exe" | "powershell" | "powershell.exe"
        ) {
            return POWERSHELL_PROBE_ARGS;
        }
        return CMD_PROBE_ARGS;
    }
    POSIX_PROBE_ARGS
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
    if is_shell_adapter(name, &config.command) {
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

fn is_shell_adapter(name: &str, command: &str) -> bool {
    name.eq_ignore_ascii_case("shell")
        || command_file_name(command).is_some_and(|file_name| {
            matches!(
                file_name.as_str(),
                "sh" | "bash" | "zsh" | "fish" | "pwsh" | "powershell" | "cmd" | "cmd.exe"
            )
        })
}

fn is_codex_adapter(name: &str, command: &str) -> bool {
    name.eq_ignore_ascii_case("codex")
        || command_file_name(command)
            .is_some_and(|file_name| file_name == "codex" || file_name == "codex.exe")
}

fn command_file_name(command: &str) -> Option<String> {
    Path::new(command)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| file_name.to_ascii_lowercase())
}

fn probe_codex_auth(command: &str) -> (AuthStatus, Option<String>) {
    match output_with_timeout(
        Command::new(command).args(["login", "status"]),
        PROBE_TIMEOUT,
    ) {
        Ok(Some(output)) => {
            let text = joined_output(&output.stdout, &output.stderr);
            let detail = text
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .unwrap_or_else(|| "codex login status returned no output".to_string());
            let status = codex_auth_status_from_output(output.status.success(), &text);
            (status, Some(detail))
        }
        Ok(None) => (
            AuthStatus::Unknown,
            Some("codex login status timed out".to_string()),
        ),
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
mod tests;
