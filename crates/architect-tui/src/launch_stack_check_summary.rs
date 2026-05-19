use std::collections::BTreeMap;

use serde_json::Value;

use crate::launch_stack::LaunchStackCheckSummary;
use crate::launch_stack_github_support::public_text;

pub(crate) fn summarize_checks(value: Option<&Value>) -> LaunchStackCheckSummary {
    let mut summary = LaunchStackCheckSummary {
        total: 0,
        passed: 0,
        pending: 0,
        failed: 0,
        names: Vec::new(),
        pending_names: Vec::new(),
        failed_names: Vec::new(),
        missing_required_names: Vec::new(),
        pending_required_names: Vec::new(),
        failed_required_names: Vec::new(),
    };
    let Some(items) = value.and_then(Value::as_array) else {
        return summary;
    };
    let mut latest_by_key = BTreeMap::<(String, Option<String>), CheckRollupEntry>::new();
    for (index, item) in items.iter().enumerate() {
        let Some(entry) = check_rollup_entry(item, index) else {
            continue;
        };
        let key = (entry.name.clone(), entry.app_key.clone());
        match latest_by_key.get(&key) {
            Some(existing)
                if (existing.sort_key.as_str(), existing.order)
                    >= (entry.sort_key.as_str(), entry.order) => {}
            _ => {
                latest_by_key.insert(key, entry);
            }
        }
    }
    summary.total = latest_by_key.len();
    let mut entries = latest_by_key.values().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.order);
    for entry in entries {
        summary.names.push(entry.name.clone());
        match entry.state {
            CheckRollupState::Passed => {
                summary.passed += 1;
            }
            CheckRollupState::Pending => {
                summary.pending += 1;
                summary.pending_names.push(entry.name.clone());
            }
            CheckRollupState::Failed => {
                summary.failed += 1;
                summary.failed_names.push(entry.name.clone());
            }
        }
    }
    summary
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckRollupEntry {
    name: String,
    app_key: Option<String>,
    state: CheckRollupState,
    sort_key: String,
    order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckRollupState {
    Passed,
    Pending,
    Failed,
}

fn check_rollup_entry(item: &Value, order: usize) -> Option<CheckRollupEntry> {
    let name = item
        .get("name")
        .or_else(|| item.get("context"))
        .and_then(Value::as_str)
        .map(|name| public_text(name, 120))
        .filter(|name| !name.trim().is_empty())?;
    Some(CheckRollupEntry {
        state: check_rollup_state(item),
        sort_key: check_rollup_sort_key(item),
        app_key: check_rollup_app_key(item),
        order,
        name,
    })
}

fn check_rollup_app_key(item: &Value) -> Option<String> {
    let app = item
        .pointer("/checkSuite/app")
        .or_else(|| item.get("app"))?;
    app_identity(app)
}

fn app_identity(app: &Value) -> Option<String> {
    app_identifier(app, "databaseId")
        .map(|value| format!("databaseId:{value}"))
        .or_else(|| app_identifier(app, "id").map(|value| format!("id:{value}")))
        .or_else(|| app_identifier(app, "slug").map(|value| format!("slug:{value}")))
        .or_else(|| app_identifier(app, "name").map(|value| format!("name:{value}")))
}

fn app_identifier(app: &Value, key: &str) -> Option<String> {
    let value = app.get(key)?;
    if let Some(text) = value.as_str() {
        let text = public_text(text, 120);
        return (!text.trim().is_empty()).then_some(text);
    }
    value
        .as_i64()
        .map(|number| number.to_string())
        .or_else(|| value.as_u64().map(|number| number.to_string()))
}

fn check_rollup_state(item: &Value) -> CheckRollupState {
    if let Some(state) = item.get("state").and_then(Value::as_str) {
        return match state {
            "SUCCESS" => CheckRollupState::Passed,
            "FAILURE" | "ERROR" => CheckRollupState::Failed,
            _ => CheckRollupState::Pending,
        };
    }

    let status = item
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let conclusion = item
        .get("conclusion")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if status != "COMPLETED" {
        CheckRollupState::Pending
    } else if matches!(conclusion, "SUCCESS" | "SKIPPED" | "NEUTRAL") {
        CheckRollupState::Passed
    } else if matches!(
        conclusion,
        "FAILURE" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED"
    ) {
        CheckRollupState::Failed
    } else {
        CheckRollupState::Pending
    }
}

fn check_rollup_sort_key(item: &Value) -> String {
    [
        "completedAt",
        "updatedAt",
        "startedAt",
        "createdAt",
        "completed_at",
        "updated_at",
        "started_at",
        "created_at",
    ]
    .iter()
    .find_map(|key| item.get(*key).and_then(Value::as_str))
    .unwrap_or_default()
    .to_string()
}
