use std::fs;
use std::path::Path;

use crate::governance_audit_report::GovernanceFinding;
use crate::governance_audit_support::{error, package_scripts, read_to_string, warning};
use crate::governance_profile::GovernanceRepoProfile;
use crate::governance_secret_scan::{governance_config_files, looks_secret_like};

pub(crate) fn static_findings(
    workspace: &Path,
    profile: GovernanceRepoProfile,
) -> Vec<GovernanceFinding> {
    let mut findings = Vec::new();
    required_file(&mut findings, workspace, "AGENTS.md", "agent-instructions");
    required_file(
        &mut findings,
        workspace,
        "docs/architecture-contract.md",
        "architecture-contract",
    );
    required_file(&mut findings, workspace, "docs/build-plan.md", "build-plan");
    required_file(&mut findings, workspace, "README.md", "public-docs");
    required_file(&mut findings, workspace, "llms.txt", "public-docs");
    required_file(&mut findings, workspace, ".env.example", "environment");
    required_file(
        &mut findings,
        workspace,
        "package-lock.json",
        "dependencies",
    );
    if profile.requires_cargo_lock() {
        required_file(&mut findings, workspace, "Cargo.lock", "dependencies");
    }
    required_file(
        &mut findings,
        workspace,
        ".github/dependabot.yml",
        "dependencies",
    );
    required_file(&mut findings, workspace, ".github/workflows/ci.yml", "ci");

    check_package_scripts(&mut findings, workspace, profile);
    check_workflows(&mut findings, workspace, profile);
    check_memory_policy(&mut findings, workspace);
    check_secret_files(&mut findings, workspace);
    findings
}

fn required_file(
    findings: &mut Vec<GovernanceFinding>,
    workspace: &Path,
    relative: &str,
    category: &str,
) {
    let path = workspace.join(relative);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() && metadata.len() > 0 => {}
        _ => findings.push(error(
            category,
            "GOV_REQUIRED_FILE_MISSING",
            &format!("{relative} is missing or empty."),
            relative,
            "Restore the required governance artifact before release.",
        )),
    }
}

fn check_package_scripts(
    findings: &mut Vec<GovernanceFinding>,
    workspace: &Path,
    profile: GovernanceRepoProfile,
) {
    let scripts = package_scripts(workspace).unwrap_or_default();
    let mut required_scripts = vec!["release:check", "typecheck", "test", "build", "docs:build"];
    if profile.requires_rust_script() {
        required_scripts.push("rust:check");
    }
    if profile.requires_tui_live_qa() {
        required_scripts.push("tui:live-qa");
    }

    for script in required_scripts {
        if !scripts.contains_key(script) {
            findings.push(error(
                "release-readiness",
                "GOV_PACKAGE_SCRIPT_MISSING",
                &format!("package.json is missing the {script} script."),
                "package.json",
                "Add the script or update the audit policy with the replacement command.",
            ));
        }
    }

    if scripts
        .get("release:check")
        .is_some_and(|command| profile.requires_check_v10() && !command.contains("check:v10"))
    {
        findings.push(warning(
            "release-readiness",
            "GOV_RELEASE_GATE_WEAK",
            "release:check does not mention the staged readiness gate.",
            "package.json scripts.release:check",
            "Keep npm run release:check as the clean-checkout release gate.",
        ));
    }
}

fn check_workflows(
    findings: &mut Vec<GovernanceFinding>,
    workspace: &Path,
    profile: GovernanceRepoProfile,
) {
    let ci = read_to_string(workspace, ".github/workflows/ci.yml");
    let mut ci_commands = vec!["npm run typecheck", "npm test", "npm run build"];
    if profile.requires_rust_script() {
        ci_commands.insert(0, "npm run rust:check");
    }
    for command in ci_commands {
        if !ci.contains(command) {
            findings.push(warning(
                "ci",
                "GOV_CI_CHECK_MISSING",
                &format!("CI workflow does not mention {command}."),
                ".github/workflows/ci.yml",
                "Keep deterministic CI checks visible in the verify workflow.",
            ));
        }
    }

    let publish = read_to_string(workspace, ".github/workflows/npm-publish.yml");
    if profile.requires_npm_publish_workflow() && !publish.contains("npm run release:check") {
        findings.push(error(
            "release-readiness",
            "GOV_RELEASE_GATE_NOT_ENFORCED",
            "The npm publish workflow does not run npm run release:check.",
            ".github/workflows/npm-publish.yml",
            "Run the clean-checkout release gate before package publication.",
        ));
    }

    let live_qa = read_to_string(workspace, ".github/workflows/tui-live-qa.yml");
    if profile.requires_tui_live_qa() && !live_qa.contains("npm run tui:live-qa") {
        findings.push(warning(
            "qa",
            "GOV_TUI_LIVE_QA_MISSING",
            "TUI live QA workflow does not run npm run tui:live-qa.",
            ".github/workflows/tui-live-qa.yml",
            "Keep smoke evidence separate from deterministic release gates.",
        ));
    }
}

fn check_memory_policy(findings: &mut Vec<GovernanceFinding>, workspace: &Path) {
    let agents = read_to_string(workspace, "AGENTS.md").to_ascii_lowercase();
    for phrase in [
        "do not store secrets",
        "raw conversation logs",
        "transient task",
    ] {
        if !agents.contains(phrase) {
            findings.push(warning(
                "memory",
                "GOV_MEMORY_POLICY_GAP",
                &format!("AGENTS.md does not mention '{phrase}'."),
                "AGENTS.md",
                "Keep the memory policy explicit about durable context and prohibited data.",
            ));
        }
    }
}

fn check_secret_files(findings: &mut Vec<GovernanceFinding>, workspace: &Path) {
    for path in governance_config_files(workspace) {
        let relative = path
            .strip_prefix(workspace)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        let content = fs::read_to_string(&path).unwrap_or_default();
        if looks_secret_like(&content) {
            findings.push(error(
                "security",
                "GOV_SECRET_SHAPED_CONFIG",
                &format!("{relative} contains secret-shaped text."),
                &relative,
                "Remove real credentials and use environment-variable placeholders.",
            ));
        }
    }
}
