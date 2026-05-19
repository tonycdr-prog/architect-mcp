pub mod acp;
mod acp_state;
pub mod adapter;
mod adapter_health;
mod adapter_probe_command;
mod adapter_pty;
mod adapter_review;
mod approval;
#[cfg(test)]
mod approval_tests;
pub mod arena;
mod brief;
#[cfg(test)]
mod brief_tests;
pub mod config;
mod config_template;
pub mod foundry;
mod foundry_artifacts;
pub mod foundry_execution;
pub mod foundry_smoke;
mod foundry_smoke_report;
mod foundry_smoke_script;
#[cfg(test)]
mod foundry_smoke_tests;
pub mod foundry_stage;
mod foundry_stage_content;
mod gate_calls;
pub mod governance_audit;
mod governance_audit_mcp;
mod governance_audit_report;
mod governance_audit_support;
#[cfg(test)]
mod governance_audit_tests;
mod governance_memory;
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
pub mod mcp;
mod mcp_client;
pub mod orchestrator;
pub mod promotion_smoke;
mod promotion_smoke_report;
mod promotion_smoke_script;
mod promotion_smoke_verification;
mod promotion_smoke_workspace;
pub mod session;
mod session_mcp_integrations;
#[cfg(test)]
mod session_tests;
pub mod smoke;
mod smoke_report;
mod smoke_types;
pub mod ui;
mod verification;
pub mod walkthrough;
mod walkthrough_script;

pub const APP_NAME: &str = "architect-mcp-tui";
