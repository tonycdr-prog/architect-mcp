use std::path::Path;

use crate::foundry_smoke::{FoundryGithubVerification, FoundrySmokeStatus, foundry_status};

#[test]
fn live_status_requires_private_repo_and_open_draft_pr() {
    let stage = Path::new("/tmp/staged");
    let mut error = None;
    assert_eq!(
        foundry_status(
            true,
            Some(stage),
            Some("passed"),
            Some(&sample_github(true, Some(true), Some("OPEN"))),
            &mut error,
        ),
        FoundrySmokeStatus::Passed
    );

    for verification in [
        sample_github(false, Some(true), Some("OPEN")),
        sample_github(true, Some(false), Some("OPEN")),
        sample_github(true, Some(true), Some("CLOSED")),
        sample_github(true, None, Some("OPEN")),
    ] {
        let mut error = None;
        assert_eq!(
            foundry_status(
                true,
                Some(stage),
                Some("passed"),
                Some(&verification),
                &mut error,
            ),
            FoundrySmokeStatus::Failed
        );
    }
}

fn sample_github(
    is_private: bool,
    draft_pr_is_draft: Option<bool>,
    draft_pr_state: Option<&str>,
) -> FoundryGithubVerification {
    FoundryGithubVerification {
        repo_url: "https://github.com/owner/repo".to_string(),
        repo_visibility: if is_private { "PRIVATE" } else { "PUBLIC" }.to_string(),
        is_private,
        draft_pr_url: draft_pr_is_draft.map(|_| "https://github.com/owner/repo/pull/1".to_string()),
        draft_pr_number: draft_pr_is_draft.map(|_| 1),
        draft_pr_is_draft,
        draft_pr_state: draft_pr_state.map(ToString::to_string),
    }
}
