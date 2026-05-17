use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

use crate::session::TuiSession;

pub(crate) fn recommendation_request(session: &TuiSession, context: &str) -> Value {
    let mut request = Map::new();
    if !context.trim().is_empty() {
        request.insert(
            "request".to_string(),
            Value::String(context.trim().to_string()),
        );
    }
    request.insert(
        "projectBrief".to_string(),
        project_brief_for_recommendation(&session.brief),
    );
    if let Some(stack_packs) = selected_stack_packs(session) {
        request.insert("selectedStackPacks".to_string(), stack_packs);
    }
    if let Some(providers) = confirmed_providers(&session.brief) {
        request.insert("confirmedProviders".to_string(), providers);
    }
    if server_side_boundary_confirmed(&session.brief) {
        request.insert("serverSideBoundaryConfirmed".to_string(), Value::Bool(true));
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

pub(crate) fn ensure_install_review_passed(session: &TuiSession) -> Result<()> {
    if session.mcp_install_plan.is_none() {
        anyhow::bail!("create an MCP install plan first");
    }
    let review = session
        .mcp_install_review
        .as_ref()
        .context("review MCP install plan before apply or approval")?;
    match review.get("status").and_then(Value::as_str) {
        Some("pass") => Ok(()),
        Some("fail") => anyhow::bail!("MCP install review failed; regenerate a safe install plan"),
        Some(status) => {
            anyhow::bail!("MCP install review is not approved for apply/write: {status}")
        }
        None => {
            anyhow::bail!("MCP install review is missing a pass status; rerun integrations review")
        }
    }
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

fn confirmed_providers(brief: &Value) -> Option<Value> {
    let mut providers = Vec::new();
    if let Some(stack) = brief.get("stack").and_then(Value::as_object) {
        for key in ["database", "auth", "backend", "deployment", "storage"] {
            if let Some(provider) = stack.get(key).and_then(Value::as_str) {
                let provider = provider.trim();
                if !provider.is_empty() {
                    providers.push(Value::String(provider.to_string()));
                }
            }
        }
    }
    providers.sort_by(|left, right| {
        left.as_str()
            .unwrap_or_default()
            .cmp(right.as_str().unwrap_or_default())
    });
    providers.dedup();
    if providers.is_empty() {
        None
    } else {
        Some(Value::Array(providers))
    }
}

fn server_side_boundary_confirmed(brief: &Value) -> bool {
    let mut fields = Vec::new();
    if let Some(value) = brief.get("constraints").and_then(Value::as_array) {
        fields.extend(value.iter().filter_map(Value::as_str));
    }
    if let Some(value) = brief.get("coreFlows").and_then(Value::as_array) {
        fields.extend(value.iter().filter_map(Value::as_str));
    }
    if let Some(value) = brief.get("stack") {
        fields.extend(
            value
                .as_object()
                .into_iter()
                .flat_map(|stack| stack.values())
                .filter_map(Value::as_str),
        );
    }
    fields.iter().any(|value| {
        let normalized = value.to_ascii_lowercase();
        [
            "server-side",
            "server side",
            "backend",
            "api route",
            "webhook",
            "edge function",
            "cloud function",
            "secret boundary",
            "server-owned",
            "server owned",
        ]
        .iter()
        .any(|needle| normalized.contains(needle))
    })
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
