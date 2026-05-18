use serde_json::json;

use crate::launch_stack::{LaunchStackOptions, build_report_from_items, run_launch_stack};
use crate::launch_stack_discovery::LaunchStackDiscovery;
use crate::launch_stack_github::{
    issue_from_value, pr_from_value, pr_from_value_with_required_checks,
};
use crate::launch_stack_merge_plan::render_launch_stack_merge_plan;

#[test]
fn merge_plan_lists_manual_order_and_blockers_without_mutation() {
    let first = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "base slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let second = pr_from_value(
        11,
        &json!({
            "number": 11,
            "title": "top slice",
            "url": "https://github.com/example/repo/pull/11",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let blocker = issue_from_value(
        136,
        &json!({
            "number": 136,
            "title": "Run terminal QA on Linux and Windows",
            "url": "https://github.com/example/repo/issues/136",
            "state": "OPEN"
        }),
    );
    let mut report = build_report_from_items(
        Some("example/repo".to_string()),
        vec![first, second],
        vec![blocker],
        Vec::new(),
    );
    report.stack_discovery = Some(LaunchStackDiscovery {
        from_pr: 11,
        pull_requests: vec![10, 11],
        stopped_at_base: "main".to_string(),
    });

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("read-only: true"));
    assert!(text.contains("mutation: none"));
    assert!(text.contains("merge order, base to head:"));
    assert!(text.contains("1. PR #10 [ready] base slice"));
    assert!(text.contains("2. PR #11 [ready] top slice"));
    assert!(text.contains("issue #136 Warning state=OPEN"));
    assert!(text.contains("resolve or explicitly waive blocker issue #136"));
    assert!(text.contains("do not tag, publish, or claim launch go"));
}

#[test]
fn merge_plan_does_not_claim_base_to_head_for_supplied_pr_order() {
    let top = pr_from_value(
        11,
        &json!({
            "number": 11,
            "title": "top slice",
            "url": "https://github.com/example/repo/pull/11",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let base = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "base slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let report = build_report_from_items(None, vec![top, base], Vec::new(), Vec::new());

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("pull requests, supplied order:"));
    assert!(!text.contains("merge order, base to head:"));
    assert!(text.contains("verify the true base-to-head order before any manual merge"));
}

#[test]
fn merge_plan_marks_unready_prs_as_hold_with_next_action() {
    let draft = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "draft release slice",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": true,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let report = build_report_from_items(None, vec![draft], Vec::new(), Vec::new());

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("PR #10 [hold]"));
    assert!(text.contains("mark PR #10 ready when it is reviewable and still green"));
}

#[test]
fn merge_plan_shows_review_decision_holds() {
    let changes_requested = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "needs requested changes",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "CHANGES_REQUESTED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let review_required = pr_from_value(
        11,
        &json!({
            "number": 11,
            "title": "needs approval",
            "url": "https://github.com/example/repo/pull/11",
            "isDraft": false,
            "reviewDecision": "REVIEW_REQUIRED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let report = build_report_from_items(
        None,
        vec![changes_requested, review_required],
        Vec::new(),
        Vec::new(),
    );

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("PR #10 [hold] needs requested changes"));
    assert!(text.contains("review=CHANGES_REQUESTED"));
    assert!(text.contains("resolve requested changes on PR #10"));
    assert!(text.contains("PR #11 [hold] needs approval"));
    assert!(text.contains("review=REVIEW_REQUIRED"));
    assert!(text.contains("complete required review on PR #11"));
}

#[test]
fn merge_plan_shows_unresolved_review_thread_holds() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "approved with thread left open",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "unresolvedReviewThreads": 1,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("PR #10 [hold] approved with thread left open"));
    assert!(text.contains("unresolved_threads=1"));
    assert!(text.contains("resolve 1 unresolved review thread(s) on PR #10"));
}

#[test]
fn merge_plan_shows_missing_required_checks() {
    let pr = pr_from_value_with_required_checks(
        10,
        &json!({
            "number": 10,
            "title": "ready except missing platform check",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "reviewDecision": "APPROVED",
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
        &["live-qa (windows-latest)".to_string()],
    );
    let report = build_report_from_items(None, vec![pr], Vec::new(), Vec::new());

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("PR #10 [hold] ready except missing platform check"));
    assert!(text.contains("missing required checks: live-qa (windows-latest)"));
    assert!(text.contains("restore required checks on PR #10"));
}

#[test]
fn merge_plan_keeps_public_safe_text() {
    let pr = pr_from_value(
        10,
        &json!({
            "number": 10,
            "title": "leaks /Users/example/private npm_secret",
            "url": "https://github.com/example/repo/pull/10",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify /home/example", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }),
    );
    let report = build_report_from_items(
        None,
        vec![pr],
        Vec::new(),
        vec!["finding references C:/Users/example and ghp_secret".to_string()],
    );

    let text = render_launch_stack_merge_plan(&report).join("\n");

    assert!(text.contains("[redacted-local-path]"));
    assert!(text.contains("[redacted-secret]"));
    assert!(!text.contains("/Users/"));
    assert!(!text.contains("/home/"));
    assert!(!text.contains("ghp_secret"));
}

#[test]
fn merge_plan_rejects_ambiguous_json_mode_before_lookup() {
    let workspace = std::env::current_dir().expect("current dir");
    let error = run_launch_stack(
        &workspace,
        LaunchStackOptions {
            json: true,
            merge_plan: true,
            repo: None,
            stack_from_pr: None,
            prs: Vec::new(),
            blockers: Vec::new(),
            waived_blockers: Vec::new(),
            required_checks: Vec::new(),
        },
    )
    .expect_err("ambiguous output mode should fail");

    assert!(error.to_string().contains("mutually exclusive"));
}
