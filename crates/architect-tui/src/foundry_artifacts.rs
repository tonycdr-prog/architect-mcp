use serde_json::Value;

use crate::foundry::FoundryArtifact;
use crate::session::TuiSession;

pub(crate) fn artifact_plan(session: &TuiSession, verification: &[String]) -> Vec<FoundryArtifact> {
    let mut artifacts = vec![
        artifact(
            "AGENTS.md",
            "agent harness",
            "repo-local agent instructions",
        ),
        artifact(
            "README.md",
            "public docs",
            "install, run, and project overview",
        ),
        artifact(
            ".env.example",
            "config hygiene",
            "commit-safe environment template",
        ),
        artifact(
            "docs/architecture-contract.md",
            "create_pre_edit_contract",
            "governed architecture contract",
        ),
        artifact(
            "docs/build-plan.md",
            "review_build_plan",
            "ordered implementation slices",
        ),
        artifact(
            ".github/workflows/ci.yml",
            "release gate",
            "executable verification checks only",
        ),
        artifact(
            ".github/ISSUE_TEMPLATE/bug_report.md",
            "repo hygiene",
            "structured bug reports",
        ),
        artifact(
            ".github/pull_request_template.md",
            "work gate evidence",
            "verification, assumptions, and remaining gaps",
        ),
    ];
    if needs_npm_test_scaffold(verification) {
        artifacts.push(artifact(
            "package.json",
            "verification scaffold",
            "minimal npm test command for generated repo CI",
        ));
        artifacts.push(artifact(
            "tests/foundry-scaffold.test.mjs",
            "verification scaffold",
            "checks generated guardrail artifacts exist",
        ));
    }
    if has_database(&session.brief) {
        artifacts.push(artifact(
            "docs/data-boundary.md",
            "brief",
            "database ownership, migrations, and server boundary",
        ));
    }
    artifacts
}

fn artifact(path: &str, source: &str, purpose: &str) -> FoundryArtifact {
    FoundryArtifact {
        path: path.to_string(),
        source: source.to_string(),
        purpose: purpose.to_string(),
        required: true,
    }
}

fn needs_npm_test_scaffold(verification: &[String]) -> bool {
    verification.iter().any(|check| check.trim() == "npm test")
}

fn has_database(brief: &Value) -> bool {
    brief
        .get("stack")
        .and_then(|stack| stack.get("database"))
        .and_then(Value::as_str)
        .is_some_and(|database| {
            let database = database.trim();
            !database.is_empty() && !database.eq_ignore_ascii_case("none")
        })
}
