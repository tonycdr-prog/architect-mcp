use serde::Serialize;

use crate::launch_judge_report::{
    LaunchJudgeCheck, LaunchJudgeCheckStatus, LaunchJudgeResult, LaunchJudgeTerminalEvidenceSummary,
};
use crate::launch_scope::LaunchScope;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FutureLaunchReadiness {
    pub public_cli: LaunchJudgeResult,
    pub missing_evidence: Vec<String>,
}

pub(crate) fn future_launch_readiness(
    scope: LaunchScope,
    terminal_evidence: &LaunchJudgeTerminalEvidenceSummary,
    terminal_evidence_check: &LaunchJudgeCheck,
) -> FutureLaunchReadiness {
    let public_cli = match terminal_evidence_check.status {
        LaunchJudgeCheckStatus::Failed => LaunchJudgeResult::NoGo,
        LaunchJudgeCheckStatus::Passed => LaunchJudgeResult::Go,
        LaunchJudgeCheckStatus::Info
        | LaunchJudgeCheckStatus::Skipped
        | LaunchJudgeCheckStatus::Warning => LaunchJudgeResult::ConditionalGo,
    };
    let missing_evidence = if public_cli == LaunchJudgeResult::Go {
        Vec::new()
    } else if terminal_evidence.issues.is_empty() {
        vec![format!(
            "external terminal evidence is incomplete for future public-cli readiness from current {} scope",
            scope.label()
        )]
    } else {
        terminal_evidence.issues.clone()
    };
    FutureLaunchReadiness {
        public_cli,
        missing_evidence,
    }
}
