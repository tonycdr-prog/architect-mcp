use std::collections::BTreeMap;
use std::process::Command;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use crate::adapter_pty::{PtyRunOptions, run_adapter_pty};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct AdapterConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub working_directory: Option<String>,
    pub available: Option<bool>,
    pub pty: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AdapterProbe {
    pub name: String,
    pub command: String,
    pub available: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum AgentEvent {
    Started { adapter: String },
    Output { stream: String, text: String },
    Completed { exit_code: Option<i32> },
    TimedOut,
    Cancelled,
    Crashed { message: String },
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            env: BTreeMap::new(),
            working_directory: None,
            available: None,
            pty: true,
        }
    }
}

impl AdapterConfig {
    pub fn validate(&self, name: &str) -> Result<()> {
        if self.command.trim().is_empty() {
            anyhow::bail!("adapter '{name}' command must not be empty");
        }
        for (key, value) in &self.env {
            let lower = key.to_ascii_lowercase();
            let secret_like = lower.contains("token")
                || lower.contains("secret")
                || lower.contains("password")
                || lower.contains("api_key");
            if secret_like && !value.starts_with('$') && !value.starts_with("${") {
                anyhow::bail!(
                    "adapter '{name}' env '{key}' must reference an environment variable, not an inline value"
                );
            }
        }
        Ok(())
    }
}

pub fn default_adapters() -> BTreeMap<String, AdapterConfig> {
    BTreeMap::from([
        (
            "codex".to_string(),
            adapter("codex", std::iter::empty::<&str>()),
        ),
        (
            "claude".to_string(),
            adapter("claude", std::iter::empty::<&str>()),
        ),
        (
            "gemini".to_string(),
            adapter("gemini", std::iter::empty::<&str>()),
        ),
        (
            "opencode".to_string(),
            adapter("opencode", std::iter::empty::<&str>()),
        ),
        (
            "aider".to_string(),
            adapter("aider", std::iter::empty::<&str>()),
        ),
        (
            "shell".to_string(),
            adapter(default_shell(), std::iter::empty::<&str>()),
        ),
    ])
}

fn adapter<I, S>(command: &str, args: I) -> AdapterConfig
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    AdapterConfig {
        command: command.to_string(),
        args: args
            .into_iter()
            .map(|arg| arg.as_ref().to_string())
            .collect(),
        env: BTreeMap::new(),
        working_directory: None,
        available: None,
        pty: true,
    }
}

fn default_shell() -> &'static str {
    if cfg!(windows) { "cmd" } else { "sh" }
}

pub fn probe_adapter(name: &str, config: &AdapterConfig) -> AdapterProbe {
    if config.available == Some(false) {
        return AdapterProbe {
            name: name.to_string(),
            command: config.command.clone(),
            available: false,
            version: None,
        };
    }

    let output = Command::new(&config.command).arg("--version").output();
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version = stdout
                .lines()
                .chain(stderr.lines())
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string());
            AdapterProbe {
                name: name.to_string(),
                command: config.command.clone(),
                available: true,
                version,
            }
        }
        _ => AdapterProbe {
            name: name.to_string(),
            command: config.command.clone(),
            available: false,
            version: None,
        },
    }
}

pub fn print_adapter_table(adapters: &BTreeMap<String, AdapterConfig>) -> Result<()> {
    for (name, adapter) in adapters {
        let probe = probe_adapter(name, adapter);
        let status = if probe.available {
            "available"
        } else {
            "unavailable"
        };
        let version = probe.version.unwrap_or_else(|| "-".to_string());
        println!("{name}\t{}\t{status}\t{version}", adapter.command);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_templates_cover_named_agents_and_shell() {
        let adapters = default_adapters();
        for name in ["codex", "claude", "gemini", "opencode", "aider", "shell"] {
            assert!(adapters.contains_key(name), "missing {name}");
            assert!(adapters[name].pty);
        }
    }

    #[test]
    fn probe_honors_forced_unavailable() {
        let probe = probe_adapter(
            "codex",
            &AdapterConfig {
                command: "codex".to_string(),
                available: Some(false),
                ..AdapterConfig::default()
            },
        );
        assert!(!probe.available);
    }
}
