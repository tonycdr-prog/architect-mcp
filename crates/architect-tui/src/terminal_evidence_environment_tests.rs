use crate::launch_judge_report::LaunchJudgeTerminalEvidenceEnvironment;
use crate::terminal_evidence_environment::resolve_environment;

#[test]
fn terminal_evidence_accepts_explicit_environment_values() {
    assert_eq!(
        resolve_environment(Some("local-terminal")).expect("local terminal"),
        LaunchJudgeTerminalEvidenceEnvironment::LocalTerminal
    );
    assert_eq!(
        resolve_environment(Some("vm-or-cloud-terminal")).expect("vm/cloud"),
        LaunchJudgeTerminalEvidenceEnvironment::VmOrCloudTerminal
    );
    assert_eq!(
        resolve_environment(Some("hosted-ci")).expect("hosted ci"),
        LaunchJudgeTerminalEvidenceEnvironment::HostedCi
    );
    assert_eq!(
        resolve_environment(Some("container")).expect("container"),
        LaunchJudgeTerminalEvidenceEnvironment::Container
    );
}

#[test]
fn terminal_evidence_rejects_unknown_environment_value() {
    let error = resolve_environment(Some("browser-screenshot"))
        .expect_err("unsupported environment should fail");

    assert!(
        error
            .to_string()
            .contains("unsupported terminal evidence environment")
    );
}
