use std::path::Path;

use crate::adapter::AdapterConfig;
use crate::config::TuiConfig;
use crate::mcp::ArchitectMcpBridge;

pub(crate) fn config_for_walkthrough(source_workspace: &Path, mut config: TuiConfig) -> TuiConfig {
    let spec = ArchitectMcpBridge::new(source_workspace, config.clone()).process_spec();
    config.architect_mcp.command = Some(spec.command);
    config.architect_mcp.args = spec.args;
    config.architect_mcp.tool_surface = spec.tool_surface;
    config.agents.default_adapter = "walkthrough".to_string();
    config.agents.default_timeout_seconds = 30;
    config
        .adapters
        .insert("walkthrough".to_string(), walkthrough_writer());
    config
}

pub(crate) fn walkthrough_commands() -> &'static [&'static str] {
    &[
        "new app ready local notes app with users flows stack risks verification",
        "answer users=one developer managing local notes",
        "answer coreFlows=create note; edit note; delete note; search notes",
        "answer stack=frontend=TypeScript CLI; backend=local file service",
        "answer storage=local JSON file under user data directory",
        "answer enforcement=advisory findings during intake, with manual TUI approval gates before execution and promotion",
        "answer repoLayout=features=src/features/notes/live-qa.ts; services=src/services; docs=docs; tests=tests",
        "answer constraints=generate AGENTS.md, architecture contract, build plan, review gate evidence, and CI workflow before coding",
        "answer dataEntities=Note entity owned by src/features/notes; UserSettings entity owned by src/services/settings",
        "answer risk=file corruption and accidental data loss",
        "answer verification=npm test",
        "grill",
        "contract",
        "review plan",
        "review files",
        "approve run isolated walkthrough adapter",
        "run adapter",
        "diff summary",
        "diff file src/features/notes/live-qa.ts",
        "verification status",
        "record verification architecture contract validates=passed",
        "record verification npm test=passed",
        "record verification review_repo_structure=passed",
        "final review Changed files: src/features/notes/live-qa.ts. Verification: architecture contract validates passed; npm test passed; review_repo_structure passed. Assumptions: fixture adapter wrote a safe local walkthrough file. Not done: production app remains out of scope for walkthrough.",
        "session review",
        "promotion status",
        "approve promote reviewed diff",
        "promote",
    ]
}

fn walkthrough_writer() -> AdapterConfig {
    AdapterConfig {
        command: "node".to_string(),
        args: vec![
            "-e".to_string(),
            "const fs = require('node:fs'); fs.mkdirSync('src/features/notes', { recursive: true }); fs.writeFileSync('src/features/notes/live-qa.ts', 'export const walkthroughEvidence = true;');".to_string(),
        ],
        pty: false,
        ..AdapterConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TuiConfig;

    #[test]
    fn walkthrough_command_sequence_covers_full_promotion_path() {
        let commands = walkthrough_commands();
        assert!(commands.contains(&"grill"));
        assert!(commands.contains(&"contract"));
        assert!(commands.contains(&"review plan"));
        assert!(commands.contains(&"review files"));
        assert!(commands.contains(&"run adapter"));
        assert!(commands.contains(&"diff summary"));
        assert!(commands.contains(&"verification status"));
        assert!(commands.contains(&"record verification architecture contract validates=passed"));
        assert!(commands.contains(&"session review"));
        assert!(commands.contains(&"promotion status"));
        assert_eq!(commands.last(), Some(&"promote"));
    }

    #[test]
    fn walkthrough_config_uses_source_mcp_and_fixture_adapter() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut config = TuiConfig::default();
        config.architect_mcp.command = Some("node".to_string());
        config.architect_mcp.args = vec!["fake-mcp.mjs".to_string()];

        let config = config_for_walkthrough(temp.path(), config);

        assert_eq!(config.architect_mcp.command.as_deref(), Some("node"));
        assert_eq!(config.architect_mcp.args, vec!["fake-mcp.mjs"]);
        assert_eq!(config.agents.default_adapter, "walkthrough");
        let adapter = config.adapters.get("walkthrough").expect("adapter");
        assert!(!adapter.pty);
    }
}
