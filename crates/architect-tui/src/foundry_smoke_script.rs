use crate::walkthrough_script::walkthrough_commands;

pub(crate) fn foundry_smoke_commands(repo: &str, owner: &str, execute: bool) -> Vec<String> {
    let mut commands = walkthrough_commands()
        .iter()
        .take_while(|command| **command != "approve run isolated walkthrough adapter")
        .map(|command| (*command).to_string())
        .collect::<Vec<_>>();
    commands.extend([
        format!("foundry plan {repo} owner={owner}"),
        "foundry approve reviewed local staging for private repo smoke".to_string(),
        "foundry stage".to_string(),
        "foundry approve reviewed private GitHub repo creation for smoke".to_string(),
        if execute {
            "foundry create --execute".to_string()
        } else {
            "foundry create".to_string()
        },
    ]);
    commands
}

#[cfg(test)]
mod tests {
    use super::foundry_smoke_commands;

    #[test]
    fn foundry_smoke_sequence_stops_before_adapter_and_gates_execution() {
        let commands = foundry_smoke_commands("app", "owner", true);
        assert!(commands.contains(&"grill".to_string()));
        assert!(commands.contains(&"contract".to_string()));
        assert!(commands.contains(&"review plan".to_string()));
        assert!(commands.contains(&"review files".to_string()));
        assert!(!commands.contains(&"run adapter".to_string()));
        assert_eq!(
            commands.last().map(String::as_str),
            Some("foundry create --execute")
        );
    }
}
