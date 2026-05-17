use serde_json::Value;

use crate::arena::{ArenaCandidateInput, ArenaCandidateRecord, ArenaReviewStatus};
use crate::session::TuiSession;

pub(crate) fn candidate_record_from_events(adapter: &str, events: &str) -> ArenaCandidateRecord {
    let mut temp = TuiSession::new("arena candidate", adapter);
    crate::interactive_support::apply_run_evidence(&mut temp, events);
    let mut summary = Vec::new();
    for line in events.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("approval_required") => {
                if let Some(reason) = event.get("reason").and_then(Value::as_str) {
                    summary.push(format!("approval required: {reason}"));
                }
            }
            Some("agent_event") => {
                if let Some(kind) = event
                    .get("event")
                    .and_then(|event| event.get("type"))
                    .and_then(Value::as_str)
                {
                    summary.push(format!("agent event: {kind}"));
                }
            }
            _ => {}
        }
    }
    ArenaCandidateRecord {
        adapter: adapter.to_string(),
        worktree: temp
            .worktree
            .as_ref()
            .map(|path| path.display().to_string()),
        diff_stat: temp.diff_stat.clone(),
        review_status: review_status(&temp),
        verification_passed: verification_passed(&temp),
        diff_size: changed_line_count(&temp),
        contract_drift: contract_drift(&temp),
        crashed: temp.adapter_crashed,
        adapter_run_issues: temp.adapter_run_issues.clone(),
        changed_files: temp.changed_files,
        review_gates: temp.gates,
        summary,
    }
}

pub(crate) fn candidate_line(candidate: &ArenaCandidateRecord) -> String {
    format!(
        "candidate {}: review={:?} files={} worktree={}",
        candidate.adapter,
        candidate.review_status,
        file_list(candidate),
        candidate.worktree.as_deref().unwrap_or("not recorded")
    )
}

pub(crate) fn file_list(candidate: &ArenaCandidateRecord) -> String {
    let files = candidate
        .changed_files
        .iter()
        .filter_map(|file| file.get("path").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if files.is_empty() {
        "none".to_string()
    } else {
        files.join(",")
    }
}

pub(crate) fn arena_input_for(session: &TuiSession, adapter: &str) -> ArenaCandidateInput {
    if adapter != session.adapter {
        return ArenaCandidateInput {
            adapter: adapter.to_string(),
            review_status: ArenaReviewStatus::Unknown,
            verification_passed: false,
            diff_size: 0,
            contract_drift: false,
            crashed: false,
        };
    }
    ArenaCandidateInput {
        adapter: adapter.to_string(),
        review_status: review_status(session),
        verification_passed: verification_passed(session),
        diff_size: changed_line_count(session),
        contract_drift: contract_drift(session),
        crashed: session.adapter_crashed,
    }
}

fn review_status(session: &TuiSession) -> ArenaReviewStatus {
    let Some(review) = session.gates.get("review_implementation_against_contract") else {
        return ArenaReviewStatus::Unknown;
    };
    let text = review.to_string().to_lowercase();
    if text.contains("fail") || text.contains("blocker") {
        ArenaReviewStatus::Fail
    } else if text.contains("warn") || text.contains("risk") {
        ArenaReviewStatus::Warn
    } else {
        ArenaReviewStatus::Pass
    }
}

fn verification_passed(session: &TuiSession) -> bool {
    !session.verification.is_empty()
        && session
            .verification
            .values()
            .all(|status| matches!(status.as_str(), "pass" | "passed" | "ok" | "green"))
}

fn changed_line_count(session: &TuiSession) -> u64 {
    session
        .changed_files
        .iter()
        .filter_map(|file| file.get("lines").and_then(Value::as_u64))
        .sum()
}

fn contract_drift(session: &TuiSession) -> bool {
    session
        .gates
        .get("review_implementation_against_contract")
        .map(|review| review.to_string().to_lowercase().contains("drift"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn candidate_record_extracts_diff_and_crash_evidence() {
        let events = [
            json!({
                "type": "diff_evidence",
                "worktree": ".architect-mcp/worktrees/session/shell",
                "changed_files": [{ "path": "docs/live-qa.md", "lines": 8 }],
                "diff_stat": "docs/live-qa.md | 8 ++++++++"
            })
            .to_string(),
            json!({
                "type": "agent_event",
                "event": { "type": "crashed", "message": "boom" }
            })
            .to_string(),
        ]
        .join("\n");
        let candidate = candidate_record_from_events("shell", &events);
        assert_eq!(candidate.adapter, "shell");
        assert_eq!(candidate.diff_size, 8);
        assert!(candidate.crashed);
        assert_eq!(candidate.adapter_run_issues, vec!["adapter crashed: boom"]);
        assert_eq!(candidate.changed_files[0]["path"], "docs/live-qa.md");
    }

    #[test]
    fn candidate_record_preserves_review_gates_and_diff_stat() {
        let events = [
            json!({
                "type": "diff_evidence",
                "worktree": ".architect-mcp/worktrees/session/shell",
                "changed_files": [{ "path": "docs/live-qa.md", "lines": 3 }],
                "diff_stat": "docs/live-qa.md | 3 +++"
            })
            .to_string(),
            json!({
                "type": "mcp_result",
                "name": "review_repo_structure",
                "result": { "ok": true }
            })
            .to_string(),
        ]
        .join("\n");
        let candidate = candidate_record_from_events("shell", &events);
        assert_eq!(
            candidate.diff_stat.as_deref(),
            Some("docs/live-qa.md | 3 +++")
        );
        assert_eq!(candidate.review_gates["review_repo_structure"]["ok"], true);
    }

    #[test]
    fn candidate_record_marks_timeout_and_cancel_as_blocking_issues() {
        let events = [
            json!({
                "type": "agent_event",
                "event": { "type": "timed_out" }
            })
            .to_string(),
            json!({
                "type": "agent_event",
                "event": { "type": "cancelled" }
            })
            .to_string(),
        ]
        .join("\n");
        let candidate = candidate_record_from_events("shell", &events);
        assert!(candidate.crashed);
        assert_eq!(
            candidate.adapter_run_issues,
            vec!["adapter timed out", "adapter cancelled"]
        );
    }
}
