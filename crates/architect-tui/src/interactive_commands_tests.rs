use crate::interactive_commands::{WorkflowCommand, parse_workflow_command};

#[test]
fn parses_command_palette_actions() {
    assert_eq!(
        parse_workflow_command("new app offline meals"),
        WorkflowCommand::NewApp("offline meals".to_string())
    );
    assert_eq!(
        parse_workflow_command("answer users=home cooks"),
        WorkflowCommand::Answer {
            key: "users".to_string(),
            value: "home cooks".to_string()
        }
    );
    assert_eq!(
        parse_workflow_command("review files"),
        WorkflowCommand::ReviewFiles
    );
    assert_eq!(
        parse_workflow_command("resume abc-123"),
        WorkflowCommand::Resume("abc-123".to_string())
    );
    assert_eq!(
        parse_workflow_command("approve review gates passed"),
        WorkflowCommand::Approve("review gates passed".to_string())
    );
    assert_eq!(
        parse_workflow_command("arena rank"),
        WorkflowCommand::ArenaRank
    );
    assert_eq!(
        parse_workflow_command("verification status"),
        WorkflowCommand::VerificationStatus
    );
    assert_eq!(
        parse_workflow_command("arena run codex, shell"),
        WorkflowCommand::ArenaRun(vec!["codex".to_string(), "shell".to_string()])
    );
    assert_eq!(
        parse_workflow_command("arena select codex"),
        WorkflowCommand::ArenaSelect("codex".to_string())
    );
    assert_eq!(
        parse_workflow_command("integrations recommend database=supabase"),
        WorkflowCommand::IntegrationsRecommend("database=supabase".to_string())
    );
    assert_eq!(
        parse_workflow_command("integrations plan supabase target=codex"),
        WorkflowCommand::IntegrationsPlan {
            server_id: "supabase".to_string(),
            target_client: Some("codex".to_string())
        }
    );
    assert_eq!(
        parse_workflow_command("integrations plan supabase codex"),
        WorkflowCommand::IntegrationsPlan {
            server_id: "supabase".to_string(),
            target_client: Some("codex".to_string())
        }
    );
    assert_eq!(
        parse_workflow_command("integrations plan supabase target="),
        WorkflowCommand::Help
    );
    assert_eq!(
        parse_workflow_command("integrations plan supabase target=codex extra"),
        WorkflowCommand::Help
    );
    assert_eq!(
        parse_workflow_command("mcp plan supabase target=codex extra"),
        WorkflowCommand::Help
    );
    assert_eq!(
        parse_workflow_command("integrations apply .mcp.json"),
        WorkflowCommand::IntegrationsApplyDryRun {
            target_path: Some(".mcp.json".to_string())
        }
    );
    assert_eq!(
        parse_workflow_command("integrations approve reviewed plan"),
        WorkflowCommand::IntegrationsApprove("reviewed plan".to_string())
    );
    assert_eq!(
        parse_workflow_command("integrations write .mcp.json"),
        WorkflowCommand::IntegrationsWrite {
            target_path: Some(".mcp.json".to_string())
        }
    );
    assert_eq!(
        parse_workflow_command("foundry plan launchpad owner=tonycdr-prog"),
        WorkflowCommand::FoundryPlan {
            repo_name: "launchpad".to_string(),
            owner: Some("tonycdr-prog".to_string())
        }
    );
    assert_eq!(
        parse_workflow_command("foundry approve reviewed private repo plan"),
        WorkflowCommand::FoundryApprove("reviewed private repo plan".to_string())
    );
    assert_eq!(
        parse_workflow_command("foundry create"),
        WorkflowCommand::FoundryCreate { execute: false }
    );
    assert_eq!(
        parse_workflow_command("foundry create --execute"),
        WorkflowCommand::FoundryCreate { execute: true }
    );
    assert_eq!(
        parse_workflow_command("foundry stage"),
        WorkflowCommand::FoundryStage
    );
    assert_eq!(
        parse_workflow_command("diff file docs/live-qa.md"),
        WorkflowCommand::DiffFile("docs/live-qa.md".to_string())
    );
    assert_eq!(
        parse_workflow_command("override"),
        WorkflowCommand::Override(String::new())
    );
    assert_eq!(
        parse_workflow_command("override maintainer accepted known warning"),
        WorkflowCommand::Override("maintainer accepted known warning".to_string())
    );
    assert_eq!(
        parse_workflow_command("promotion status"),
        WorkflowCommand::PromotionStatus
    );
    assert_eq!(
        parse_workflow_command("promotion receipt"),
        WorkflowCommand::PromotionReceipt
    );
}
