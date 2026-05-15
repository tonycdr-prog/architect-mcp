use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::config::TuiConfig;
use crate::mcp::ArchitectMcpBridge;
use crate::orchestrator::{AppBuildWorkflow, Orchestrator, workspace_name};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Panel {
    Sessions,
    Transcript,
    Inspector,
    Command,
}

#[derive(Debug, Clone)]
pub struct AgentNode {
    pub name: String,
    pub status: String,
    pub pinned: bool,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub workspace: PathBuf,
    pub title: String,
    pub active_panel: Panel,
    pub agents: Vec<AgentNode>,
    pub transcript: Vec<String>,
    pub inspector: Vec<String>,
    pub input: String,
    pub dirty: BTreeSet<Panel>,
    pub workflow: AppBuildWorkflow,
}

impl AppState {
    pub fn new(workspace: PathBuf, config: TuiConfig) -> Self {
        let orchestrator = Orchestrator::new(workspace.clone(), config.clone());
        let bridge = ArchitectMcpBridge::new(workspace.clone(), config.clone());
        let workflow = orchestrator.new_app_workflow("new app idea");
        let agents = config
            .adapters
            .keys()
            .map(|name| AgentNode {
                name: name.clone(),
                status: "ready".to_string(),
                pinned: name == &config.agents.default_adapter,
            })
            .collect();
        let transcript = bridge
            .work_gate_plan("new app idea")
            .into_iter()
            .map(|call| format!("{}: {}", call.name, call.purpose))
            .collect();
        Self {
            title: format!("architect-mcp-tui - {}", workspace_name(&workspace)),
            workspace,
            active_panel: Panel::Command,
            agents,
            transcript,
            inspector: vec![
                "mode: new app".to_string(),
                "surface: advanced".to_string(),
                "approval: manual".to_string(),
            ],
            input: String::new(),
            dirty: BTreeSet::from([
                Panel::Sessions,
                Panel::Transcript,
                Panel::Inspector,
                Panel::Command,
            ]),
            workflow,
        }
    }

    pub fn mark_dirty(&mut self, panel: Panel) {
        self.dirty.insert(panel);
    }

    pub fn take_dirty(&mut self) -> BTreeSet<Panel> {
        std::mem::take(&mut self.dirty)
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Sessions => Panel::Transcript,
            Panel::Transcript => Panel::Inspector,
            Panel::Inspector => Panel::Command,
            Panel::Command => Panel::Sessions,
        };
        self.mark_dirty(self.active_panel);
    }

    pub fn pin_agent_at(&mut self, index: usize) {
        if let Some(agent) = self.agents.get_mut(index) {
            agent.pinned = !agent.pinned;
            agent.status = if agent.pinned { "pinned" } else { "ready" }.to_string();
            self.mark_dirty(Panel::Sessions);
        }
    }

    pub fn append_input(&mut self, ch: char) {
        self.input.push(ch);
        self.mark_dirty(Panel::Command);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        self.mark_dirty(Panel::Command);
    }

    pub fn submit_input(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        self.transcript.push(format!("user: {}", self.input.trim()));
        self.input.clear();
        self.mark_dirty(Panel::Transcript);
        self.mark_dirty(Panel::Command);
    }
}
