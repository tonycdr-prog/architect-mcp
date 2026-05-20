use architect_tui::config::TuiConfig;
use architect_tui::foundry_audit::{
    FoundryAuditOptions, FoundryAuditStatus, build_foundry_audit_report,
};
use architect_tui::interactive::InteractiveWorkflowEngine;
use architect_tui::orchestrator::Orchestrator;
use std::path::{Path, PathBuf};
use std::process::Command;

#[tokio::test]
async fn foundry_audit_chains_mcp_tools_and_leaves_git_status_clean() {
    let Some((workspace, config, _temp)) = fake_mcp_workspace() else {
        return;
    };
    if !git_available() {
        return;
    }
    init_clean_git_repo(&workspace);

    let report = build_foundry_audit_report(
        workspace.clone(),
        config,
        &FoundryAuditOptions {
            json: true,
            public_summary: false,
            max_files: 50,
            mcp_workspace: None,
        },
    )
    .await;

    assert!(report.read_only);
    assert_eq!(report.server_writes_performed, 0);
    assert_eq!(report.status, FoundryAuditStatus::PassedWithWarnings);
    assert_eq!(
        report.mcp.tools_called,
        [
            "review_local_workspace",
            "derive_local_repo_constitution",
            "normalize_foundry_evidence",
            "score_foundry_actionability",
            "route_foundry_decisions",
            "forge_foundry_previews",
        ]
    );
    assert_eq!(report.evidence.total_evidence, 2);
    assert_eq!(report.ledger.by_route.get("pr_preview"), Some(&1));
    assert_eq!(report.forge.pull_request_previews, 1);
    assert_eq!(report.decisions.len(), 1);
    assert_eq!(report.decisions[0].route, "pr_preview");
    assert_eq!(report.decisions[0].evidence_count, 2);
    assert_eq!(report.decisions[0].risk, "redacted");
    assert_eq!(report.decisions[0].approval_state, "approval_required");
    assert!(
        report.decisions[0]
            .next_action
            .contains("[redacted-local-path]")
    );
    assert!(
        report.decisions[0]
            .next_action
            .contains("[redacted-secret]")
    );
    assert_eq!(git_status(&workspace), "");
    assert!(!workspace.join(".architect-mcp").exists());
}

#[tokio::test]
async fn interactive_foundry_audit_renders_ephemeral_ledger_without_approval() {
    let Some((workspace, config, _temp)) = fake_mcp_workspace() else {
        return;
    };
    let orchestrator = Orchestrator::new(workspace.clone(), config);
    let mut engine = InteractiveWorkflowEngine::new(orchestrator);

    let update = engine
        .apply_input("foundry audit")
        .await
        .expect("foundry audit");
    let transcript = update.transcript.join("\n");
    let inspector = update.inspector.join("\n");

    assert!(transcript.contains("foundry audit:"));
    assert!(transcript.contains("read-only: true"));
    assert!(transcript.contains("pr_preview"));
    assert!(transcript.contains("evidence=2"));
    assert!(transcript.contains("forge previews: total=1"));
    assert!(transcript.contains("preview=pull_request"));
    assert!(inspector.contains("foundry audit:"));
    assert!(inspector.contains("foundry ledger decisions: 1"));
    assert!(update.session.is_none());
    assert!(!workspace.join(".architect-mcp").exists());

    let ledger = engine
        .apply_input("foundry ledger")
        .await
        .expect("foundry ledger");
    assert!(
        ledger
            .transcript
            .join("\n")
            .contains("approval=approval_required")
    );
    assert!(
        ledger
            .transcript
            .join("\n")
            .contains("preview=pull_request")
    );
}

fn fake_mcp_workspace() -> Option<(PathBuf, TuiConfig, tempfile::TempDir)> {
    if Command::new("node").arg("--version").output().is_err() {
        return None;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("repo");
    std::fs::create_dir_all(workspace.join("src")).expect("workspace");
    std::fs::write(workspace.join("README.md"), "# fake\n").expect("readme");
    std::fs::write(workspace.join("src").join("main.rs"), "fn main() {}\n").expect("main");
    let server_path = temp.path().join("fake-foundry-mcp.mjs");
    std::fs::write(&server_path, fake_mcp_server_script()).expect("fake server");
    let mut config = TuiConfig::default();
    config.architect_mcp.command = Some("node".to_string());
    config.architect_mcp.args = vec![server_path.display().to_string()];
    Some((workspace, config, temp))
}

fn fake_mcp_server_script() -> &'static str {
    r#"
import readline from 'node:readline';
const rl = readline.createInterface({ input: process.stdin });
function respond(id, result) {
  console.log(JSON.stringify({ jsonrpc: '2.0', id, result }));
}
function tool(id, value) {
  respond(id, { content: [{ type: 'text', text: JSON.stringify(value) }], structuredContent: value });
}
rl.on('line', (line) => {
  const msg = JSON.parse(line);
  if (!msg.id) return;
  if (msg.method === 'initialize') {
    respond(msg.id, { protocolVersion: '2025-11-25', capabilities: {}, serverInfo: { name: 'fake', version: '0.0.0' } });
    return;
  }
  if (msg.method !== 'tools/call') {
    respond(msg.id, {});
    return;
  }
  const name = msg.params.name;
  if (name === 'review_local_workspace') {
    tool(msg.id, {
      filesReviewed: 12,
      scan: { truncated: false },
      summary: { errors: 0, warnings: 1 },
      report: { gate: { status: 'warn' } },
      violations: [{ severity: 'warning', code: 'ARCH_TEST', path: 'src/main.rs' }]
    });
  } else if (name === 'derive_local_repo_constitution') {
    tool(msg.id, {
      filesReviewed: 12,
      constitution: {
        schemaVersion: 1,
        summary: { hardSignals: 2, advisorySignals: 1, warnings: 0, missingRecommendedSignals: [], primaryLanguages: ['Rust'], packageManagers: ['cargo'], repoShape: 'rust_cli' },
        pullRequests: { templates: [{ path: '.github/pull_request_template.md', contentProvided: true, hiddenCommentOnly: false, headings: ['Summary', 'Verification'], checklistItems: 1, mentionsLinkedIssues: false, mentionsReleaseNotes: false, mentionsVerification: true }], recentStyle: { advisory: true, sampleSize: 1, acceptedSamples: 1, maintainerAuthoredSamples: 1, botSamplesIgnored: 0, nonMergedOrUnknownSamplesIgnored: 0, commonHeadings: [], checklistObserved: true, releaseNoteObserved: false, linkedIssueObserved: false }, precedence: [] },
        ci: { workflows: [{ path: '.github/workflows/ci.yml', name: 'CI', triggers: ['pull_request'] }], labelerConfigPaths: [] },
        release: { changelogPaths: [], releaseWorkflowPaths: [], releaseDocPaths: [] },
        packageMetadata: [],
        maintainerConstraints: [],
        findings: [],
        provenance: [],
        publicSafety: { rawContentIncluded: false, mutationAllowed: false }
      }
    });
  } else if (name === 'normalize_foundry_evidence') {
    tool(msg.id, {
      inventory: {
        schemaVersion: 1,
        summary: { totalEvidence: 2, bySourceType: { architect_review: 1 }, byConfidence: { high: 1 }, byPublicSafetyClass: { redacted: 1 }, redacted: 1, omittedRawPayloads: 1, suppressionCandidates: 0, coverageCaveats: 0 },
        evidence: [],
        coverage: { scanTruncated: false, detailedFindingsTruncated: false, filesReviewed: 12, maxFiles: 50, topScannedDirectories: [], findingHistogram: [], caveats: [] },
        suppressionPrerequisites: [],
        publicSafety: { rawPayloadsIncluded: false, rawRepoContentIncluded: false, mutationAllowed: false }
      }
    });
  } else if (name === 'score_foundry_actionability') {
    tool(msg.id, {
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 1, byDecision: { pr_preview_candidate: 1 }, prPreviewCandidates: 1, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 1 },
        assessments: [],
        publicSafety: { rawPayloadsIncluded: false, mutationAllowed: false, publicRecommendationsOnly: true }
      }
    });
  } else if (name === 'route_foundry_decisions') {
    tool(msg.id, {
      ledger: {
        schemaVersion: 1,
        ledgerId: 'fake-ledger',
        summary: { totalEntries: 1, byRoute: { pr_preview: 1 }, approvalRequired: 1, humanRequired: 0, publicSafetyHolds: 1, serverWritesPerformed: 0 },
        entries: [{
          id: 'fdl-001-pr_preview',
          route: 'pr_preview',
          evidenceIds: ['fde-001', 'fde-002'],
          source: { score: 76 },
          decisionReason: 'Route pr_preview at score 76.',
          redactionState: 'redacted',
          verificationRequirements: ['Run cargo test before mutation approval.'],
          approvalState: 'approval_required',
          nextAction: 'Draft a local PR preview from /Users/alice/worktree with ghp_123456; no mutation yet.',
          mutation: { serverMutationAllowed: false, explicitApprovalRequired: true }
        }],
        publicSafety: { rawPayloadsIncluded: false, rawRepoContentIncluded: false, localPathsIncluded: false, tokenValuesIncluded: false, mutationAllowed: false, publicLedgerOnly: true }
      }
    });
  } else if (name === 'forge_foundry_previews') {
    tool(msg.id, {
      forge: {
        schemaVersion: 1,
        summary: { totalDecisions: 1, previewsGenerated: 1, pullRequestPreviews: 1, architectIssuePreviews: 0, exceptionRecords: 0, noOpRecords: 0, humanQuestions: 0, skipped: 0, serverWritesPerformed: 0 },
        previews: [{ id: 'ffp-001-pull_request', kind: 'pull_request', sourceDecisionId: 'fdl-001-pr_preview', title: '[foundry-preview] redacted decision', warnings: ['Decision is redacted.'], mutation: { serverMutationAllowed: false, explicitApprovalRequired: true } }],
        publicSafety: { rawPayloadsIncluded: false, rawRepoContentIncluded: false, localPathsIncluded: false, tokenValuesIncluded: false, mutationAllowed: false, publicPreviewsOnly: true }
      }
    });
  } else {
    tool(msg.id, { ok: true });
  }
});
"#
}

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn init_clean_git_repo(workspace: &Path) {
    run_git(workspace, &["init"]);
    run_git(
        workspace,
        &["config", "user.email", "architect@example.invalid"],
    );
    run_git(workspace, &["config", "user.name", "Architect Test"]);
    run_git(workspace, &["add", "."]);
    run_git(workspace, &["commit", "-m", "baseline"]);
    assert_eq!(git_status(workspace), "");
}

fn git_status(workspace: &Path) -> String {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(workspace)
        .output()
        .expect("git status");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_git(workspace: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}
