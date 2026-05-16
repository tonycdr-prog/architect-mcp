use anyhow::Result;
use serde_json::{Value, json};

use crate::session::TuiSession;

const VALID_STATUSES: &[&str] = &["not_run", "passed", "failed", "skipped"];

pub(crate) fn normalize_verification_status(status: &str) -> Result<String> {
    let normalized = status.trim().to_ascii_lowercase();
    if VALID_STATUSES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        anyhow::bail!(
            "verification status must be one of: {}",
            VALID_STATUSES.join(", ")
        )
    }
}

pub(crate) fn required_checks(session: &TuiSession) -> Vec<String> {
    if session.required_verification.is_empty() {
        session.verification.keys().cloned().collect()
    } else {
        session.required_verification.clone()
    }
}

pub(crate) fn verification_records(session: &TuiSession) -> Vec<Value> {
    required_checks(session)
        .into_iter()
        .map(|check| {
            let status = session
                .verification
                .get(&check)
                .map(String::as_str)
                .unwrap_or("not_run");
            json!({ "check": check, "status": status })
        })
        .collect()
}

pub(crate) fn ensure_verification_passed(session: &TuiSession) -> Result<()> {
    let blockers = verification_blockers(session);
    if blockers.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "record passed verification before final/session review: {}",
        blockers.join(", ")
    )
}

fn verification_blockers(session: &TuiSession) -> Vec<String> {
    let checks = required_checks(session);
    if checks.is_empty() {
        return vec!["no required checks captured".to_string()];
    }
    checks
        .into_iter()
        .filter_map(
            |check| match session.verification.get(&check).map(String::as_str) {
                Some("passed") => None,
                Some(status) => Some(format!("{check}={status}")),
                None => Some(format!("{check}=not_run")),
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_requires_each_required_check_to_pass() {
        let mut session = TuiSession::new("build", "shell");
        session.set_required_verification(vec!["npm test".to_string()]);
        assert!(ensure_verification_passed(&session).is_err());

        session
            .verification
            .insert("npm test".to_string(), "failed".to_string());
        assert!(ensure_verification_passed(&session).is_err());

        session
            .verification
            .insert("npm test".to_string(), "passed".to_string());
        assert!(ensure_verification_passed(&session).is_ok());
    }
}
