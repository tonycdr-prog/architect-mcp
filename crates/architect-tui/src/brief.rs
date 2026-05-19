use serde_json::{Map, Value, json};

pub(crate) fn brief_from_prompt(prompt: &str) -> Value {
    let mut brief = json!({ "idea": prompt.trim() });
    for (key, value) in labeled_segments(prompt) {
        apply_brief_answer(&mut brief, &key, &value);
    }
    brief
}

pub(crate) fn apply_brief_answer(brief: &mut Value, key: &str, value: &str) {
    let Some(field) = canonical_key(key) else {
        return;
    };
    match field.as_str() {
        "idea" | "users" | "storage" | "enforcement" | "risk" => {
            append_or_set_text(brief, &field, value);
        }
        "coreFlows" | "verification" | "constraints" | "dataEntities" => {
            brief[&field] = json!(split_list(value));
        }
        "stack" => apply_stack_answer(brief, value),
        field if field.starts_with("stack.") => {
            let stack_field = field.trim_start_matches("stack.");
            ensure_object_field(brief, "stack")
                .insert(stack_field.to_string(), json!(trim_sentence_period(value)));
        }
        "repoLayout" => {
            brief["repoLayout"] = json!({ "pathMap": repo_layout_map(value) });
        }
        "job" => append_or_set_text(brief, "users", value),
        other => set_text_field(brief, other, value),
    }
}

pub(crate) fn answer_affects_mcp_recommendations(key: &str) -> bool {
    match canonical_key(key).as_deref() {
        Some("idea" | "users" | "job" | "coreFlows" | "stack" | "constraints") => true,
        Some(field) if field.starts_with("stack.") => true,
        _ => false,
    }
}

fn labeled_segments(input: &str) -> Vec<(String, String)> {
    let labels = [
        ("core flows", "coreFlows"),
        ("coreflows", "coreFlows"),
        ("repo layout", "repoLayout"),
        ("repolayout", "repoLayout"),
        ("data entities", "dataEntities"),
        ("dataentities", "dataEntities"),
        ("verification", "verification"),
        ("checks", "verification"),
        ("constraints", "constraints"),
        ("enforcement", "enforcement"),
        ("storage", "storage"),
        ("risks", "risk"),
        ("risk", "risk"),
        ("users", "users"),
        ("user", "users"),
        ("job", "job"),
        ("stack", "stack"),
        ("idea", "idea"),
    ];
    let mut hits = Vec::new();
    for (label, field) in labels {
        let marker = format!("{label}:");
        let mut offset = 0;
        while let Some(start) = find_ascii_marker(input, &marker, offset) {
            hits.push((start, start + marker.len(), field.to_string()));
            offset = start + marker.len();
        }
    }
    hits.sort_by_key(|(start, _, _)| *start);
    hits.dedup_by_key(|(start, _, _)| *start);

    let mut segments = Vec::new();
    for (idx, (_, value_start, field)) in hits.iter().enumerate() {
        let value_end = hits
            .get(idx + 1)
            .map(|(next_start, _, _)| *next_start)
            .unwrap_or(input.len());
        let value = input[*value_start..value_end].trim();
        if !value.is_empty() {
            segments.push((field.clone(), value.to_string()));
        }
    }
    segments
}

fn find_ascii_marker(input: &str, marker: &str, offset: usize) -> Option<usize> {
    input
        .get(offset..)?
        .char_indices()
        .map(|(relative_start, _)| offset + relative_start)
        .find(|start| starts_with_ascii_case_insensitive(&input[*start..], marker))
}

fn starts_with_ascii_case_insensitive(input: &str, marker: &str) -> bool {
    let bytes = input.as_bytes();
    let marker = marker.as_bytes();
    bytes.len() >= marker.len()
        && bytes[..marker.len()]
            .iter()
            .zip(marker)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn canonical_key(key: &str) -> Option<String> {
    let normalized = key
        .trim()
        .replace(['-', '_'], "")
        .replace(' ', "")
        .to_lowercase();
    match normalized.as_str() {
        "idea" => Some("idea".to_string()),
        "users" | "user" => Some("users".to_string()),
        "job" => Some("job".to_string()),
        "coreflows" | "flows" | "workflows" => Some("coreFlows".to_string()),
        "stack" => Some("stack".to_string()),
        "frontend" => Some("stack.frontend".to_string()),
        "backend" => Some("stack.backend".to_string()),
        "database" => Some("stack.database".to_string()),
        "auth" => Some("stack.auth".to_string()),
        "deployment" => Some("stack.deployment".to_string()),
        "storage" => Some("storage".to_string()),
        "enforcement" => Some("enforcement".to_string()),
        "repolayout" | "layout" => Some("repoLayout".to_string()),
        "risk" | "risks" => Some("risk".to_string()),
        "verification" | "checks" => Some("verification".to_string()),
        "constraints" => Some("constraints".to_string()),
        "dataentities" | "entities" => Some("dataEntities".to_string()),
        _ if normalized.starts_with("stack.") => {
            let stack_field = normalized.trim_start_matches("stack.");
            canonical_stack_field(stack_field).map(|field| format!("stack.{field}"))
        }
        _ => None,
    }
}

fn canonical_stack_field(field: &str) -> Option<&'static str> {
    match field {
        "frontend" => Some("frontend"),
        "backend" => Some("backend"),
        "database" => Some("database"),
        "auth" => Some("auth"),
        "deployment" => Some("deployment"),
        _ => None,
    }
}

fn apply_stack_answer(brief: &mut Value, value: &str) {
    let pairs = key_value_pairs(value);
    if pairs.is_empty() {
        ensure_object_field(brief, "stack").insert("backend".to_string(), json!(value.trim()));
        return;
    }
    for (key, value) in pairs {
        if let Some(field) = canonical_key(&key)
            && let Some(stack_field) = field.strip_prefix("stack.")
        {
            ensure_object_field(brief, "stack").insert(stack_field.to_string(), json!(value));
        }
    }
}

fn repo_layout_map(value: &str) -> Map<String, Value> {
    let pairs = key_value_pairs(value);
    if pairs.is_empty() {
        let mut map = Map::new();
        map.insert("existing".to_string(), json!(split_list(value)));
        return map;
    }
    pairs
        .into_iter()
        .map(|(key, value)| (key, json!(split_list(&value))))
        .collect()
}

fn key_value_pairs(value: &str) -> Vec<(String, String)> {
    split_list(value)
        .into_iter()
        .filter_map(|part| {
            let (key, value) = part.split_once('=').or_else(|| part.split_once(':'))?;
            let key = key.trim();
            let value = trim_sentence_period(value);
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key.to_string(), value.to_string()))
            }
        })
        .collect()
}

fn split_list(value: &str) -> Vec<String> {
    let delimiter = if value.contains('|') {
        '|'
    } else if value.contains(';') {
        ';'
    } else {
        '\n'
    };
    value
        .split(delimiter)
        .map(|item| {
            let item = item
                .trim()
                .trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.' || ch == ')')
                .trim();
            trim_sentence_period(item)
        })
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn trim_sentence_period(value: &str) -> &str {
    let value = value.trim();
    let Some(without_period) = value.strip_suffix('.') else {
        return value;
    };
    if without_period
        .chars()
        .last()
        .is_some_and(|ch| !ch.is_whitespace())
    {
        without_period.trim_end()
    } else {
        value
    }
}

fn ensure_object_field<'a>(value: &'a mut Value, key: &str) -> &'a mut Map<String, Value> {
    if !value.is_object() {
        *value = json!({});
    }
    if !value.get(key).is_some_and(Value::is_object) {
        value[key] = json!({});
    }
    value[key].as_object_mut().expect("object field")
}

fn append_or_set_text(brief: &mut Value, key: &str, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    if let Some(existing) = brief.get(key).and_then(Value::as_str)
        && !existing.trim().is_empty()
        && existing != value
    {
        brief[key] = json!(format!("{existing} {value}"));
        return;
    }
    set_text_field(brief, key, value);
}

fn set_text_field(brief: &mut Value, key: &str, value: &str) {
    if !brief.is_object() {
        *brief = json!({});
    }
    brief[key] = json!(value.trim());
}
