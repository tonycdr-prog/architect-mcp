use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use crate::adapter_health::{
    AdapterHealth, AuthStatus, adapter_healths, codex_auth_status_from_output, print_adapter_table,
    probe_adapter_health,
};
pub use crate::adapter_pty::{PtyRunOptions, run_adapter_process, run_adapter_pty};

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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    Started {
        adapter: String,
    },
    Output {
        stream: String,
        text: String,
        truncated: bool,
    },
    Completed {
        exit_code: Option<i32>,
    },
    TimedOut,
    Cancelled,
    Crashed {
        message: String,
    },
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
        ("codex".to_string(), codex_exec_adapter()),
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

fn codex_exec_adapter() -> AdapterConfig {
    adapter(
        "codex",
        [
            "exec",
            "--sandbox",
            "workspace-write",
            "--color",
            "never",
            "--ephemeral",
        ],
    )
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
    let health = probe_adapter_health(name, config);
    AdapterProbe {
        name: health.name,
        command: health.command,
        available: health.installed,
        version: health.version,
    }
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
        assert_eq!(
            adapters["codex"].args,
            vec![
                "exec".to_string(),
                "--sandbox".to_string(),
                "workspace-write".to_string(),
                "--color".to_string(),
                "never".to_string(),
                "--ephemeral".to_string()
            ]
        );
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
