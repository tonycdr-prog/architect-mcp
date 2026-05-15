use architect_tui::config::TuiConfig;
use architect_tui::orchestrator::Orchestrator;

#[test]
fn new_app_workflow_has_golden_gate_order() {
    let orchestrator = Orchestrator::new(".", TuiConfig::default());
    let workflow = orchestrator.new_app_workflow("offline recipe planner");
    let gates: Vec<_> = workflow
        .steps
        .iter()
        .map(|step| step.gate.as_str())
        .collect();
    assert_eq!(gates[0], "grill_me");
    assert!(gates.contains(&"create_pre_edit_contract"));
    assert!(gates.contains(&"review_build_plan"));
    assert!(gates.contains(&"review_proposed_file_plan"));
    assert!(gates.contains(&"review_implementation_against_contract"));
    assert_eq!(gates.last(), Some(&"review_agent_session"));
}

#[test]
fn arena_candidates_use_isolated_worktrees() {
    let orchestrator = Orchestrator::new("/tmp/work", TuiConfig::default());
    let candidates = orchestrator.arena_candidates("session-1", &["codex".into(), "claude".into()]);
    assert!(candidates[0].worktree.ends_with("session-1/codex"));
    assert!(candidates[1].worktree.ends_with("session-1/claude"));
}
