use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Subcommand;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::adapter::{AdapterConfig, default_adapters};
use crate::config_template::default_repo_config;

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub workspace: PathBuf,
    pub user: PathBuf,
    pub repo: PathBuf,
    pub explicit: Option<PathBuf>,
}

impl ConfigPaths {
    pub fn discover(workspace: &Path, explicit: Option<PathBuf>) -> Result<Self> {
        let project_dirs = ProjectDirs::from("dev", "architect-mcp", "architect-mcp")
            .context("could not resolve user config directory")?;
        Ok(Self {
            workspace: workspace.to_path_buf(),
            user: project_dirs.config_dir().join("tui.toml"),
            repo: workspace.join(".architect-mcp").join("tui.toml"),
            explicit,
        })
    }

    fn ordered_sources(&self) -> Vec<PathBuf> {
        let mut sources = vec![self.user.clone(), self.repo.clone()];
        if let Some(explicit) = &self.explicit {
            sources.push(explicit.clone());
        }
        sources
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TuiConfig {
    pub architect_mcp: ArchitectMcpConfig,
    pub ui: UiConfig,
    pub agents: AgentOrchestrationConfig,
    pub adapters: BTreeMap<String, AdapterConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ArchitectMcpConfig {
    pub command: Option<String>,
    pub args: Vec<String>,
    pub tool_surface: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiConfig {
    pub mouse: bool,
    pub dirty_render: bool,
    pub tick_millis: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentOrchestrationConfig {
    pub default_adapter: String,
    pub worktree_isolation: bool,
    pub shared_workspace_requires_confirmation: bool,
    pub default_timeout_seconds: u64,
    pub approval_policy: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    /// Write .architect-mcp/tui.toml for this workspace.
    Init {
        #[arg(long)]
        force: bool,
    },
    /// Validate merged user and repo config.
    Doctor,
    /// Probe configured adapters.
    Adapters,
}

impl Default for ArchitectMcpConfig {
    fn default() -> Self {
        Self {
            command: None,
            args: Vec::new(),
            tool_surface: "advanced".to_string(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            mouse: true,
            dirty_render: true,
            tick_millis: 33,
        }
    }
}

impl Default for AgentOrchestrationConfig {
    fn default() -> Self {
        Self {
            default_adapter: "codex".to_string(),
            worktree_isolation: true,
            shared_workspace_requires_confirmation: true,
            default_timeout_seconds: 1800,
            approval_policy: "manual".to_string(),
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            architect_mcp: ArchitectMcpConfig::default(),
            ui: UiConfig::default(),
            agents: AgentOrchestrationConfig::default(),
            adapters: default_adapters(),
        }
    }
}

impl TuiConfig {
    pub fn load(paths: &ConfigPaths) -> Result<Self> {
        let mut config = TuiConfig::default();
        for path in paths.ordered_sources() {
            if !path.exists() {
                continue;
            }
            let loaded = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config {}", path.display()))?;
            let parsed: PartialTuiConfig = toml::from_str(&loaded)
                .with_context(|| format!("failed to parse config {}", path.display()))?;
            config.merge(parsed, &path, &paths.workspace)?;
        }
        config.validate()?;
        Ok(config)
    }

    pub fn init(paths: &ConfigPaths, force: bool) -> Result<()> {
        if paths.repo.exists() && !force {
            anyhow::bail!(
                "{} already exists; pass --force to overwrite",
                paths.repo.display()
            );
        }
        if let Some(parent) = paths.repo.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&paths.repo, default_repo_config())?;
        println!("wrote {}", paths.repo.display());
        Ok(())
    }

    pub fn doctor(paths: &ConfigPaths, config: &TuiConfig) -> Result<()> {
        config.validate()?;
        println!("workspace: {}", paths.workspace.display());
        println!("user config: {}", paths.user.display());
        println!("repo config: {}", paths.repo.display());
        println!("tool surface: {}", config.architect_mcp.tool_surface);
        println!("default adapter: {}", config.agents.default_adapter);
        println!("adapters: {}", config.adapters.len());
        Ok(())
    }

    fn merge(&mut self, partial: PartialTuiConfig, source: &Path, workspace: &Path) -> Result<()> {
        if let Some(architect_mcp) = partial.architect_mcp {
            if let Some(command) = architect_mcp.command {
                self.architect_mcp.command = Some(command);
            }
            if let Some(args) = architect_mcp.args {
                self.architect_mcp.args = args;
            }
            if let Some(tool_surface) = architect_mcp.tool_surface {
                self.architect_mcp.tool_surface = tool_surface;
            }
        }
        if let Some(ui) = partial.ui {
            if let Some(mouse) = ui.mouse {
                self.ui.mouse = mouse;
            }
            if let Some(dirty_render) = ui.dirty_render {
                self.ui.dirty_render = dirty_render;
            }
            if let Some(tick_millis) = ui.tick_millis {
                self.ui.tick_millis = tick_millis;
            }
        }
        if let Some(agents) = partial.agents {
            if let Some(default_adapter) = agents.default_adapter {
                self.agents.default_adapter = default_adapter;
            }
            if let Some(worktree_isolation) = agents.worktree_isolation {
                self.agents.worktree_isolation = worktree_isolation;
            }
            if let Some(shared_workspace_requires_confirmation) =
                agents.shared_workspace_requires_confirmation
            {
                self.agents.shared_workspace_requires_confirmation =
                    shared_workspace_requires_confirmation;
            }
            if let Some(default_timeout_seconds) = agents.default_timeout_seconds {
                self.agents.default_timeout_seconds = default_timeout_seconds;
            }
            if let Some(approval_policy) = agents.approval_policy {
                self.agents.approval_policy = approval_policy;
            }
        }
        if let Some(adapters) = partial.adapters {
            ensure_repo_adapter_scope(source, workspace)?;
            for (name, adapter) in adapters {
                self.adapters.insert(name, adapter);
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.architect_mcp.tool_surface != "advanced" {
            anyhow::bail!("architect-mcp-tui requires ARCHITECT_MCP_TOOL_SURFACE=advanced");
        }
        if !self.adapters.contains_key(&self.agents.default_adapter) {
            anyhow::bail!(
                "default adapter '{}' is not configured",
                self.agents.default_adapter
            );
        }
        for (name, adapter) in &self.adapters {
            adapter.validate(name)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PartialTuiConfig {
    architect_mcp: Option<PartialArchitectMcpConfig>,
    ui: Option<PartialUiConfig>,
    agents: Option<PartialAgentConfig>,
    adapters: Option<BTreeMap<String, AdapterConfig>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PartialArchitectMcpConfig {
    command: Option<String>,
    args: Option<Vec<String>>,
    tool_surface: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PartialUiConfig {
    mouse: Option<bool>,
    dirty_render: Option<bool>,
    tick_millis: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PartialAgentConfig {
    default_adapter: Option<String>,
    worktree_isolation: Option<bool>,
    shared_workspace_requires_confirmation: Option<bool>,
    default_timeout_seconds: Option<u64>,
    approval_policy: Option<String>,
}

fn ensure_repo_adapter_scope(source: &Path, workspace: &Path) -> Result<()> {
    let canonical_source = source
        .canonicalize()
        .unwrap_or_else(|_| source.to_path_buf());
    let canonical_workspace = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    if canonical_source.starts_with(&canonical_workspace) || source.ends_with("tui.toml") {
        return Ok(());
    }
    anyhow::bail!(
        "adapter overrides must come from user config or the current workspace, got {}",
        source.display()
    )
}
