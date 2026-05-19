#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowCommand {
    NewApp(String),
    Resume(String),
    Answer {
        key: String,
        value: String,
    },
    Grill,
    CreateContract,
    ReviewPlan,
    ReviewFiles,
    RunAdapter,
    Approve(String),
    Reject(String),
    Override(String),
    Promote,
    PromotionStatus,
    DiffSummary,
    DiffFile(String),
    ArenaRun(Vec<String>),
    ArenaRank,
    ArenaSelect(String),
    IntegrationsRecommend(String),
    IntegrationsPlan {
        server_id: String,
        target_client: Option<String>,
    },
    IntegrationsReview,
    IntegrationsApplyDryRun {
        target_path: Option<String>,
    },
    IntegrationsApprove(String),
    IntegrationsWrite {
        target_path: Option<String>,
    },
    VerificationStatus,
    RecordVerification {
        check: String,
        status: String,
    },
    FinalReview(String),
    SessionReview,
    Cancel,
    Help,
}

pub fn parse_workflow_command(input: &str) -> WorkflowCommand {
    let trimmed = input.trim();
    if let Some(idea) = trimmed.strip_prefix("new app ") {
        return WorkflowCommand::NewApp(idea.trim().to_string());
    }
    if let Some(id) = trimmed
        .strip_prefix("resume ")
        .or_else(|| trimmed.strip_prefix("load session "))
    {
        return WorkflowCommand::Resume(id.trim().to_string());
    }
    if let Some(answer) = trimmed.strip_prefix("answer ")
        && let Some((key, value)) = answer.split_once('=')
    {
        return WorkflowCommand::Answer {
            key: key.trim().to_string(),
            value: value.trim().to_string(),
        };
    }
    match trimmed {
        "grill" => WorkflowCommand::Grill,
        "contract" | "create contract" => WorkflowCommand::CreateContract,
        "review plan" => WorkflowCommand::ReviewPlan,
        "review files" | "review file plan" => WorkflowCommand::ReviewFiles,
        "run adapter" => WorkflowCommand::RunAdapter,
        "approve" => WorkflowCommand::Approve("approved in TUI".to_string()),
        "reject" => WorkflowCommand::Reject("rejected in TUI".to_string()),
        "override" => WorkflowCommand::Override("manual TUI override".to_string()),
        "promote" => WorkflowCommand::Promote,
        "promotion status" => WorkflowCommand::PromotionStatus,
        "diff" | "diff summary" => WorkflowCommand::DiffSummary,
        "arena rank" => WorkflowCommand::ArenaRank,
        "integrations recommend" | "mcp recommend" => {
            WorkflowCommand::IntegrationsRecommend(String::new())
        }
        "integrations review" | "mcp review" => WorkflowCommand::IntegrationsReview,
        "integrations apply" | "mcp apply" => {
            WorkflowCommand::IntegrationsApplyDryRun { target_path: None }
        }
        "integrations write" | "mcp write" => {
            WorkflowCommand::IntegrationsWrite { target_path: None }
        }
        "verification" | "verification status" => WorkflowCommand::VerificationStatus,
        "session review" => WorkflowCommand::SessionReview,
        "cancel" => WorkflowCommand::Cancel,
        _ if trimmed.starts_with("approve ") => {
            WorkflowCommand::Approve(trimmed["approve ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("reject ") => {
            WorkflowCommand::Reject(trimmed["reject ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("override ") => {
            WorkflowCommand::Override(trimmed["override ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("diff file ") => {
            WorkflowCommand::DiffFile(trimmed["diff file ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("arena run ") => {
            WorkflowCommand::ArenaRun(parse_adapters(&trimmed["arena run ".len()..]))
        }
        _ if trimmed.starts_with("arena select ") => {
            WorkflowCommand::ArenaSelect(trimmed["arena select ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("integrations recommend ") => {
            WorkflowCommand::IntegrationsRecommend(
                trimmed["integrations recommend ".len()..]
                    .trim()
                    .to_string(),
            )
        }
        _ if trimmed.starts_with("mcp recommend ") => WorkflowCommand::IntegrationsRecommend(
            trimmed["mcp recommend ".len()..].trim().to_string(),
        ),
        _ if trimmed.starts_with("integrations plan ") => {
            parse_integration_plan(&trimmed["integrations plan ".len()..])
        }
        _ if trimmed.starts_with("mcp plan ") => {
            parse_integration_plan(&trimmed["mcp plan ".len()..])
        }
        _ if trimmed.starts_with("integrations apply ") => {
            WorkflowCommand::IntegrationsApplyDryRun {
                target_path: optional_string(&trimmed["integrations apply ".len()..]),
            }
        }
        _ if trimmed.starts_with("mcp apply ") => WorkflowCommand::IntegrationsApplyDryRun {
            target_path: optional_string(&trimmed["mcp apply ".len()..]),
        },
        _ if trimmed.starts_with("integrations approve ") => WorkflowCommand::IntegrationsApprove(
            trimmed["integrations approve ".len()..].trim().to_string(),
        ),
        _ if trimmed.starts_with("mcp approve ") => {
            WorkflowCommand::IntegrationsApprove(trimmed["mcp approve ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("integrations write ") => WorkflowCommand::IntegrationsWrite {
            target_path: optional_string(&trimmed["integrations write ".len()..]),
        },
        _ if trimmed.starts_with("mcp write ") => WorkflowCommand::IntegrationsWrite {
            target_path: optional_string(&trimmed["mcp write ".len()..]),
        },
        _ if trimmed.starts_with("record verification ") => parse_verification(trimmed),
        _ if trimmed.starts_with("final review ") => {
            WorkflowCommand::FinalReview(trimmed["final review ".len()..].trim().to_string())
        }
        _ => WorkflowCommand::Help,
    }
}

fn parse_integration_plan(value: &str) -> WorkflowCommand {
    let mut parts = value.split_whitespace();
    let Some(server_id) = parts.next() else {
        return WorkflowCommand::Help;
    };
    let target_client = parts.next().map(|part| {
        part.strip_prefix("target=")
            .unwrap_or(part)
            .trim()
            .to_string()
    });
    WorkflowCommand::IntegrationsPlan {
        server_id: server_id.trim().to_string(),
        target_client,
    }
}

fn optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_adapters(value: &str) -> Vec<String> {
    value
        .split([',', ' '])
        .map(str::trim)
        .filter(|adapter| !adapter.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_verification(input: &str) -> WorkflowCommand {
    let value = input["record verification ".len()..].trim();
    let Some((check, status)) = value.rsplit_once('=') else {
        return WorkflowCommand::Help;
    };
    WorkflowCommand::RecordVerification {
        check: check.trim().to_string(),
        status: status.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            parse_workflow_command("diff file docs/live-qa.md"),
            WorkflowCommand::DiffFile("docs/live-qa.md".to_string())
        );
        assert_eq!(
            parse_workflow_command("override maintainer accepted known warning"),
            WorkflowCommand::Override("maintainer accepted known warning".to_string())
        );
        assert_eq!(
            parse_workflow_command("promotion status"),
            WorkflowCommand::PromotionStatus
        );
    }
}
