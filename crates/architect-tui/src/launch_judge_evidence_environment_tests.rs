use crate::launch_judge_evidence::build_terminal_evidence_summary;
use crate::launch_judge_report::{
    LaunchJudgeCheckStatus, LaunchJudgeTerminalEvidenceEnvironment,
    LaunchJudgeTerminalEvidenceReport, LaunchJudgeTerminalEvidenceStatus,
};

#[test]
fn hosted_ci_terminal_evidence_stays_conditional() {
    let reports = vec![
        report(
            "linux",
            Some(LaunchJudgeTerminalEvidenceEnvironment::HostedCi),
        ),
        report(
            "windows",
            Some(LaunchJudgeTerminalEvidenceEnvironment::VmOrCloudTerminal),
        ),
    ];

    let (summary, check) =
        build_terminal_evidence_summary(Some("issue #136".to_string()), reports, Vec::new(), false);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Warning);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("hosted CI"))
    );
}

#[test]
fn missing_or_container_environment_provenance_stays_conditional() {
    let reports = vec![
        report("linux", None),
        report(
            "windows",
            Some(LaunchJudgeTerminalEvidenceEnvironment::Container),
        ),
    ];

    let (summary, check) =
        build_terminal_evidence_summary(Some("issue #136".to_string()), reports, Vec::new(), false);

    assert_eq!(check.status, LaunchJudgeCheckStatus::Warning);
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("needs environment provenance"))
    );
    assert!(
        summary
            .issues
            .iter()
            .any(|issue| issue.contains("container"))
    );
}

fn report(
    platform: &str,
    environment: Option<LaunchJudgeTerminalEvidenceEnvironment>,
) -> LaunchJudgeTerminalEvidenceReport {
    LaunchJudgeTerminalEvidenceReport {
        platform: platform.to_string(),
        status: LaunchJudgeTerminalEvidenceStatus::Passed,
        environment,
        source: format!("issue #136 {platform} public-safe summary"),
        command_summary: format!("architect-mcp-tui terminal-evidence passed on {platform}"),
        collected_at: Some("2026-05-17".to_string()),
        notes: Some("summary only, no raw logs".to_string()),
    }
}
