import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";

describe("supply-chain and release hygiene", () => {
  it("ships license, security policy, env example, and portable MCP setup docs", () => {
    const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
      main: string;
      types: string;
      exports: Record<string, unknown>;
      files: string[];
      bin: Record<string, string>;
    };

    assert.equal(existsSync("LICENSE"), true);
    assert.equal(existsSync("SECURITY.md"), true);
    assert.equal(existsSync(".env.example"), true);
    assert.equal(packageJson.main, "./dist/library.js");
    assert.equal(packageJson.types, "./dist/library.d.ts");
    assert.equal(Object.hasOwn(packageJson.exports, "./http"), false);
    assert.equal(packageJson.files.includes("docs"), false);
    assert.equal(packageJson.files.includes("docs/**/*.md"), true);
    assert.equal(packageJson.files.includes("mcp-catalog"), true);
    assert.equal(packageJson.files.includes("bin/architect-mcp-tui.cjs"), true);
    assert.equal(packageJson.files.includes("crates/architect-tui"), true);
    assert.equal(packageJson.bin["architect-mcp-tui"], "./bin/architect-mcp-tui.cjs");
    assert.match(readFileSync(".gitignore", "utf8"), /!\.env\.example/);
    assert.doesNotMatch(readFileSync("README.md", "utf8"), /\/Users\/tonycordner/);
    assert.match(readFileSync("README.md", "utf8"), /npx/);
  });

  it("pins GitHub Actions to full commit SHAs and runs real checks", () => {
    const workflow = readFileSync(".github/workflows/ci.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    for (const command of ["rustup toolchain install 1.94.0", "npm run secret:scan", "npm run rust:check", "npm run typecheck", "npm test", "npm run build", "npm audit", "npm run pack:dry-run"]) {
      assert.match(workflow, new RegExp(command.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
    }
  });

  it("pins the GitHub Pages workflow and builds VitePress docs", () => {
    const workflow = readFileSync(".github/workflows/pages.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /npm run docs:build/);
    assert.match(workflow, /pages:\s+write/);
    assert.match(workflow, /id-token:\s+write/);
    assert.match(workflow, /docs\/\.vitepress\/dist/);
  });

  it("publishes npm only after the clean release gate and tarball smoke", () => {
    const workflow = readFileSync(".github/workflows/npm-publish.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /types:\s*\n\s+- published/);
    assert.match(workflow, /id-token:\s+write/);
    assert.match(workflow, /node-version: "24"/);
    assert.match(workflow, /npm install -g npm@\^11\.5\.1/);
    assert.match(workflow, /npm run release:check/);
    assert.match(workflow, /npm pack --json/);
    assert.match(workflow, /npm install --prefix "\$smoke_dir"/);
    assert.match(workflow, /npm publish --access public --provenance/);
    assert.match(workflow, /NODE_AUTH_TOKEN: \$\{\{ secrets\.NPM_TOKEN \}\}/);
  });

  it("builds and uploads checksummed TUI release binaries", () => {
    const workflow = readFileSync(".github/workflows/tui-release.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /cargo build --workspace --release --bin architect-mcp-tui/);
    assert.match(workflow, /architect-mcp-tui-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}/);
    assert.match(workflow, /\.sha256/);
    assert.match(workflow, /gh release upload/);
    assert.match(workflow, /os:\s+ubuntu-latest[\s\S]*platform:\s+linux[\s\S]*arch:\s+x64[\s\S]*target:\s+x86_64-unknown-linux-gnu/);
    assert.match(workflow, /os:\s+ubuntu-24\.04-arm[\s\S]*platform:\s+linux[\s\S]*arch:\s+arm64[\s\S]*target:\s+aarch64-unknown-linux-gnu/);
    assert.match(workflow, /platform:\s+macos/);
    assert.match(workflow, /platform:\s+windows/);
  });

  it("smokes TUI install behavior across hosted OSes with pinned actions", () => {
    const workflow = readFileSync(".github/workflows/tui-install-smoke.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /ubuntu-latest/);
    assert.match(workflow, /ubuntu-24\.04-arm/);
    assert.match(workflow, /macos-14/);
    assert.match(workflow, /windows-latest/);
    assert.match(workflow, /rustup toolchain install 1\.94\.0 --profile minimal/);
    assert.match(workflow, /npm run tui:build/);
    assert.match(workflow, /node bin\/architect-mcp-tui\.cjs --help/);
    assert.match(workflow, /node --import tsx --test tests\/tuiShim\.test\.ts/);
  });

  it("documents app-build gates, post-release TUI evidence, and publishing auth migration", () => {
    const newAppGuide = readFileSync("docs/new-app-work-gate.md", "utf8");
    const releaseReadiness = readFileSync("docs/release-readiness.md", "utf8");
    const tuiLiveQa = readFileSync("docs/tui-live-qa.md", "utf8");
    const terminalQa = readFileSync("docs/terminal-qa.md", "utf8");
    const threatModel = readFileSync("docs/prompt-injection-threat-model.md", "utf8");
    const terminalQaIssue = readFileSync(".github/ISSUE_TEMPLATE/terminal-qa-report.yml", "utf8");
    const readme = readFileSync("README.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(newAppGuide, /grill_me/);
    assert.match(newAppGuide, /create_pre_edit_contract/);
    assert.match(newAppGuide, /review_proposed_file_plan/);
    assert.match(newAppGuide, /diff_evidence/);
    assert.match(newAppGuide, /AGENTS\.md/);
    assert.match(threatModel, /MCP tools are report-only/);
    assert.match(threatModel, /direct-mutation-without-gates/);
    assert.match(threatModel, /audit_work_gate_completeness/);
    assert.match(threatModel, /create_work_gate_sequence_receipt/);
    assert.match(threatModel, /fabricated-verification-claim/);
    assert.match(threatModel, /selective-tool-call/);
    assert.match(readme, /new-app-work-gate/);
    assert.match(readme, /prompt-injection-threat-model/);
    assert.match(llms, /docs\/new-app-work-gate\.md/);
    assert.match(llms, /docs\/prompt-injection-threat-model\.md/);
    assert.match(releaseReadiness, /Trusted Publishing Migration/);
    assert.match(releaseReadiness, /Token Rotation/);
    assert.match(releaseReadiness, /npm run release:check/);
    assert.match(tuiLiveQa, /Post-Release Evidence/);
    assert.match(tuiLiveQa, /Manual Linux terminal smoke/);
    assert.match(tuiLiveQa, /Manual Windows terminal smoke/);
    assert.match(tuiLiveQa, /architect-mcp-tui terminal-evidence --markdown/);
    assert.match(terminalQa, /architect-mcp-tui terminal-evidence --markdown/);
    assert.match(terminalQa, /--issue-url https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/136/);
    assert.match(terminalQa, /architect-mcp-tui terminal-evidence --json/);
    assert.match(terminalQa, /collectedAt/);
    assert.match(terminalQa, /--collected-at YYYY-MM-DD/);
    assert.match(terminalQa, /terminal-evidence --markdown/);
    assert.match(terminalQa, /collect-terminal-evidence --json/);
    assert.match(terminalQa, /read-only and does not create, edit, or close issues/);
    assert.match(terminalQa, /launch-judge --json --terminal-evidence terminal-evidence\.json/);
    assert.match(terminalQa, /--terminal-evidence linux-evidence\.json --terminal-evidence windows-evidence\.json/);
    assert.match(terminalQa, /launch-judge --public-summary --terminal-evidence linux-evidence\.json --terminal-evidence windows-evidence\.json/);
    assert.match(terminalQa, /Do not paste the raw smoke JSON into public issues/);
    assert.match(terminalQa, /unchanged template values/);
    assert.match(terminalQa, /placeholder evidence/);
    assert.match(terminalQaIssue, /Launch judge terminal evidence Markdown/);
    assert.match(terminalQaIssue, /architect-mcp-tui terminal-evidence --markdown/);
    assert.match(terminalQaIssue, /--issue-url https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/136/);
    assert.match(terminalQaIssue, /Prefer pasting the public-safe output/);
    assert.match(terminalQaIssue, /collectedAt/);
    assert.match(terminalQaIssue, /architect-mcp-tui terminal-evidence --json/);
    assert.match(terminalQaIssue, /environment/);
    assert.match(terminalQaIssue, /local_terminal/);
    assert.match(terminalQaIssue, /vm_or_cloud_terminal/);
    assert.match(terminalQaIssue, /hosted_ci, container, missing, or unknown provenance does not satisfy #136 manual terminal QA and remains conditional/);
    assert.match(terminalQaIssue, /Do not paste raw smoke JSON/);
    assert.match(terminalQaIssue, /```json/);
    assert.match(terminalQaIssue, /REPLACE with public issue or PR link/);
    assert.doesNotMatch(terminalQaIssue, /issue #136 public-safe terminal QA report/);
    assert.doesNotMatch(terminalQaIssue, /Paste the output from architect-mcp-tui smoke --json/);
  });

  it("runs cross-platform TUI live QA smoke with pinned actions", () => {
    const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
      scripts: Record<string, string>;
    };
    const rustTui = readFileSync("docs/rust-tui.md", "utf8");
    const workflow = readFileSync(".github/workflows/tui-live-qa.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /ubuntu-latest/);
    assert.match(workflow, /macos-14/);
    assert.match(workflow, /windows-latest/);
    assert.match(workflow, /npm run tui:live-qa/);
    assert.match(workflow, /terminal-evidence --json/);
    assert.match(workflow, /GITHUB_STEP_SUMMARY/);
    assert.match(workflow, /manual issue #136 terminal QA still required/);
    assert.match(packageJson.scripts["tui:live-qa"], /launch-judge --public-summary --skip-mcp --skip-smoke/);
    assert.doesNotMatch(packageJson.scripts["tui:live-qa"], /launch-judge --json --skip-mcp --skip-smoke/);
    assert.match(rustTui, /architect-mcp-tui launch-judge --json/);
    assert.match(rustTui, /architect-mcp-tui launch-judge --public-summary/);
    assert.match(rustTui, /architect-mcp-tui launch-stack --json/);
    assert.match(rustTui, /architect-mcp-tui launch-stack --merge-plan/);
    assert.match(rustTui, /architect-mcp-tui launch-readiness --json/);
    assert.match(rustTui, /architect-mcp-tui foundry-smoke --owner <github-owner> --public-summary/);
    assert.match(rustTui, /--retention-decision retain/);
    assert.match(rustTui, /--retention-reason/);
    assert.match(rustTui, /private repo verification, draft PR verification, command pass\/fail counts/);
    assert.match(rustTui, /retention decision metadata/);
    assert.match(rustTui, /omits workspace paths, staged repo paths, private repo target names\/URLs, raw command strings, stdout\/stderr tails/);
    assert.match(rustTui, /does not delete proof repositories/);
    assert.match(rustTui, /architect-mcp-tui evidence-index --json/);
    assert.match(rustTui, /architect-mcp-tui evidence-index --markdown/);
    assert.match(rustTui, /evidence-index --markdown-output <path>/);
    assert.match(rustTui, /evidence-index --require-go/);
    assert.match(rustTui, /terminal-evidence status, environment provenance, collected dates/);
    assert.match(rustTui, /architect-mcp-tui collect-terminal-evidence --json/);
    assert.match(rustTui, /extracts fenced terminal-evidence JSON/);
    assert.match(rustTui, /requested PR changes, or review-thread lookup failures are `no_go`/);
    assert.match(rustTui, /unresolved review threads, pending checks, unstable merge states without explicit required-check evidence, and open blocker issues are `conditional_go`/);
    assert.match(rustTui, /unresolved review-thread counts/);
    assert.match(rustTui, /mergeable=MERGEABLE/);
    assert.match(rustTui, /does not merge PRs, close issues, edit branches, tag releases, publish packages/);
    assert.match(rustTui, /architect-mcp-tui terminal-evidence --markdown/);
    assert.match(rustTui, /architect-mcp-tui terminal-evidence --json/);
    assert.match(rustTui, /Untrusted input labeling is metadata, not prompt-injection prevention/);
    assert.match(rustTui, /adapter prompt includes an untrusted-input policy notice/);
    assert.match(rustTui, /--collected-at YYYY-MM-DD/);
    assert.match(rustTui, /--terminal-evidence linux-evidence\.json --terminal-evidence windows-evidence\.json/);
    assert.match(rustTui, /--terminal-evidence/);
    assert.match(rustTui, /public-safe JSON summary/);
    assert.match(rustTui, /terminal-evidence freeform source text, command summaries, notes/);
    assert.match(rustTui, /Template placeholders are treated as incomplete evidence/);
    assert.match(rustTui, /conditional_go/);
    const tuiLiveQa = readFileSync("docs/tui-live-qa.md", "utf8");
    assert.match(tuiLiveQa, /hosted CI baseline evidence/);
    assert.match(tuiLiveQa, /not a substitute for the real post-release terminal QA tracked in #136/);
    assert.match(tuiLiveQa, /Unchanged issue-template placeholders are not acceptable evidence/);
  });

  it("runs recurring governance audit with pinned actions and public-safe summaries", () => {
    const workflow = readFileSync(".github/workflows/governance-audit.yml", "utf8");
    const issueTemplate = readFileSync(".github/ISSUE_TEMPLATE/governance-audit-report.yml", "utf8");
    const docs = readFileSync("docs/governance-audit.md", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /workflow_dispatch:/);
    assert.match(workflow, /schedule:/);
    assert.match(workflow, /npm run build && cargo build --workspace --bin architect-mcp-tui/);
    assert.match(workflow, /id: governance_audit/);
    assert.match(workflow, /continue-on-error: true/);
    assert.match(workflow, /node bin\/architect-mcp-tui\.cjs governance-audit --public-summary > governance-audit-public-summary\.json/);
    assert.match(workflow, /steps\.governance_audit\.outcome == 'failure'/);
    assert.match(workflow, /run: exit 1/);
    assert.doesNotMatch(workflow, /governance-audit --json > governance-audit\.json/);
    assert.match(workflow, /GITHUB_STEP_SUMMARY/);
    assert.doesNotMatch(workflow, /upload-artifact/);
    assert.match(issueTemplate, /Do not include secrets/);
    assert.match(issueTemplate, /Memory safety confirmed/);
    assert.match(docs, /public-safe/i);
    assert.match(docs, /governance-audit --public-summary/);
    assert.match(docs, /launch-judge --public-summary/);
    assert.match(docs, /launch-stack --json/);
    assert.match(docs, /--required-check <name>/);
    assert.match(docs, /missing required-check evidence/);
    assert.match(docs, /unresolved review threads/);
    assert.match(docs, /launch-readiness --json/);
    assert.match(docs, /evidence-index --json/);
    assert.match(docs, /evidence-index --markdown/);
    assert.match(docs, /evidence-index --markdown-output <path>/);
    assert.match(docs, /evidence-index --require-go/);
    assert.match(docs, /terminal-evidence status and environment provenance/);
    assert.match(docs, /npm run release:check/);
  });

  it("keeps the evolved goal ledger aligned with the active slice and launch boundary", () => {
    const goal = readFileSync("docs/goal-ai-software-foundry.md", "utf8");

    assert.match(goal, /Active slice: \[#261 - Expose terminal evidence provenance in launch readiness summaries\]/);
    assert.match(goal, /\[PR #213\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/213\)/);
    assert.match(goal, /\[PR #215\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/215\)/);
    assert.match(goal, /\[PR #217\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/217\)/);
    assert.match(goal, /\[PR #219\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/219\)/);
    assert.match(goal, /\[PR #221\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/221\)/);
    assert.match(goal, /\[PR #223\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/223\)/);
    assert.match(goal, /\[PR #225\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/225\)/);
    assert.match(goal, /\[PR #227\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/227\)/);
    assert.match(goal, /\[PR #229\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/229\)/);
    assert.match(goal, /\[PR #231\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/231\)/);
    assert.match(goal, /\[PR #233\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/233\)/);
    assert.match(goal, /\[PR #235\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/pull\/235\)/);
    assert.match(goal, /\[#224\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/224\)/);
    assert.match(goal, /\[#226\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/226\)/);
    assert.match(goal, /\[#228\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/228\)/);
    assert.match(goal, /\[#230\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/230\)/);
    assert.match(goal, /\[#232\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/232\)/);
    assert.match(goal, /\[#234\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/234\)/);
    assert.match(goal, /\[#236\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/236\)/);
    assert.match(goal, /\[#239\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/239\)/);
    assert.match(goal, /\[#241\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/241\)/);
    assert.match(goal, /\[#242\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/242\)/);
    assert.match(goal, /\[#244\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/244\)/);
    assert.match(goal, /\[#245\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/245\)/);
    assert.match(goal, /\[#246\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/246\)/);
    assert.match(goal, /\[#247\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/247\)/);
    assert.match(goal, /\[#253\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/253\)/);
    assert.match(goal, /\[#255\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/255\)/);
    assert.match(goal, /\[#257\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/257\)/);
    assert.match(goal, /\[#259\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/259\)/);
    assert.match(goal, /\[#261\]\(https:\/\/github\.com\/tonycdr-prog\/architect-mcp\/issues\/261\)/);
    assert.match(goal, /fail-closed ACP session configuration/);
    assert.match(goal, /unknown session parameters/);
    assert.match(goal, /strict ACP session-method parameter validation/);
    assert.match(goal, /strict ACP JSON-RPC envelope validation/);
    assert.match(goal, /explicit TUI promotion override reason evidence/);
    assert.match(goal, /durable TUI promotion receipts/);
    assert.match(goal, /operator-facing receipt inspection/);
    assert.match(goal, /read-only launch-stack merge checklist/);
    assert.match(goal, /public-safe repo-foundry smoke summaries/);
    assert.match(goal, /explicit foundry proof-repo retention decisions/);
    assert.match(goal, /provenance-aware terminal evidence validation/);
    assert.match(goal, /PR review-decision launch readiness/);
    assert.match(goal, /explicit required-check launch readiness/);
    assert.match(goal, /public required-check evidence handoffs/);
    assert.match(goal, /published-package hosted smoke coverage/);
    assert.match(goal, /Linux ARM64 TUI release assets/);
    assert.match(goal, /prompt-injection\/gate-bypass threat-model coverage/);
    assert.match(goal, /TUI untrusted-input labels/);
    assert.match(goal, /direct-client work-gate completeness auditing/);
    assert.match(goal, /structured verification command receipts/);
    assert.match(goal, /direct-client work-gate sequence receipts/);
    assert.match(goal, /mergeable unstable launch-readiness handling/);
    assert.match(goal, /safer issue-targeted terminal-evidence Markdown/);
    assert.match(goal, /unresolved review-thread launch-readiness gating/);
    assert.match(goal, /terminal-QA template provenance guidance/);
    assert.match(goal, /launch-readiness\/evidence-index terminal provenance summaries/);
    assert.match(goal, /#228 in PR #229/);
    assert.match(goal, /#230 in PR #231/);
    assert.match(goal, /#232 in PR #233/);
    assert.match(goal, /#234 in PR #235/);
    assert.match(goal, /#236 in PR #238/);
    assert.match(goal, /#239 in PR #240/);
    assert.match(goal, /#241 in PR #243/);
    assert.match(goal, /#242 in PR #248/);
    assert.match(goal, /#244 in PR #249/);
    assert.match(goal, /#245 in PR #250/);
    assert.match(goal, /#246 in PR #251/);
    assert.match(goal, /#247 in PR #252/);
    assert.match(goal, /#253 in PR #254/);
    assert.match(goal, /#255 in PR #256/);
    assert.match(goal, /#257 in PR #258/);
    assert.match(goal, /#259 in PR #260/);
    assert.match(goal, /#261 in the current PR/);
    assert.match(goal, /#136 remains open and is still the evidence boundary/);
    assert.match(goal, /runtime `\/goal` remains active/);
    assert.match(goal, /npm run release:check/);
    assert.doesNotMatch(goal, /The full evolved objective is complete/i);
  });

  it("configures Dependabot for npm, Cargo, and GitHub Actions", () => {
    const config = readFileSync(".github/dependabot.yml", "utf8");

    assert.match(config, /package-ecosystem:\s+npm/);
    assert.match(config, /package-ecosystem:\s+github-actions/);
    assert.match(config, /package-ecosystem:\s+cargo/);
  });
});
