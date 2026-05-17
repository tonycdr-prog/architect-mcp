use crate::launch_stack::LaunchStackCheckSummary;
use crate::launch_stack_github::public_text;

pub(crate) fn apply_required_checks(
    checks: &mut LaunchStackCheckSummary,
    required_checks: &[String],
) {
    checks.missing_required_names = missing_required_checks(&checks.names, required_checks);
}

pub(crate) fn missing_required_checks(
    available_names: &[String],
    required_checks: &[String],
) -> Vec<String> {
    let available = available_names
        .iter()
        .map(|name| normalize_check_name(name))
        .collect::<Vec<_>>();
    let mut missing = Vec::new();

    for raw in required_checks {
        let required = public_text(raw, 120);
        let normalized = normalize_check_name(&required);
        if normalized.is_empty() {
            continue;
        }
        if missing
            .iter()
            .any(|name: &String| normalize_check_name(name) == normalized)
        {
            continue;
        }
        if !available.iter().any(|name| name == &normalized) {
            missing.push(required);
        }
    }

    missing
}

fn normalize_check_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}
