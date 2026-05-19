use crate::launch_stack_github::public_text;
use crate::launch_stack_required_checks::missing_required_checks;

#[test]
fn required_checks_match_exactly_and_dedupe_missing_names() {
    let missing = missing_required_checks(
        &[
            "verify".to_string(),
            "CI / lint".to_string(),
            "live-qa (ubuntu-latest)".to_string(),
        ],
        &[
            "verify".to_string(),
            "VERIFY".to_string(),
            "ci/lint".to_string(),
            "ci/lint".to_string(),
            "live-qa (windows-latest)".to_string(),
            "live-qa (windows-latest)".to_string(),
            " ".to_string(),
        ],
    );

    assert_eq!(
        missing,
        vec!["VERIFY", "ci/lint", "live-qa (windows-latest)"]
    );
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

#[test]
fn public_text_preserves_github_urls_with_path_like_segments() {
    let text = public_text(
        "docs https://api.github.com/users/example repo https://github.com/example/tmp/pull/10 path=/tmp/workspace",
        300,
    );

    assert!(text.contains("https://api.github.com/users/example"));
    assert!(text.contains("https://github.com/example/tmp/pull/10"));
    assert!(text.contains("[redacted-local-path]"));
    assert!(!text.contains("/tmp/workspace"));
}
