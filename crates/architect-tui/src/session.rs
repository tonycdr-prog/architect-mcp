use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::arena::ArenaCandidateRecord;
use crate::brief::{apply_brief_answer, brief_from_prompt};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Created,
    IntakeBlocked,
    IntakeReady,
    ContractReady,
    PlanReviewed,
    FilePlanReviewed,
    AdapterRunning,
    ReviewRequired,
    Complete,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
    Promoted,
    Override,
}

fn default_approval_status() -> ApprovalStatus {
    ApprovalStatus::Pending
}

const ADAPTER_RUN_GATES: &[&str] = &[
    "review_implementation_against_contract",
    "review_repo_structure",
    "review_agent_final_response",
    "review_agent_session",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuiSession {
    pub id: String,
    pub prompt: String,
    pub adapter: String,
    pub phase: SessionPhase,
    pub brief: Value,
    pub gates: BTreeMap<String, Value>,
    pub verification: BTreeMap<String, String>,
    #[serde(default)]
    pub required_verification: Vec<String>,
    #[serde(default)]
    pub final_response: Option<String>,
    pub worktree: Option<PathBuf>,
    pub diff_stat: Option<String>,
    pub changed_files: Vec<Value>,
    #[serde(default)]
    pub execution_approved: bool,
    #[serde(default)]
    pub execution_approval_reason: Option<String>,
    #[serde(default)]
    pub adapter_crashed: bool,
    #[serde(default)]
    pub adapter_run_issues: Vec<String>,
    #[serde(default)]
    pub arena_candidates: Vec<ArenaCandidateRecord>,
    #[serde(default = "default_approval_status")]
    pub approval_status: ApprovalStatus,
    pub approval_reason: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl TuiSession {
    pub fn new(prompt: impl Into<String>, adapter: impl Into<String>) -> Self {
        let prompt = prompt.into();
        let now = unix_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            prompt: prompt.clone(),
            adapter: adapter.into(),
            phase: SessionPhase::Created,
            brief: brief_from_prompt(&prompt),
            gates: BTreeMap::new(),
            verification: BTreeMap::new(),
            required_verification: Vec::new(),
            final_response: None,
            worktree: None,
            diff_stat: None,
            changed_files: Vec::new(),
            execution_approved: false,
            execution_approval_reason: None,
            adapter_crashed: false,
            adapter_run_issues: Vec::new(),
            arena_candidates: Vec::new(),
            approval_status: ApprovalStatus::Pending,
            approval_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_answer(&mut self, key: &str, value: &str) {
        apply_brief_answer(&mut self.brief, key, value);
        self.updated_at = unix_timestamp();
    }

    pub fn set_gate(&mut self, name: &str, value: Value) {
        self.gates.insert(name.to_string(), value);
        self.updated_at = unix_timestamp();
    }

    pub fn set_required_verification(&mut self, checks: Vec<String>) {
        let mut checks = checks
            .into_iter()
            .map(|check| check.trim().to_string())
            .filter(|check| !check.is_empty())
            .collect::<Vec<_>>();
        checks.sort();
        checks.dedup();
        self.required_verification = checks;
        self.updated_at = unix_timestamp();
    }

    pub fn set_final_response(&mut self, response: impl Into<String>) {
        self.final_response = Some(response.into());
        self.updated_at = unix_timestamp();
    }

    pub fn approve(&mut self, reason: impl Into<String>) {
        self.approval_status = ApprovalStatus::Approved;
        self.approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn approve_execution(&mut self, reason: impl Into<String>) {
        self.execution_approved = true;
        self.execution_approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn clear_execution_approval(&mut self) {
        self.execution_approved = false;
        self.execution_approval_reason = None;
        self.updated_at = unix_timestamp();
    }

    pub fn record_adapter_run_issue(&mut self, issue: impl Into<String>) {
        let issue = issue.into();
        if !self
            .adapter_run_issues
            .iter()
            .any(|existing| existing == &issue)
        {
            self.adapter_run_issues.push(issue);
        }
        self.adapter_crashed = true;
        self.updated_at = unix_timestamp();
    }

    pub fn clear_adapter_run_evidence(&mut self) {
        self.worktree = None;
        self.diff_stat = None;
        self.changed_files.clear();
        self.verification.clear();
        self.final_response = None;
        self.adapter_crashed = false;
        self.adapter_run_issues.clear();
        self.approval_status = ApprovalStatus::Pending;
        self.approval_reason = None;
        for gate in ADAPTER_RUN_GATES {
            self.gates.remove(*gate);
        }
        self.updated_at = unix_timestamp();
    }

    pub fn clear_arena_candidates(&mut self) {
        self.arena_candidates.clear();
        self.updated_at = unix_timestamp();
    }

    pub fn reject(&mut self, reason: impl Into<String>) {
        self.approval_status = ApprovalStatus::Rejected;
        self.approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn override_approval(&mut self, reason: impl Into<String>) {
        self.approval_status = ApprovalStatus::Override;
        self.approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn mark_promoted(&mut self) {
        self.approval_status = ApprovalStatus::Promoted;
        self.updated_at = unix_timestamp();
    }

    pub fn can_promote(&self) -> bool {
        matches!(
            self.approval_status,
            ApprovalStatus::Approved | ApprovalStatus::Override
        )
    }
}

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    pub fn for_workspace(workspace: &Path) -> Self {
        Self {
            root: workspace
                .join(".architect-mcp")
                .join("tui")
                .join("sessions"),
        }
    }

    pub fn save(&self, session: &mut TuiSession) -> Result<PathBuf> {
        session.updated_at = unix_timestamp();
        fs::create_dir_all(&self.root)?;
        let path = self.path_for(&session.id);
        let json = serde_json::to_string_pretty(session)?;
        fs::write(&path, json).with_context(|| format!("failed to save {}", path.display()))?;
        Ok(path)
    }

    pub fn load(&self, id: &str) -> Result<TuiSession> {
        let path = self.path_for(id);
        let body = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_str(&body).with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn path_for(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
