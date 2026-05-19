use crate::launch_stack::LaunchStackCheckSummary;
use crate::launch_stack_github::public_text;

pub(crate) fn apply_required_checks(
    checks: &mut LaunchStackCheckSummary,
    required_checks: &[String],
) {
    let required = public_required_checks(required_checks);
    checks.missing_required_names =
        required_names_not_in(&required, &checks.names).collect::<Vec<_>>();
    checks.pending_required_names =
        required_names_in(&required, &checks.pending_names).collect::<Vec<_>>();
    checks.failed_required_names =
        required_names_in(&required, &checks.failed_names).collect::<Vec<_>>();
}

#[cfg(test)]
pub(crate) fn missing_required_checks(
    available_names: &[String],
    required_checks: &[String],
) -> Vec<String> {
    let required = public_required_checks(required_checks);
    required_names_not_in(&required, available_names).collect::<Vec<_>>()
}

fn public_required_checks(required_checks: &[String]) -> Vec<String> {
    let mut missing = Vec::new();

    for raw in required_checks {
        let required = public_text(raw, 120);
        if required.trim().is_empty() {
            continue;
        }
        if missing.iter().any(|name: &String| name == &required) {
            continue;
        }
        missing.push(required);
    }

    missing
}

fn required_names_not_in<'a>(
    required: &'a [String],
    available_names: &'a [String],
) -> impl Iterator<Item = String> + 'a {
    required
        .iter()
        .filter(|required| !available_names.iter().any(|name| name == *required))
        .cloned()
}

fn required_names_in<'a>(
    required: &'a [String],
    available_names: &'a [String],
) -> impl Iterator<Item = String> + 'a {
    required
        .iter()
        .filter(|required| available_names.iter().any(|name| name == *required))
        .cloned()
}
