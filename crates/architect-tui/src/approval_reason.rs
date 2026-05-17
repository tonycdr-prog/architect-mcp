use anyhow::Result;

use crate::session::{ApprovalStatus, TuiSession};

const DEFAULT_PROMOTION_OVERRIDE_REASON: &str = "manual TUI override";

pub(crate) fn normalize_promotion_override_reason(reason: &str) -> Result<String> {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        anyhow::bail!("promotion override requires an explicit reason; use override <reason>");
    }
    if trimmed.eq_ignore_ascii_case(DEFAULT_PROMOTION_OVERRIDE_REASON) {
        anyhow::bail!(
            "promotion override requires a maintainer-written reason, not the default placeholder"
        );
    }
    Ok(trimmed.to_string())
}

pub(crate) fn valid_promotion_override_recorded(session: &TuiSession) -> bool {
    session.approval_status == ApprovalStatus::Override
        && session
            .approval_reason
            .as_deref()
            .is_some_and(|reason| normalize_promotion_override_reason(reason).is_ok())
}
