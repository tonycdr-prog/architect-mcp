use std::path::PathBuf;

use anyhow::Result;
use architect_tui::acp::run_acp_stdio;
use architect_tui::adapter::print_adapter_table;
use architect_tui::config::{ConfigCommand, ConfigPaths, TuiConfig};
use architect_tui::foundry_smoke::{FoundrySmokeOptions, run_foundry_smoke};
use architect_tui::orchestrator::{HeadlessRunOptions, Orchestrator};
use architect_tui::promotion_smoke::{PromotionSmokeOptions, run_promotion_smoke};
use architect_tui::smoke::{SmokeOptions, run_smoke};
use architect_tui::ui::run_interactive;
use architect_tui::walkthrough::{WalkthroughOptions, run_walkthrough};
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
    /// Run a secret-safe terminal QA smoke report.
    Smoke {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = SmokeOptions::DEFAULT_PROMPT)]
        prompt: String,
        #[arg(long)]
        skip_gate: bool,
    },
    /// Run a scripted interactive command-palette walkthrough in a throwaway workspace.
    Walkthrough {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        keep_workspace: bool,
    },
    /// Run a local-only real-adapter promotion smoke in a disposable git workspace.
    PromotionSmoke {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "codex")]
        adapter: String,
        #[arg(long)]
        keep_workspace: bool,
        #[arg(long, default_value_t = 600)]
        timeout_seconds: u64,
    },
    /// Run a repo-foundry smoke; live GitHub creation requires explicit confirmation.
    FoundrySmoke {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        owner: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        confirm_private_repo_mutation: bool,
        #[arg(long)]
        keep_workspace: bool,
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
        Some(Commands::Smoke {
            json,
            prompt,
            skip_gate,
        }) => {
            run_smoke(
                workspace,
                config,
                SmokeOptions {
                    json,
                    prompt,
                    skip_gate,
                },
            )
            .await?;
        }
        Some(Commands::Walkthrough {
            json,
            keep_workspace,
        }) => {
            run_walkthrough(
                workspace,
                config,
                WalkthroughOptions {
                    json,
                    keep_workspace,
                },
            )
            .await?;
        }
        Some(Commands::PromotionSmoke {
            json,
            adapter,
            keep_workspace,
            timeout_seconds,
        }) => {
            run_promotion_smoke(
                workspace,
                config,
                PromotionSmokeOptions {
                    json,
                    adapter,
                    keep_workspace,
                    timeout_seconds,
                },
            )
            .await?;
        }
        Some(Commands::FoundrySmoke {
            json,
            owner,
            repo,
            execute,
            confirm_private_repo_mutation,
            keep_workspace,
        }) => {
            run_foundry_smoke(
                workspace,
                config,
                FoundrySmokeOptions {
                    json,
                    owner,
                    repo,
                    execute,
                    confirm_private_repo_mutation,
                    keep_workspace,
                },
            )
            .await?;
        }
        None => run_interactive(workspace, config).await?,
    }

    Ok(())
}
