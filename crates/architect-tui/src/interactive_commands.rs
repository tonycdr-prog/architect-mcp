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
    PromotionReceipt,
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
    FoundryPlan {
        repo_name: String,
        owner: Option<String>,
    },
    FoundryStatus,
    FoundryAudit {
        target_path: Option<String>,
    },
    FoundryLedger,
    FoundryApprove(String),
    FoundryStage,
    FoundryCreate {
        execute: bool,
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
        "override" => WorkflowCommand::Override(String::new()),
        "promote" => WorkflowCommand::Promote,
        "promotion status" => WorkflowCommand::PromotionStatus,
        "promotion receipt" | "receipt" => WorkflowCommand::PromotionReceipt,
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
        "foundry status" | "repo status" => WorkflowCommand::FoundryStatus,
        "foundry audit" | "repo audit" => WorkflowCommand::FoundryAudit { target_path: None },
        "foundry ledger" | "repo ledger" => WorkflowCommand::FoundryLedger,
        "foundry stage" | "repo stage" => WorkflowCommand::FoundryStage,
        "foundry create" | "repo create" => WorkflowCommand::FoundryCreate { execute: false },
        "foundry create --execute" | "repo create --execute" | "foundry execute" => {
            WorkflowCommand::FoundryCreate { execute: true }
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
        _ if trimmed.starts_with("foundry plan ") => {
            parse_foundry_plan(&trimmed["foundry plan ".len()..])
        }
        _ if trimmed.starts_with("repo plan ") => {
            parse_foundry_plan(&trimmed["repo plan ".len()..])
        }
        _ if trimmed.starts_with("foundry audit ") => WorkflowCommand::FoundryAudit {
            target_path: optional_path_arg(&trimmed["foundry audit ".len()..]),
        },
        _ if trimmed.starts_with("repo audit ") => WorkflowCommand::FoundryAudit {
            target_path: optional_path_arg(&trimmed["repo audit ".len()..]),
        },
        _ if trimmed.starts_with("foundry approve ") => {
            WorkflowCommand::FoundryApprove(trimmed["foundry approve ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("repo approve ") => {
            WorkflowCommand::FoundryApprove(trimmed["repo approve ".len()..].trim().to_string())
        }
        _ if trimmed.starts_with("record verification ") => parse_verification(trimmed),
        _ if trimmed.starts_with("final review ") => {
            WorkflowCommand::FinalReview(trimmed["final review ".len()..].trim().to_string())
        }
        _ => WorkflowCommand::Help,
    }
}

fn optional_path_arg(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value.strip_prefix("path=").unwrap_or(value).trim();
    optional_string(value)
}

fn parse_foundry_plan(value: &str) -> WorkflowCommand {
    let mut repo_name = String::new();
    let mut owner = None;
    for part in value.split_whitespace() {
        if let Some(owner_value) = part.strip_prefix("owner=") {
            owner = optional_string(owner_value);
        } else if repo_name.is_empty() {
            repo_name = part.to_string();
        }
    }
    WorkflowCommand::FoundryPlan { repo_name, owner }
}

fn parse_integration_plan(value: &str) -> WorkflowCommand {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    let ([server_id] | [server_id, _]) = parts.as_slice() else {
        return WorkflowCommand::Help;
    };
    let server_id = server_id.trim();
    if server_id.is_empty() {
        return WorkflowCommand::Help;
    }
    let target_client = match parts.as_slice() {
        [_] => None,
        [_, target] => {
            let target = target.strip_prefix("target=").unwrap_or(target).trim();
            if target.is_empty() {
                return WorkflowCommand::Help;
            }
            Some(target.to_string())
        }
        _ => unreachable!("integration plan parser already rejected extra arguments"),
    };
    WorkflowCommand::IntegrationsPlan {
        server_id: server_id.to_string(),
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
