use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

use crate::session::TuiSession;

pub(crate) fn recommendation_request(session: &TuiSession, context: &str) -> Value {
    let mut request = Map::new();
    let request_text = [session.prompt.as_str(), context.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    request.insert("request".to_string(), Value::String(request_text));
    request.insert(
        "projectBrief".to_string(),
        project_brief_for_recommendation(&session.brief),
    );
    if let Some(stack_packs) = selected_stack_packs(session) {
        request.insert("selectedStackPacks".to_string(), stack_packs);
    }
    Value::Object(request)
}

pub(crate) fn ensure_recommended(session: &TuiSession, server_id: &str) -> Result<()> {
    let recommendation = session
        .mcp_recommendation
        .as_ref()
        .context("run integrations recommend before creating an install plan")?;
    if recommendation.get("status").and_then(Value::as_str) == Some("needs-clarification") {
        anyhow::bail!("answer MCP recommendation questions and rerun integrations recommend");
    }
    let recommended = recommendation
        .get("recommendations")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .any(|item| item.get("serverId").and_then(Value::as_str) == Some(server_id))
        })
        .unwrap_or(false);
    if !recommended {
        anyhow::bail!(
            "MCP server '{server_id}' was not recommended; clarify provider or boundary first"
        );
    }
    Ok(())
}

pub(crate) fn ensure_install_review_not_failed(session: &TuiSession) -> Result<()> {
    if session.mcp_install_plan.is_none() {
        anyhow::bail!("create an MCP install plan first");
    }
    let review = session
        .mcp_install_review
        .as_ref()
        .context("review MCP install plan before apply or approval")?;
    if review.get("status").and_then(Value::as_str) == Some("fail") {
        anyhow::bail!("MCP install review failed; regenerate a safe install plan");
    }
    Ok(())
}

pub(crate) fn apply_args(
    plan: Value,
    target_path: Option<&str>,
    write_files: bool,
    approved: bool,
) -> Value {
    let mut request = Map::new();
    request.insert("plan".to_string(), plan);
    if let Some(path) = target_path.filter(|path| !path.trim().is_empty()) {
        request.insert(
            "targetPath".to_string(),
            Value::String(path.trim().to_string()),
        );
    }
    request.insert("writeFiles".to_string(), Value::Bool(write_files));
    request.insert("explicitApproval".to_string(), Value::Bool(approved));
    json!({ "request": Value::Object(request) })
}

pub(crate) fn recommendation_lines(result: Option<&Value>) -> Vec<String> {
    let Some(result) = result else {
        return vec!["MCP recommendations: none".to_string()];
    };
    let mut lines = vec![format!(
        "MCP recommendations: {}",
        result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )];
    if let Some(questions) = result.get("questions").and_then(Value::as_array) {
        lines.extend(
            questions
                .iter()
                .filter_map(Value::as_str)
                .map(|question| format!("question: {question}")),
        );
    }
    if let Some(recommendations) = result.get("recommendations").and_then(Value::as_array) {
        lines.extend(recommendations.iter().filter_map(|item| {
            let id = item.get("serverId").and_then(Value::as_str)?;
            let confidence = item
                .get("confidence")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            Some(format!("recommended: {id} ({confidence})"))
        }));
    }
    lines
}

pub(crate) fn plan_lines(plan: Option<&Value>) -> Vec<String> {
    let Some(plan) = plan else {
        return vec!["MCP install plan: none".to_string()];
    };
    vec![
        format!(
            "MCP install plan: {}",
            plan.get("serverId")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
        format!(
            "target: {}",
            plan.get("targetClient")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
        format!(
            "package: {}",
            plan.get("packagePin")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
        "next: integrations review".to_string(),
    ]
}

pub(crate) fn review_lines(review: Option<&Value>) -> Vec<String> {
    let Some(review) = review else {
        return vec!["MCP install review: none".to_string()];
    };
    let mut lines = vec![format!(
        "MCP install review: {}",
        review
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )];
    if let Some(findings) = review.get("findings").and_then(Value::as_array) {
        lines.extend(findings.iter().filter_map(|finding| {
            let code = finding.get("code").and_then(Value::as_str)?;
            let severity = finding
                .get("severity")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            Some(format!("finding: {severity} {code}"))
        }));
    }
    lines.push("next: integrations apply, then integrations approve before write".to_string());
    lines
}

pub(crate) fn apply_lines(result: Option<&Value>, wrote: bool) -> Vec<String> {
    let Some(result) = result else {
        return vec!["MCP install apply: none".to_string()];
    };
    let mut lines = vec![format!(
        "MCP install apply: {}",
        result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )];
    if let Some(path) = result.get("targetPath").and_then(Value::as_str) {
        lines.push(format!("target: {path}"));
    }
    if !wrote {
        lines.push("dry-run only; no files written".to_string());
    }
    lines
}

fn project_brief_for_recommendation(brief: &Value) -> Value {
    let mut project = Map::new();
    copy_field(brief, &mut project, "idea");
    copy_field(brief, &mut project, "users");
    copy_field(brief, &mut project, "coreFlows");
    copy_field(brief, &mut project, "stack");
    copy_field(brief, &mut project, "constraints");
    Value::Object(project)
}

fn copy_field(source: &Value, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_string(), value.clone());
    }
}

fn selected_stack_packs(session: &TuiSession) -> Option<Value> {
    let packs = session
        .gates
        .get("grill_me")?
        .get("selectedStackPacks")?
        .as_array()?
        .iter()
        .filter_map(|pack| {
            pack.as_str().map(ToString::to_string).or_else(|| {
                pack.get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
        })
        .map(Value::String)
        .collect::<Vec<_>>();
    if packs.is_empty() {
        None
    } else {
        Some(Value::Array(packs))
    }
}
