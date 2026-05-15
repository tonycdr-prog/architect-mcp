use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use serde::Serialize;
use tokio::io::{AsyncWriteExt, stdout};
use uuid::Uuid;

use crate::adapter::{AgentEvent, run_adapter_pty};
use crate::config::TuiConfig;
use crate::mcp::ArchitectMcpBridge;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum WorkflowMode {
    NewApp,
    CodingTask,
    Automation,
    Arena,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum WorktreePolicy {
    Isolated,
    SharedRequiresConfirmation,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorkflowStep {
    pub name: String,
    pub gate: String,
    pub approval_required: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AppBuildWorkflow {
    pub mode: WorkflowMode,
    pub idea: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ArenaCandidate {
    pub adapter: String,
    pub worktree: PathBuf,
    pub score: Option<u8>,
    pub review: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HeadlessRunOptions {
    pub prompt: String,
    pub adapter: String,
    pub jsonl: bool,
    pub concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct Orchestrator {
    workspace: PathBuf,
    config: TuiConfig,
    bridge: ArchitectMcpBridge,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum JsonlEvent<'a> {
    SessionStarted {
        session_id: &'a str,
        workspace: &'a str,
        adapter: &'a str,
        concurrency: usize,
    },
    McpCall {
        name: &'a str,
        purpose: &'a str,
    },
    AgentEvent {
        event: &'a AgentEvent,
    },
    WorkflowStep {
        name: &'a str,
        gate: &'a str,
        approval_required: bool,
    },
    Complete {
        session_id: &'a str,
        status: &'a str,
    },
}

impl Orchestrator {
    pub fn new(workspace: impl Into<PathBuf>, config: TuiConfig) -> Self {
        let workspace = workspace.into();
        let bridge = ArchitectMcpBridge::new(workspace.clone(), config.clone());
        Self {
            workspace,
            config,
            bridge,
        }
    }

    pub fn new_app_workflow(&self, idea: impl Into<String>) -> AppBuildWorkflow {
        let idea = idea.into();
        AppBuildWorkflow {
            mode: WorkflowMode::NewApp,
            idea,
            steps: vec![
                step("intake", "grill_me", true),
                step("contract", "create_pre_edit_contract", true),
                step("plan", "review_build_plan", true),
                step("file plan", "review_proposed_file_plan", true),
                step("agent slice", "adapter execution", true),
                step(
                    "implementation review",
                    "review_implementation_against_contract",
                    true,
                ),
                step("verification", "tests and checks", true),
                step("session review", "review_agent_session", true),
            ],
        }
    }

    pub fn worktree_policy(&self) -> WorktreePolicy {
        if self.config.agents.worktree_isolation {
            WorktreePolicy::Isolated
        } else {
            WorktreePolicy::SharedRequiresConfirmation
        }
    }

    pub fn worktree_path_for_agent(&self, session_id: &str, adapter: &str) -> PathBuf {
        self.workspace
            .join(".architect-mcp")
            .join("worktrees")
            .join(session_id)
            .join(adapter)
    }

    pub fn arena_candidates(&self, session_id: &str, adapters: &[String]) -> Vec<ArenaCandidate> {
        adapters
            .iter()
            .map(|adapter| ArenaCandidate {
                adapter: adapter.clone(),
                worktree: self.worktree_path_for_agent(session_id, adapter),
                score: None,
                review: None,
            })
            .collect()
    }

    pub async fn run_headless(&self, options: HeadlessRunOptions) -> Result<()> {
        let session_id = Uuid::new_v4().to_string();
        let mut output = stdout();
        let workspace_display = self.workspace.display().to_string();
        let calls = self.bridge.work_gate_plan(&options.prompt);
        let workflow = self.new_app_workflow(&options.prompt);

        emit(
            &mut output,
            options.jsonl,
            &JsonlEvent::SessionStarted {
                session_id: &session_id,
                workspace: &workspace_display,
                adapter: &options.adapter,
                concurrency: options.concurrency,
            },
        )
        .await?;

        for call in &calls {
            emit(
                &mut output,
                options.jsonl,
                &JsonlEvent::McpCall {
                    name: &call.name,
                    purpose: &call.purpose,
                },
            )
            .await?;
        }

        for step in &workflow.steps {
            emit(
                &mut output,
                options.jsonl,
                &JsonlEvent::WorkflowStep {
                    name: &step.name,
                    gate: &step.gate,
                    approval_required: step.approval_required,
                },
            )
            .await?;
        }

        if let Some(adapter) = self.config.adapters.get(&options.adapter) {
            if adapter.available == Some(false) {
                let event = AgentEvent::Crashed {
                    message: format!("adapter '{}' is unavailable", options.adapter),
                };
                emit(
                    &mut output,
                    options.jsonl,
                    &JsonlEvent::AgentEvent { event: &event },
                )
                .await?;
            } else {
                let events = run_adapter_pty(
                    adapter,
                    crate::adapter::PtyRunOptions {
                        adapter_name: options.adapter.clone(),
                        prompt: options.prompt.clone(),
                        timeout: Duration::from_secs(self.config.agents.default_timeout_seconds),
                    },
                )
                .unwrap_or_else(|error| {
                    vec![AgentEvent::Crashed {
                        message: error.to_string(),
                    }]
                });
                for event in &events {
                    emit(
                        &mut output,
                        options.jsonl,
                        &JsonlEvent::AgentEvent { event },
                    )
                    .await?;
                }
            }
        }

        emit(
            &mut output,
            options.jsonl,
            &JsonlEvent::Complete {
                session_id: &session_id,
                status: "review_required",
            },
        )
        .await?;
        Ok(())
    }
}

fn step(name: &str, gate: &str, approval_required: bool) -> WorkflowStep {
    WorkflowStep {
        name: name.to_string(),
        gate: gate.to_string(),
        approval_required,
    }
}

async fn emit<W: AsyncWriteExt + Unpin, T: Serialize>(
    output: &mut W,
    jsonl: bool,
    event: &T,
) -> Result<()> {
    if jsonl {
        let line = serde_json::to_string(event)?;
        output.write_all(line.as_bytes()).await?;
        output.write_all(b"\n").await?;
    } else {
        let pretty = serde_json::to_string_pretty(event)?;
        output.write_all(pretty.as_bytes()).await?;
        output.write_all(b"\n").await?;
    }
    Ok(())
}

pub fn workspace_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string()
}
