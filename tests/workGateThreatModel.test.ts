import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  UNTRUSTED_AGENT_INPUTS,
  WORK_GATE_BOUNDARIES,
  WORK_GATE_BYPASS_CASES,
  classifyWorkGateBoundary,
  evaluateBypassCase
} from "../src/domain/workGateThreatModel.js";

describe("work gate threat model", () => {
  it("maps advisory/report-only gates separately from code-enforced TUI and release gates", () => {
    assert.equal(classifyWorkGateBoundary("core-mcp-tools").enforcement, "mcp_report_only");
    assert.equal(classifyWorkGateBoundary("core-mcp-tools").codeEnforced, false);
    assert.equal(classifyWorkGateBoundary("tui-promotion").enforcement, "tui_state_enforced");
    assert.equal(classifyWorkGateBoundary("tui-promotion").codeEnforced, true);
    assert.equal(classifyWorkGateBoundary("release-check").enforcement, "ci_release_enforced");
    assert.equal(classifyWorkGateBoundary("release-check").codeEnforced, true);
    assert.equal(classifyWorkGateBoundary("direct-shell-files").codeEnforced, false);
  });

  it("fails closed for unknown gate boundaries", () => {
    const unknown = classifyWorkGateBoundary("imaginary gate");

    assert.equal(unknown.enforcement, "unknown");
    assert.equal(unknown.codeEnforced, false);
    assert.match(unknown.limitation, /Do not claim/);
  });

  it("documents public-safe bypass classes that cannot be claimed as fully prevented by architect-mcp alone", () => {
    assert.equal(WORK_GATE_BYPASS_CASES.length >= 3, true);
    assert.equal(WORK_GATE_BYPASS_CASES.every((testCase) => testCase.publicSafe), true);

    for (const testCase of WORK_GATE_BYPASS_CASES) {
      assert.equal(testCase.reproducibleSteps.length >= 3, true, `${testCase.id} needs reproducible steps`);
      assert.equal(evaluateBypassCase(testCase.id).protectedByArchitectMcpAlone, false, `${testCase.id} should not be overclaimed`);
    }
  });

  it("covers the main untrusted input paths agents see during repository work", () => {
    const sourceIds = UNTRUSTED_AGENT_INPUTS.map((source) => source.id);

    for (const expected of ["issue-pr-text", "repo-docs", "tool-output", "web-research", "memory"]) {
      assert.equal(sourceIds.includes(expected), true);
    }
    assert.equal(UNTRUSTED_AGENT_INPUTS.every((source) => /data|advisory|separated|authority/i.test(source.handling)), true);
  });

  it("keeps the public docs aligned with the threat model and avoids gate overclaiming", () => {
    const doc = readFileSync("docs/prompt-injection-threat-model.md", "utf8");
    const readme = readFileSync("README.md", "utf8");
    const llms = readFileSync("llms.txt", "utf8");

    for (const boundary of WORK_GATE_BOUNDARIES) {
      assert.match(doc, new RegExp(`\\b${boundary.id}\\b`));
    }
    for (const testCase of WORK_GATE_BYPASS_CASES) {
      assert.match(doc, new RegExp(`\\b${testCase.id}\\b`));
    }
    assert.match(doc, /MCP tools are report-only/);
    assert.match(doc, /architect-mcp does not sandbox the shell or filesystem by itself/);
    assert.match(doc, /github\.com\/tonycdr-prog\/architect-mcp\/issues\/244/);
    assert.match(doc, /github\.com\/tonycdr-prog\/architect-mcp\/issues\/245/);
    assert.match(doc, /github\.com\/tonycdr-prog\/architect-mcp\/issues\/246/);
    assert.match(doc, /github\.com\/tonycdr-prog\/architect-mcp\/issues\/247/);
    assert.match(readme, /Prompt Injection And Gate Bypass Threat Model/);
    assert.match(llms, /docs\/prompt-injection-threat-model\.md/);
  });
});
