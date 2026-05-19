use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::governance_audit_report::{
    GovernanceAuditCategory, GovernanceAuditStatus, GovernanceFinding, GovernanceFindingSeverity,
    GovernanceGateEvidence,
};

pub(crate) fn package_scripts(workspace: &Path) -> Option<BTreeMap<String, String>> {
    let package = fs::read_to_string(workspace.join("package.json")).ok()?;
    let value: Value = serde_json::from_str(&package).ok()?;
    let scripts = value.get("scripts")?.as_object()?;
    Some(
        scripts
            .iter()
            .filter_map(|(key, value)| {
                value
                    .as_str()
                    .map(|script| (key.clone(), script.to_string()))
            })
            .collect(),
    )
}

pub(crate) fn read_to_string(workspace: &Path, relative: &str) -> String {
    fs::read_to_string(workspace.join(relative)).unwrap_or_default()
}

pub(crate) fn governance_config_files(workspace: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_config_files(workspace, workspace, 0, &mut files);
    files
}

fn collect_config_files(root: &Path, current: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if ignored_dir(&name) {
                continue;
            }
            collect_config_files(root, &path, depth + 1, files);
        } else if is_sensitive_config_name(&name) {
            files.push(path);
        }
    }
    files.sort_by(|left, right| {
        let left = left.strip_prefix(root).unwrap_or(left);
        let right = right.strip_prefix(root).unwrap_or(right);
        left.cmp(right)
    });
}

fn ignored_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "dist" | ".next" | ".vitepress"
    )
}

fn is_sensitive_config_name(name: &str) -> bool {
    name == ".mcp.json" || (name.starts_with(".env") && name != ".env.example")
}

pub(crate) fn looks_secret_like(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    content.contains("npm_")
        || content.contains("ghp_")
        || content.contains("github_pat_")
        || content.contains("AKIA")
        || content.contains("BEGIN PRIVATE KEY")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
}

pub(crate) fn deterministic_gates(workspace: &Path) -> Vec<GovernanceGateEvidence> {
    let scripts = package_scripts(workspace).unwrap_or_default();
    [
        ("clean checkout release gate", "npm run release:check", true),
        ("rust gate", "npm run rust:check", false),
        ("typecheck", "npm run typecheck", false),
        ("unit tests", "npm test", false),
        ("build", "npm run build", false),
        ("docs build", "npm run docs:build", false),
    ]
    .into_iter()
    .filter_map(|(name, command, required)| {
        let script_name = command.strip_prefix("npm run ").unwrap_or("test");
        let exists = if command == "npm test" {
            scripts.contains_key("test")
        } else {
            scripts.contains_key(script_name)
        };
        exists.then(|| GovernanceGateEvidence {
            name: name.to_string(),
            kind: "deterministic".to_string(),
            command: command.to_string(),
            required_for_release: required,
            source: "package.json".to_string(),
        })
    })
    .collect()
}

pub(crate) fn smoke_evidence(workspace: &Path) -> Vec<GovernanceGateEvidence> {
    let scripts = package_scripts(workspace).unwrap_or_default();
    let mut evidence = vec![
        GovernanceGateEvidence {
            name: "terminal smoke".to_string(),
            kind: "smoke".to_string(),
            command: "architect-mcp-tui smoke --json".to_string(),
            required_for_release: false,
            source: "operator evidence".to_string(),
        },
        GovernanceGateEvidence {
            name: "scripted walkthrough".to_string(),
            kind: "smoke".to_string(),
            command: "architect-mcp-tui walkthrough --json".to_string(),
            required_for_release: false,
            source: "operator evidence".to_string(),
        },
    ];
    if scripts.contains_key("tui:live-qa") {
        evidence.push(GovernanceGateEvidence {
            name: "cross-platform TUI live QA".to_string(),
            kind: "smoke".to_string(),
            command: "npm run tui:live-qa".to_string(),
            required_for_release: false,
            source: "package.json".to_string(),
        });
    }
    evidence
}

pub(crate) fn categories(findings: &[GovernanceFinding]) -> Vec<GovernanceAuditCategory> {
    let mut names: BTreeSet<String> = [
        "agent-instructions",
        "architecture-contract",
        "build-plan",
        "ci",
        "dependencies",
        "environment",
        "memory",
        "mcp-review",
        "public-docs",
        "qa",
        "release-readiness",
        "security",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect();
    names.extend(findings.iter().map(|finding| finding.category.clone()));
    names
        .into_iter()
        .map(|name| {
            let related: Vec<&GovernanceFinding> = findings
                .iter()
                .filter(|finding| finding.category == name)
                .collect();
            let status = status_for_findings_refs(&related);
            GovernanceAuditCategory {
                name,
                status,
                summary: if related.is_empty() {
                    "no findings".to_string()
                } else {
                    format!("{} finding(s)", related.len())
                },
            }
        })
        .collect()
}

pub(crate) fn status_for_findings(findings: &[GovernanceFinding]) -> GovernanceAuditStatus {
    if findings
        .iter()
        .any(|finding| finding.severity == GovernanceFindingSeverity::Error)
    {
        GovernanceAuditStatus::Failed
    } else if findings
        .iter()
        .any(|finding| finding.severity == GovernanceFindingSeverity::Warning)
    {
        GovernanceAuditStatus::PassedWithWarnings
    } else {
        GovernanceAuditStatus::Passed
    }
}

fn status_for_findings_refs(findings: &[&GovernanceFinding]) -> GovernanceAuditStatus {
    if findings
        .iter()
        .any(|finding| finding.severity == GovernanceFindingSeverity::Error)
    {
        GovernanceAuditStatus::Failed
    } else if findings
        .iter()
        .any(|finding| finding.severity == GovernanceFindingSeverity::Warning)
    {
        GovernanceAuditStatus::PassedWithWarnings
    } else {
        GovernanceAuditStatus::Passed
    }
}

pub(crate) fn error(
    category: &str,
    code: &str,
    message: &str,
    evidence: &str,
    next_action: &str,
) -> GovernanceFinding {
    finding(
        GovernanceFindingSeverity::Error,
        category,
        code,
        message,
        evidence,
        next_action,
    )
}

pub(crate) fn warning(
    category: &str,
    code: &str,
    message: &str,
    evidence: &str,
    next_action: &str,
) -> GovernanceFinding {
    finding(
        GovernanceFindingSeverity::Warning,
        category,
        code,
        message,
        evidence,
        next_action,
    )
}

fn finding(
    severity: GovernanceFindingSeverity,
    category: &str,
    code: &str,
    message: &str,
    evidence: &str,
    next_action: &str,
) -> GovernanceFinding {
    GovernanceFinding {
        severity,
        category: category.to_string(),
        code: code.to_string(),
        message: message.to_string(),
        evidence: evidence.to_string(),
        next_action: next_action.to_string(),
    }
}
