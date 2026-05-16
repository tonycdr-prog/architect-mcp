use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::TuiConfig;
use crate::mcp::ArchitectMcpBridge;

pub use crate::headless::HeadlessRunOptions;

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
pub struct Orchestrator {
    pub(crate) workspace: PathBuf,
    pub(crate) config: TuiConfig,
    pub(crate) bridge: ArchitectMcpBridge,
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
        AppBuildWorkflow {
            mode: WorkflowMode::NewApp,
            idea: idea.into(),
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
}

fn step(name: &str, gate: &str, approval_required: bool) -> WorkflowStep {
    WorkflowStep {
        name: name.to_string(),
        gate: gate.to_string(),
        approval_required,
    }
}

pub fn workspace_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string()
}
