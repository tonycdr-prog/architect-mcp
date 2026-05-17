use serde_json::{Map, Value};

pub(crate) fn mcp_recommendation_summary(value: &Value) -> Value {
    let mut summary = Map::new();
    copy_field(value, &mut summary, "status");
    copy_field(value, &mut summary, "questions");
    copy_field(value, &mut summary, "policy");
    if let Some(recommendations) = value.get("recommendations").and_then(Value::as_array) {
        summary.insert(
            "recommendations".to_string(),
            Value::Array(
                recommendations
                    .iter()
                    .map(|item| {
                        let mut recommendation = Map::new();
                        for key in [
                            "serverId",
                            "name",
                            "provider",
                            "category",
                            "confidence",
                            "rationale",
                            "installMode",
                            "requiredEnv",
                            "warnings",
                        ] {
                            copy_field(item, &mut recommendation, key);
                        }
                        Value::Object(recommendation)
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(summary)
}

pub(crate) fn mcp_install_plan_summary(value: &Value) -> Value {
    let mut summary = Map::new();
    for key in [
        "id",
        "serverId",
        "serverName",
        "targetClient",
        "status",
        "hostedMode",
        "localOnly",
        "requiresApproval",
        "writeFiles",
        "packagePin",
        "env",
        "postInstall",
        "warnings",
    ] {
        copy_field(value, &mut summary, key);
    }
    Value::Object(summary)
}

pub(crate) fn mcp_install_review_summary(value: &Value) -> Value {
    let mut summary = Map::new();
    copy_field(value, &mut summary, "status");
    if let Some(findings) = value.get("findings").and_then(Value::as_array) {
        summary.insert(
            "findings".to_string(),
            Value::Array(
                findings
                    .iter()
                    .map(|finding| {
                        let mut sanitized = Map::new();
                        for key in ["code", "severity", "message", "recommendation"] {
                            copy_field(finding, &mut sanitized, key);
                        }
                        Value::Object(sanitized)
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(summary)
}

pub(crate) fn mcp_install_apply_summary(value: &Value) -> Value {
    let mut summary = Map::new();
    copy_field(value, &mut summary, "status");
    copy_field(value, &mut summary, "targetPath");
    if let Some(review) = value.get("review") {
        summary.insert("review".to_string(), mcp_install_review_summary(review));
    }
    Value::Object(summary)
}

fn copy_field(source: &Value, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_string(), value.clone());
    }
}
