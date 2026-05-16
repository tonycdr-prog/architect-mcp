pub mod acp;
pub mod adapter;
mod adapter_health;
mod adapter_pty;
pub mod config;
mod config_template;
mod headless;
mod headless_adapter;
mod headless_events;
mod headless_support;
pub mod mcp;
mod mcp_client;
pub mod orchestrator;
pub mod ui;

pub const APP_NAME: &str = "architect-mcp-tui";
