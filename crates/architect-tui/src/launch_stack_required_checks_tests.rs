use crate::launch_stack_required_checks::missing_required_checks;

#[test]
fn required_checks_match_case_insensitively_and_dedupe_missing_names() {
    let missing = missing_required_checks(
        &["verify".to_string(), "live-qa (ubuntu-latest)".to_string()],
        &[
            "VERIFY".to_string(),
            "live-qa (windows-latest)".to_string(),
            "live-qa (windows-latest)".to_string(),
            " ".to_string(),
        ],
    );

    assert_eq!(missing, vec!["live-qa (windows-latest)"]);
}

#[test]
fn required_check_names_are_public_safe() {
    let missing =
        missing_required_checks(&[], &["publish npm_SECRET from /Users/example".to_string()]);

    assert_eq!(
        missing,
        vec!["publish [redacted-secret] from [redacted-local-path]"]
    );
}
