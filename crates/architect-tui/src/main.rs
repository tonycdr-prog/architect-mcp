use std::path::PathBuf;

use anyhow::Result;
use architect_tui::acp::run_acp_stdio;
use architect_tui::adapter::print_adapter_table;
use architect_tui::config::{ConfigCommand, ConfigPaths, TuiConfig};
use architect_tui::orchestrator::{HeadlessRunOptions, Orchestrator};
use architect_tui::ui::run_interactive;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "architect-mcp-tui")]
#[command(about = "Ratatui agent client for architect-mcp work-gate workflows.")]
struct Cli {
    #[arg(long, global = true, env = "ARCHITECT_MCP_TUI_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long, global = true, env = "ARCHITECT_MCP_WORKSPACE")]
    workspace: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a scriptable headless prompt through the work gate and one adapter.
    Run {
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value = "codex")]
        adapter: String,
        #[arg(long)]
        jsonl: bool,
        #[arg(long, default_value_t = 1)]
        concurrency: usize,
        #[arg(long)]
        execute: bool,
    },
    /// Serve an Agent Client Protocol endpoint over stdio.
    Acp {
        #[arg(long)]
        stdio: bool,
    },
    /// Manage architect-mcp-tui configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let cli = Cli::parse();
    let workspace = cli.workspace.unwrap_or(std::env::current_dir()?);
    let paths = ConfigPaths::discover(&workspace, cli.config)?;
    let config = TuiConfig::load(&paths)?;

    match cli.command {
        Some(Commands::Run {
            prompt,
            adapter,
            jsonl,
            concurrency,
            execute,
        }) => {
            let orchestrator = Orchestrator::new(workspace, config);
            orchestrator
                .run_headless(HeadlessRunOptions {
                    prompt,
                    adapter,
                    jsonl,
                    concurrency,
                    execute,
                })
                .await?;
        }
        Some(Commands::Acp { stdio }) => {
            if !stdio {
                anyhow::bail!("ACP mode currently requires --stdio");
            }
            run_acp_stdio(config).await?;
        }
        Some(Commands::Config { command }) => match command {
            ConfigCommand::Init { force } => TuiConfig::init(&paths, force)?,
            ConfigCommand::Doctor => TuiConfig::doctor(&paths, &config)?,
            ConfigCommand::Adapters { json } => print_adapter_table(&config.adapters, json)?,
        },
        None => run_interactive(workspace, config).await?,
    }

    Ok(())
}
