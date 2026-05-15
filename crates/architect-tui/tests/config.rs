use std::collections::BTreeMap;

use architect_tui::adapter::AdapterConfig;
use architect_tui::config::TuiConfig;

#[test]
fn default_config_uses_advanced_surface_and_codex() {
    let config = TuiConfig::default();
    assert_eq!(config.architect_mcp.tool_surface, "advanced");
    assert!(config.adapters.contains_key("codex"));
    assert!(config.validate().is_ok());
}

#[test]
fn rejects_inline_secret_like_env_values() {
    let mut config = TuiConfig::default();
    config.adapters.insert(
        "bad".to_string(),
        AdapterConfig {
            command: "bad".to_string(),
            args: vec![],
            env: BTreeMap::from([("API_TOKEN".to_string(), "plain-value".to_string())]),
            working_directory: None,
            available: None,
            pty: true,
        },
    );
    assert!(config.validate().is_err());
}
