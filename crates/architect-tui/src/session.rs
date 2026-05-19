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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Promoted,
    Override,
}

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
    pub worktree: Option<PathBuf>,
    pub diff_stat: Option<String>,
    pub changed_files: Vec<Value>,
    #[serde(default)]
    pub adapter_crashed: bool,
    #[serde(default)]
    pub arena_candidates: Vec<ArenaCandidateRecord>,
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
            worktree: None,
            diff_stat: None,
            changed_files: Vec::new(),
            adapter_crashed: false,
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

    pub fn approve(&mut self, reason: impl Into<String>) {
        self.approval_status = ApprovalStatus::Approved;
        self.approval_reason = Some(reason.into());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_store_round_trips_without_secrets() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = SessionStore::for_workspace(temp.path());
        let mut session = TuiSession::new("build a recipe app", "codex");
        session.set_answer("users", "home cooks");
        let path = store.save(&mut session).expect("save");
        assert!(path.exists());

        let loaded = store.load(&session.id).expect("load");
        assert_eq!(loaded.prompt, "build a recipe app");
        assert_eq!(loaded.brief["users"], "home cooks");
    }

    #[test]
    fn approval_state_controls_promotion_readiness() {
        let mut session = TuiSession::new("build a recipe app", "codex");
        assert!(!session.can_promote());
        session.approve("review gates passed");
        assert!(session.can_promote());
        session.mark_promoted();
        assert_eq!(session.approval_status, ApprovalStatus::Promoted);
    }

    #[test]
    fn answers_shape_project_brief_for_live_grill() {
        let mut session = TuiSession::new("build controlled TUI", "codex");
        session.set_answer("users", "maintainers need to control agents");
        session.set_answer("coreFlows", "grill; review plan; promote");
        session.set_answer(
            "verification",
            "cargo test --workspace; npm run release:check",
        );
        session.set_answer("stack", "frontend=Rust Ratatui; backend=TypeScript MCP");
        session.set_answer(
            "repoLayout",
            "tui=crates/architect-tui/src; tests=crates/architect-tui/tests",
        );

        assert_eq!(session.brief["users"], "maintainers need to control agents");
        assert_eq!(
            session.brief["coreFlows"].as_array().expect("flows").len(),
            3
        );
        assert_eq!(
            session.brief["verification"]
                .as_array()
                .expect("verification")
                .len(),
            2
        );
        assert_eq!(session.brief["stack"]["backend"], "TypeScript MCP");
        assert_eq!(
            session.brief["repoLayout"]["pathMap"]["tui"][0],
            "crates/architect-tui/src"
        );
    }
}
