use super::*;

#[test]
fn codex_auth_probe_requires_logged_in_success() {
    assert_eq!(
        codex_auth_status_from_output(true, "Logged in as user@example.com"),
        AuthStatus::Authenticated
    );
    assert_eq!(
        codex_auth_status_from_output(true, "Not logged in"),
        AuthStatus::NeedsLogin
    );
    assert_eq!(
        codex_auth_status_from_output(false, "Logged in"),
        AuthStatus::NeedsLogin
    );
}

#[test]
fn adapter_health_json_uses_public_field_names() {
    let health = AdapterHealth {
        name: "codex".to_string(),
        command: "codex".to_string(),
        installed: true,
        version: Some("codex 1.0.0".to_string()),
        auth_status: AuthStatus::Authenticated,
        ready: true,
        detail: "Logged in".to_string(),
    };
    let json = serde_json::to_value(&health).expect("json");
    assert_eq!(json["authStatus"], "authenticated");
    assert_eq!(json["ready"], true);
}

#[cfg(unix)]
#[test]
fn shell_probe_uses_noop_instead_of_version() {
    let health = probe_adapter_health(
        "shell",
        &AdapterConfig {
            command: "sh".to_string(),
            ..AdapterConfig::default()
        },
    );
    assert!(health.installed);
    assert!(health.ready);
    assert_eq!(health.auth_status, AuthStatus::NotApplicable);
}
