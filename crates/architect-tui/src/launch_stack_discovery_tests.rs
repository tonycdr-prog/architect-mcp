use serde_json::json;

use crate::launch_stack_discovery::{discover_stack_from_records, pr_record_from_value};

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
