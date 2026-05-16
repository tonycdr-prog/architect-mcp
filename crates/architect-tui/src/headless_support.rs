use std::collections::BTreeMap;

use serde_json::{Value, json};

pub(crate) fn pre_edit_args(idea: &str, grill_value: &Value, verification: &[String]) -> Value {
    json!({
        "input": {
            "request": idea,
            "stack": grill_value
                .get("updatedBrief")
                .and_then(|brief| brief.get("stack"))
                .cloned()
                .unwrap_or_else(|| json!({})),
            "verification": verification
        }
    })
}

pub(crate) fn string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn verification_checks(grill_value: &Value, build_plan: &Value) -> Vec<String> {
    let mut checks = string_array(
        grill_value.get("updatedBrief").unwrap_or(&Value::Null),
        "verification",
    );
    if checks.is_empty() {
        checks.extend(
            build_plan
                .get("slices")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .flat_map(|slice| {
                    slice
                        .get("checks")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                })
                .filter_map(Value::as_str)
                .map(ToString::to_string),
        );
    }
    checks.sort();
    checks.dedup();
    checks
}

pub(crate) fn likely_files(build_plan: &Value) -> Vec<String> {
    let mut files = build_plan
        .get("slices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|slice| {
            slice
                .get("files")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

pub(crate) fn proposed_file_plan(grill_value: &Value) -> Value {
    let mut files: BTreeMap<String, Value> = BTreeMap::new();
    collect_scaffold_files(grill_value, &mut files);
    collect_artifact_files(grill_value, &mut files);
    if files.is_empty() {
        files.insert(
            "docs/architecture-contract.md".to_string(),
            json!({
                "path": "docs/architecture-contract.md",
                "purpose": "Document the pre-edit architecture contract.",
                "responsibilities": ["architecture contract"]
            }),
        );
    }
    json!({ "files": files.into_values().collect::<Vec<_>>() })
}

fn collect_scaffold_files(grill_value: &Value, files: &mut BTreeMap<String, Value>) {
    for item in grill_value
        .get("scaffoldPlan")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(path) = item.get("path").and_then(Value::as_str) else {
            continue;
        };
        if item.get("action").and_then(Value::as_str) == Some("create-directory") {
            continue;
        }
        let purpose = item
            .get("rationale")
            .and_then(Value::as_str)
            .unwrap_or("Generated scaffold artifact.");
        let responsibility = item
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("create-file");
        files.insert(
            path.to_string(),
            json!({
                "path": path,
                "purpose": purpose,
                "responsibilities": [responsibility]
            }),
        );
    }
}

fn collect_artifact_files(grill_value: &Value, files: &mut BTreeMap<String, Value>) {
    for artifact in grill_value
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(path) = artifact.get("path").and_then(Value::as_str) else {
            continue;
        };
        let purpose = artifact
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("Generated repo artifact.");
        files.entry(path.to_string()).or_insert_with(|| {
            json!({
                "path": path,
                "purpose": purpose,
                "responsibilities": ["repo artifact"]
            })
        });
    }
}
