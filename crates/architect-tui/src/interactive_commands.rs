#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowCommand {
    NewApp(String),
    Answer { key: String, value: String },
    Grill,
    CreateContract,
    ReviewPlan,
    ReviewFiles,
    RunAdapter,
    RecordVerification { check: String, status: String },
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
        "session review" => WorkflowCommand::SessionReview,
        "cancel" => WorkflowCommand::Cancel,
        _ if trimmed.starts_with("record verification ") => parse_verification(trimmed),
        _ if trimmed.starts_with("final review ") => {
            WorkflowCommand::FinalReview(trimmed["final review ".len()..].trim().to_string())
        }
        _ => WorkflowCommand::Help,
    }
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
    }
}
