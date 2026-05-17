use anyhow::Result;
use architect_tui::acp::run_acp_stdio;
use architect_tui::adapter::print_adapter_table;
use architect_tui::cli::{Cli, Commands};
use architect_tui::config::{ConfigCommand, ConfigPaths, TuiConfig};
use architect_tui::evidence_index::{EvidenceIndexOptions, run_evidence_index};
use architect_tui::foundry_smoke::{FoundrySmokeOptions, run_foundry_smoke};
use architect_tui::governance_audit::{GovernanceAuditOptions, run_governance_audit};
use architect_tui::issue_terminal_evidence::{
    IssueTerminalEvidenceOptions, run_issue_terminal_evidence,
};
use architect_tui::launch_judge::{LaunchJudgeOptions, run_launch_judge};
use architect_tui::launch_readiness::{LaunchReadinessOptions, run_launch_readiness};
use architect_tui::launch_stack::{LaunchStackOptions, run_launch_stack};
use architect_tui::orchestrator::{HeadlessRunOptions, Orchestrator};
use architect_tui::promotion_smoke::{PromotionSmokeOptions, run_promotion_smoke};
use architect_tui::smoke::{SmokeOptions, run_smoke};
use architect_tui::terminal_evidence::{TerminalEvidenceOptions, run_terminal_evidence};
use architect_tui::ui::run_interactive;
use architect_tui::walkthrough::{WalkthroughOptions, run_walkthrough};
use clap::Parser;
use tracing_subscriber::EnvFilter;

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
        Some(Commands::TerminalEvidence {
            json,
            prompt,
            skip_gate,
            platform,
            source,
            notes,
        }) => {
            run_terminal_evidence(
                workspace,
                config,
                TerminalEvidenceOptions {
                    json,
                    prompt,
                    skip_gate,
                    platform,
                    source,
                    notes,
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
        Some(Commands::GovernanceAudit {
            json,
            public_summary,
            skip_mcp,
            max_files,
        }) => {
            run_governance_audit(
                workspace,
                config,
                GovernanceAuditOptions {
                    json,
                    public_summary,
                    skip_mcp,
                    max_files,
                },
            )
            .await?;
        }
        Some(Commands::LaunchJudge {
            json,
            public_summary,
            skip_mcp,
            skip_smoke,
            run_release_check,
            require_clean_git,
            max_files,
            terminal_evidence,
        }) => {
            run_launch_judge(
                workspace,
                config,
                LaunchJudgeOptions {
                    json,
                    public_summary,
                    skip_mcp,
                    skip_smoke,
                    run_release_check,
                    require_clean_git,
                    max_files,
                    terminal_evidence,
                },
            )
            .await?;
        }
        Some(Commands::LaunchStack {
            json,
            repo,
            stack_from_pr,
            prs,
            blockers,
            waived_blockers,
        }) => run_launch_stack(
            &workspace,
            LaunchStackOptions {
                json,
                repo,
                stack_from_pr,
                prs,
                blockers,
                waived_blockers,
            },
        )?,
        Some(Commands::LaunchReadiness {
            json,
            public_summary,
            repo,
            stack_from_pr,
            prs,
            blockers,
            waived_blockers,
            terminal_evidence_issue,
            terminal_evidence_waivers,
        }) => run_launch_readiness(
            &workspace,
            LaunchReadinessOptions {
                json,
                public_summary,
                repo,
                stack_from_pr,
                prs,
                blockers,
                waived_blockers,
                terminal_evidence_issue,
                terminal_evidence_waivers,
            },
        )?,
        Some(Commands::EvidenceIndex {
            json,
            markdown,
            repo,
            stack_from_pr,
            prs,
            blockers,
            waived_blockers,
            terminal_evidence_issue,
            terminal_evidence_waivers,
            skip_mcp,
            max_files,
        }) => {
            run_evidence_index(
                workspace,
                config,
                EvidenceIndexOptions {
                    json,
                    markdown,
                    repo,
                    stack_from_pr,
                    prs,
                    blockers,
                    waived_blockers,
                    terminal_evidence_issue,
                    terminal_evidence_waivers,
                    skip_mcp,
                    max_files,
                },
            )
            .await?
        }
        Some(Commands::CollectTerminalEvidence { json, repo, issue }) => {
            run_issue_terminal_evidence(
                &workspace,
                IssueTerminalEvidenceOptions { json, repo, issue },
            )?
        }
        None => run_interactive(workspace, config).await?,
    }

    Ok(())
}
