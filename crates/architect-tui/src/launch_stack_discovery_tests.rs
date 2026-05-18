use std::cell::RefCell;
use std::path::Path;

use serde_json::{Value, json};

use crate::launch_stack_discovery::{
    STACK_DISCOVERY_PR_FIELDS, discover_stack_from_pr_with_runner, discover_stack_from_records,
    pr_record_from_value,
};

#[test]
fn stack_discovery_orders_stacked_prs_base_to_head() {
    let records = vec![
        pr_record_from_value(&json!({
            "number": 190,
            "title": "combined readiness",
            "url": "https://github.com/example/repo/pull/190",
            "headRefName": "codex/launch-readiness-report",
            "baseRefName": "codex/launch-stack-waivers",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        })),
        pr_record_from_value(&json!({
            "number": 188,
            "title": "waivers",
            "url": "https://github.com/example/repo/pull/188",
            "headRefName": "codex/launch-stack-waivers",
            "baseRefName": "codex/tui-terminal-evidence-placeholders",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        })),
        pr_record_from_value(&json!({
            "number": 186,
            "title": "placeholder evidence",
            "url": "https://github.com/example/repo/pull/186",
            "headRefName": "codex/tui-terminal-evidence-placeholders",
            "baseRefName": "main",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": [
                {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        })),
    ];

    let discovery = discover_stack_from_records(190, &records).expect("stack discovery");

    assert_eq!(discovery.from_pr, 190);
    assert_eq!(discovery.pull_requests, vec![186, 188, 190]);
    assert_eq!(discovery.stopped_at_base, "main");
}

#[test]
fn stack_discovery_targeted_lookup_uses_pr_view_and_per_base_head_queries() {
    let calls = RefCell::new(Vec::<Vec<String>>::new());
    let runner = |_: &Path, args: &[String]| -> Result<Value, String> {
        calls.borrow_mut().push(args.to_vec());
        match args.get(1).map(String::as_str) {
            Some("view") => Ok(json!({
                "number": 190,
                "headRefName": "codex/launch-readiness-report",
                "baseRefName": "codex/launch-stack-waivers",
                "state": "OPEN"
            })),
            Some("list") => match arg_after(args, "--head").as_deref() {
                Some("codex/launch-stack-waivers") => Ok(json!([{
                    "number": 188,
                    "headRefName": "codex/launch-stack-waivers",
                    "baseRefName": "codex/tui-terminal-evidence-placeholders",
                    "state": "OPEN"
                }])),
                Some("codex/tui-terminal-evidence-placeholders") => Ok(json!([{
                    "number": 186,
                    "headRefName": "codex/tui-terminal-evidence-placeholders",
                    "baseRefName": "main",
                    "state": "OPEN"
                }])),
                Some("main") => Ok(json!([])),
                other => Err(format!("unexpected head lookup: {other:?}")),
            },
            other => Err(format!("unexpected gh command: {other:?}")),
        }
    };

    let discovery = discover_stack_from_pr_with_runner(
        Path::new("."),
        Some("tonycdr-prog/architect-mcp"),
        190,
        &runner,
    )
    .expect("targeted stack discovery");

    assert_eq!(discovery.pull_requests, vec![186, 188, 190]);
    assert_eq!(discovery.stopped_at_base, "main");
    let calls = calls.into_inner();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].get(1).map(String::as_str), Some("view"));
    assert_eq!(
        calls
            .iter()
            .skip(1)
            .filter_map(|args| arg_after(args, "--head"))
            .collect::<Vec<_>>(),
        vec![
            "codex/launch-stack-waivers",
            "codex/tui-terminal-evidence-placeholders",
            "main"
        ]
    );
    assert!(calls.iter().all(|args| {
        args.windows(2)
            .any(|window| window[0] == "--json" && window[1] == STACK_DISCOVERY_PR_FIELDS)
    }));
    assert!(
        !calls
            .iter()
            .flatten()
            .any(|arg| arg.contains("statusCheckRollup"))
    );
}

#[test]
fn stack_discovery_fails_closed_when_head_pr_is_not_open() {
    let records = vec![pr_record_from_value(&json!({
        "number": 190,
        "title": "combined readiness",
        "url": "https://github.com/example/repo/pull/190",
        "headRefName": "codex/launch-readiness-report",
        "baseRefName": "main",
        "state": "CLOSED",
        "isDraft": false,
        "mergeStateStatus": "CLEAN",
        "statusCheckRollup": [
            {"name": "verify", "status": "COMPLETED", "conclusion": "SUCCESS"}
        ]
    }))];

    let error = discover_stack_from_records(190, &records).expect_err("closed PR should fail");

    assert!(error.contains("not an open PR"));
}

#[test]
fn stack_discovery_detects_cycles() {
    let records = vec![
        pr_record_from_value(&json!({
            "number": 1,
            "title": "first",
            "url": "https://github.com/example/repo/pull/1",
            "headRefName": "codex/first",
            "baseRefName": "codex/second",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
        pr_record_from_value(&json!({
            "number": 2,
            "title": "second",
            "url": "https://github.com/example/repo/pull/2",
            "headRefName": "codex/second",
            "baseRefName": "codex/first",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
    ];

    let error = discover_stack_from_records(1, &records).expect_err("cycle should fail");

    assert!(error.contains("cycle detected"));
}

#[test]
fn stack_discovery_fails_closed_when_base_branch_targets_non_open_pr() {
    let records = vec![
        pr_record_from_value(&json!({
            "number": 190,
            "title": "combined readiness",
            "url": "https://github.com/example/repo/pull/190",
            "headRefName": "codex/launch-readiness-report",
            "baseRefName": "codex/launch-stack-waivers",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
        pr_record_from_value(&json!({
            "number": 188,
            "title": "waivers",
            "url": "https://github.com/example/repo/pull/188",
            "headRefName": "codex/launch-stack-waivers",
            "baseRefName": "main",
            "state": "CLOSED",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
    ];

    let error =
        discover_stack_from_records(190, &records).expect_err("closed linked PR should fail");

    assert!(error.contains("not open"));
    assert!(error.contains("codex/launch-stack-waivers"));
}

#[test]
fn stack_discovery_fails_closed_when_base_branch_matches_duplicate_open_heads() {
    let records = vec![
        pr_record_from_value(&json!({
            "number": 190,
            "title": "combined readiness",
            "url": "https://github.com/example/repo/pull/190",
            "headRefName": "codex/launch-readiness-report",
            "baseRefName": "codex/launch-stack-waivers",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
        pr_record_from_value(&json!({
            "number": 188,
            "title": "waivers",
            "url": "https://github.com/example/repo/pull/188",
            "headRefName": "codex/launch-stack-waivers",
            "baseRefName": "main",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
        pr_record_from_value(&json!({
            "number": 191,
            "title": "forked branch with same name",
            "url": "https://github.com/example/repo/pull/191",
            "headRefName": "codex/launch-stack-waivers",
            "baseRefName": "main",
            "state": "OPEN",
            "isDraft": false,
            "mergeStateStatus": "CLEAN",
            "statusCheckRollup": []
        })),
    ];

    let error =
        discover_stack_from_records(190, &records).expect_err("duplicate heads should fail");

    assert!(error.contains("multiple open PR heads"));
    assert!(error.contains("#188"));
    assert!(error.contains("#191"));
    assert!(error.contains("codex/launch-stack-waivers"));
}

fn arg_after(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}
