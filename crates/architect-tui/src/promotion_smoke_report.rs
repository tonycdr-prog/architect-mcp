use std::path::PathBuf;

use serde::Serialize;

use crate::session::{ApprovalStatus, SessionPhase, TuiSession};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromotionSmokeStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionSmokeReport {
    pub schema_version: u8,
    pub status: PromotionSmokeStatus,
    pub adapter: String,
    pub workspace: PathBuf,
    pub commands: Vec<PromotionSmokeCommandReport>,
    pub verification: Vec<PromotionSmokeVerificationReport>,
    pub promoted_files: Vec<String>,
    pub final_phase: Option<SessionPhase>,
    pub final_approval: Option<ApprovalStatus>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionSmokeCommandReport {
    pub command: String,
    pub ok: bool,
    pub phase: Option<SessionPhase>,
    pub approval: Option<ApprovalStatus>,
    pub changed_files: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionSmokeVerificationReport {
    pub check: String,
    pub status: String,
    pub command: Option<String>,
    pub output: String,
}

pub(crate) fn failed_report(adapter: &str, error: &anyhow::Error) -> PromotionSmokeReport {
    PromotionSmokeReport {
        schema_version: 1,
        status: PromotionSmokeStatus::Failed,
        adapter: adapter.to_string(),
        workspace: PathBuf::new(),
        commands: Vec::new(),
        verification: Vec::new(),
        promoted_files: Vec::new(),
        final_phase: None,
        final_approval: None,
        error: Some(error.to_string()),
    }
}

pub(crate) fn session_changed_files(session: &TuiSession) -> Vec<String> {
    session
        .changed_files
        .iter()
        .filter_map(|file| file.get("path").and_then(|value| value.as_str()))
        .map(normalize_report_path)
        .collect()
}

fn normalize_report_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub(crate) fn print_text_report(report: &PromotionSmokeReport) {
    println!("architect-mcp-tui promotion smoke: {:?}", report.status);
    println!("adapter: {}", report.adapter);
    if let Some(error) = &report.error {
        println!("error: {error}");
    }
    println!("workspace: {}", report.workspace.display());
    for command in &report.commands {
        println!(
            "- {}: {}",
            command.command,
            if command.ok { "ok" } else { "failed" }
        );
        if let Some(error) = &command.error {
            println!("  error: {error}");
        }
    }
    for check in &report.verification {
        println!("- verification {}: {}", check.check, check.status);
    }
    if !report.promoted_files.is_empty() {
        println!("promoted files: {}", report.promoted_files.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn session_changed_files_normalizes_windows_separators() {
        let mut session = TuiSession::new("build", "codex");
        session.changed_files = vec![json!({ "path": "docs\\codex-adapter-smoke.md" })];

        assert_eq!(
            session_changed_files(&session),
            vec!["docs/codex-adapter-smoke.md".to_string()]
        );
    }
}
