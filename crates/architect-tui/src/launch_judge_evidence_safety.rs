use serde_json::Value;

pub(crate) fn collect_unsafe_content(value: &Value, path: &str, issues: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let child_path = format!("{path}.{key}");
                if risky_key(key) {
                    issues.push(format!(
                        "terminal evidence contains raw or private field at {child_path}"
                    ));
                }
                collect_unsafe_content(value, &child_path, issues);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_unsafe_content(value, &format!("{path}[{index}]"), issues);
            }
        }
        Value::String(text) => {
            if text.len() > 2_000 {
                issues.push(format!(
                    "terminal evidence string at {path} is too long for public-safe summary"
                ));
            }
            if text.lines().count() > 8 {
                issues.push(format!(
                    "terminal evidence string at {path} looks like raw multiline output"
                ));
            }
            if contains_sensitive_text(text) {
                issues.push(format!(
                    "terminal evidence string at {path} contains secret-shaped or local-path content"
                ));
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn risky_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "stdout"
            | "stderr"
            | "raw"
            | "log"
            | "logs"
            | "rawlog"
            | "rawlogs"
            | "localpath"
            | "privatepath"
            | "privaterepo"
    ) || key.contains("stdout")
        || key.contains("stderr")
        || key.contains("raw_log")
        || key.contains("rawlog")
        || key.contains("local_path")
        || key.contains("localpath")
        || key.contains("private_repo")
        || key.contains("privaterepo")
}

fn contains_sensitive_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains("/Users/")
        || text.contains("C:\\Users\\")
        || text.contains("C:/Users/")
        || lower.contains("/home/")
        || text.contains("BEGIN PRIVATE KEY")
        || lower.contains("npm_")
        || lower.contains("ghp_")
        || lower.contains("github_pat_")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
}
