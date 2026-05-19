pub mod acp;
#[cfg(test)]
mod acp_envelope_tests;
#[cfg(test)]
mod acp_param_tests;
mod acp_state;
#[cfg(test)]
mod acp_tests;
pub mod adapter;
mod adapter_health;
mod adapter_probe_command;
mod adapter_pty;
mod adapter_review;
mod approval;
#[cfg(test)]
mod approval_gate_evidence_tests;
mod approval_reason;
#[cfg(test)]
mod approval_tests;
pub mod arena;
mod brief;
#[cfg(test)]
mod brief_tests;
pub mod cli;
pub mod config;
mod config_template;
pub mod evidence_index;
mod evidence_index_markdown;
mod evidence_index_report;
#[cfg(test)]
mod evidence_index_required_checks_tests;
#[cfg(test)]
mod evidence_index_test_support;
#[cfg(test)]
mod evidence_index_tests;
pub mod foundry;
mod foundry_artifacts;
pub mod foundry_execution;
#[cfg(test)]
mod foundry_execution_tests;
pub mod foundry_smoke;
mod foundry_smoke_github;
mod foundry_smoke_public_summary;
#[cfg(test)]
mod foundry_smoke_public_summary_tests;
mod foundry_smoke_report;
mod foundry_smoke_retention;
mod foundry_smoke_script;
#[cfg(test)]
mod foundry_smoke_tests;
pub mod foundry_stage;
mod foundry_stage_content;
#[cfg(test)]
mod foundry_stage_tests;
mod gate_calls;
pub mod governance_audit;
#[cfg(test)]
mod governance_audit_hardening_tests;
mod governance_audit_mcp;
mod governance_audit_public_summary;
#[cfg(test)]
mod governance_audit_public_summary_tests;
mod governance_audit_report;
mod governance_audit_support;
#[cfg(test)]
mod governance_audit_tests;
mod governance_memory;
mod governance_secret_scan;
mod headless;
mod headless_adapter;
mod headless_events;
mod headless_support;
pub mod interactive;
mod interactive_approval;
mod interactive_arena;
mod interactive_arena_evidence;
mod interactive_commands;
#[cfg(test)]
mod interactive_commands_tests;
mod interactive_diff;
mod interactive_foundry;
mod interactive_gate_flow;
mod interactive_integrations;
mod interactive_integrations_summary;
mod interactive_integrations_support;
mod interactive_support;
mod interactive_update;
pub mod issue_terminal_evidence;
mod issue_terminal_evidence_source;
#[cfg(test)]
mod issue_terminal_evidence_tests;
pub mod launch_judge;
mod launch_judge_evidence;
#[cfg(test)]
mod launch_judge_evidence_environment_tests;
mod launch_judge_evidence_safety;
mod launch_judge_public_summary;
#[cfg(test)]
mod launch_judge_public_summary_tests;
mod launch_judge_report;
#[cfg(test)]
mod launch_judge_tests;
pub mod launch_readiness;
mod launch_readiness_output;
mod launch_readiness_public_summary;
#[cfg(test)]
mod launch_readiness_public_summary_test_support;
#[cfg(test)]
mod launch_readiness_public_summary_tests;
#[cfg(test)]
mod launch_readiness_tests;
pub mod launch_stack;
mod launch_stack_discovery;
#[cfg(test)]
mod launch_stack_discovery_tests;
mod launch_stack_github;
mod launch_stack_github_support;
mod launch_stack_merge_plan;
#[cfg(test)]
mod launch_stack_merge_plan_tests;
#[cfg(test)]
mod launch_stack_mergeability_tests;
mod launch_stack_output;
mod launch_stack_pr_status;
#[cfg(test)]
mod launch_stack_required_check_integration_tests;
mod launch_stack_required_checks;
#[cfg(test)]
mod launch_stack_required_checks_tests;
#[cfg(test)]
mod launch_stack_review_decision_tests;
mod launch_stack_review_threads;
#[cfg(test)]
mod launch_stack_tests;
mod launch_stack_waivers;
pub mod mcp;
mod mcp_client;
pub mod orchestrator;
pub mod promotion_receipt;
pub mod promotion_smoke;
mod promotion_smoke_report;
mod promotion_smoke_script;
mod promotion_smoke_verification;
mod promotion_smoke_workspace;
pub mod session;
mod session_approval;
mod session_mcp_integrations;
#[cfg(test)]
mod session_tests;
pub mod smoke;
mod smoke_report;
mod smoke_types;
pub mod terminal_evidence;
mod terminal_evidence_date;
mod terminal_evidence_environment;
#[cfg(test)]
mod terminal_evidence_environment_tests;
#[cfg(test)]
mod terminal_evidence_hardening_tests;
mod terminal_evidence_issue_url;
#[cfg(test)]
mod terminal_evidence_tests;
pub mod ui;
pub mod untrusted_input;
mod verification;
pub mod walkthrough;
mod walkthrough_script;

pub const APP_NAME: &str = "architect-mcp-tui";
