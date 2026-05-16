use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::adapter::{AuthStatus, probe_adapter_health};
use crate::config::TuiConfig;
use crate::interactive::WorkflowUpdate;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelLayout {
    pub sessions: Rect,
    pub transcript: Rect,
    pub inspector: Rect,
    pub command: Rect,
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
    pub layout: Option<PanelLayout>,
    pending_inputs: VecDeque<String>,
}

impl PanelLayout {
    pub fn from_area(area: Rect) -> Self {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(3)])
            .split(area);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),
                Constraint::Min(50),
                Constraint::Length(34),
            ])
            .split(root[0]);
        Self {
            sessions: columns[0],
            transcript: columns[1],
            inspector: columns[2],
            command: root[1],
        }
    }

    pub fn panel_at(&self, column: u16, row: u16) -> Option<Panel> {
        if contains(self.sessions, column, row) {
            Some(Panel::Sessions)
        } else if contains(self.transcript, column, row) {
            Some(Panel::Transcript)
        } else if contains(self.inspector, column, row) {
            Some(Panel::Inspector)
        } else if contains(self.command, column, row) {
            Some(Panel::Command)
        } else {
            None
        }
    }

    pub fn agent_index_at(&self, row: u16) -> Option<usize> {
        if row <= self.sessions.y {
            return None;
        }
        Some(row.saturating_sub(self.sessions.y + 1) as usize)
    }
}

impl AppState {
    pub fn new(workspace: PathBuf, config: TuiConfig) -> Self {
        let orchestrator = Orchestrator::new(workspace.clone(), config.clone());
        let bridge = ArchitectMcpBridge::new(workspace.clone(), config.clone());
        let workflow = orchestrator.new_app_workflow("new app idea");
        let agents = config
            .adapters
            .iter()
            .map(|(name, adapter)| {
                let health = probe_adapter_health(name, adapter);
                AgentNode {
                    name: name.clone(),
                    status: agent_status(&health),
                    pinned: name == &config.agents.default_adapter,
                }
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
            layout: None,
            pending_inputs: VecDeque::new(),
        }
    }

    pub fn set_layout(&mut self, layout: PanelLayout) {
        self.layout = Some(layout);
    }

    pub fn panel_at(&self, column: u16, row: u16) -> Option<Panel> {
        self.layout.and_then(|layout| layout.panel_at(column, row))
    }

    pub fn agent_index_at(&self, row: u16) -> Option<usize> {
        self.layout.and_then(|layout| layout.agent_index_at(row))
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
        let input = self.input.trim().to_string();
        self.transcript.push(format!("user: {input}"));
        self.pending_inputs.push_back(input);
        self.input.clear();
        self.mark_dirty(Panel::Transcript);
        self.mark_dirty(Panel::Command);
    }

    pub fn pop_pending_input(&mut self) -> Option<String> {
        self.pending_inputs.pop_front()
    }

    pub fn apply_workflow_update(&mut self, update: WorkflowUpdate) {
        self.transcript.extend(update.transcript);
        if !update.inspector.is_empty() {
            self.inspector = update.inspector;
        }
        if let Some(session) = update.session {
            self.workflow.idea = session.prompt;
        }
        self.mark_dirty(Panel::Transcript);
        self.mark_dirty(Panel::Inspector);
    }
}

fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn agent_status(health: &crate::adapter::AdapterHealth) -> String {
    if !health.installed {
        return "unavailable".to_string();
    }
    if health.ready {
        return "ready".to_string();
    }
    match health.auth_status {
        AuthStatus::NeedsLogin => "needs login".to_string(),
        AuthStatus::Unknown => "auth unknown".to_string(),
        AuthStatus::Authenticated | AuthStatus::NotApplicable => "ready".to_string(),
    }
}
