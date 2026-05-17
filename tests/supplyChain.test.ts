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
    assert.match(workflow, /platform:\s+linux/);
    assert.match(workflow, /platform:\s+macos/);
    assert.match(workflow, /platform:\s+windows/);
  });

  it("smokes TUI install behavior across hosted OSes with pinned actions", () => {
    const workflow = readFileSync(".github/workflows/tui-install-smoke.yml", "utf8");
    const usesLines = workflow.split("\n").filter((line) => line.trim().startsWith("uses:"));

    assert.equal(usesLines.length > 0, true);
    assert.equal(usesLines.every((line) => /@[0-9a-f]{40}(?:\s+#.*)?$/.test(line.trim())), true);
    assert.match(workflow, /ubuntu-latest/);
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
    const terminalQaIssue = readFileSync(".github/ISSUE_TEMPLATE/terminal-qa-report.yml", "utf8");
    const readme = readFileSync("README.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    assert.match(newAppGuide, /grill_me/);
    assert.match(newAppGuide, /create_pre_edit_contract/);
    assert.match(newAppGuide, /review_proposed_file_plan/);
    assert.match(newAppGuide, /diff_evidence/);
    assert.match(newAppGuide, /AGENTS\.md/);
    assert.match(readme, /new-app-work-gate/);
    assert.match(llms, /docs\/new-app-work-gate\.md/);
    assert.match(releaseReadiness, /Trusted Publishing Migration/);
    assert.match(releaseReadiness, /Token Rotation/);
    assert.match(releaseReadiness, /npm run release:check/);
    assert.match(tuiLiveQa, /Post-Release Evidence/);
    assert.match(tuiLiveQa, /Manual Linux terminal smoke/);
    assert.match(tuiLiveQa, /Manual Windows terminal smoke/);
    assert.match(terminalQa, /architect-mcp-tui terminal-evidence --json/);
    assert.match(terminalQa, /collect-terminal-evidence --json/);
    assert.match(terminalQa, /read-only and does not create, edit, or close issues/);
    assert.match(terminalQa, /launch-judge --json --terminal-evidence terminal-evidence\.json/);
    assert.match(terminalQa, /--terminal-evidence linux-evidence\.json --terminal-evidence windows-evidence\.json/);
    assert.match(terminalQa, /launch-judge --public-summary --terminal-evidence linux-evidence\.json --terminal-evidence windows-evidence\.json/);
    assert.match(terminalQa, /Do not paste the raw smoke JSON into public issues/);
    assert.match(terminalQa, /unchanged template values/);
    assert.match(terminalQa, /placeholder evidence/);
    assert.match(terminalQaIssue, /Launch judge terminal evidence JSON/);
    assert.match(terminalQaIssue, /architect-mcp-tui terminal-evidence --json/);
    assert.match(terminalQaIssue, /Do not paste raw smoke JSON/);
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
    assert.match(rustTui, /architect-mcp-tui launch-readiness --json/);
    assert.match(rustTui, /architect-mcp-tui evidence-index --json/);
    assert.match(rustTui, /architect-mcp-tui evidence-index --markdown/);
    assert.match(rustTui, /evidence-index --markdown-output <path>/);
    assert.match(rustTui, /architect-mcp-tui collect-terminal-evidence --json/);
    assert.match(rustTui, /extracts fenced terminal-evidence JSON/);
    assert.match(rustTui, /draft PRs, pending checks, temporarily unstable merge states caused by pending checks, and open blocker issues are `conditional_go`/);
    assert.match(rustTui, /architect-mcp-tui terminal-evidence --json/);
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
    assert.match(docs, /launch-readiness --json/);
    assert.match(docs, /evidence-index --json/);
    assert.match(docs, /evidence-index --markdown/);
    assert.match(docs, /evidence-index --markdown-output <path>/);
    assert.match(docs, /npm run release:check/);
  });

  it("configures Dependabot for npm, Cargo, and GitHub Actions", () => {
    const config = readFileSync(".github/dependabot.yml", "utf8");

    assert.match(config, /package-ecosystem:\s+npm/);
    assert.match(config, /package-ecosystem:\s+github-actions/);
    assert.match(config, /package-ecosystem:\s+cargo/);
  });
});
