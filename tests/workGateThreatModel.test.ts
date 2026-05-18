import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  UNTRUSTED_AGENT_INPUTS,
  WORK_GATE_BOUNDARIES,
  WORK_GATE_BYPASS_CASES,
  classifyWorkGateBoundary,
  evaluateBypassCase,
  validateThreatModelReferences,
  type WorkGateBoundary,
  type WorkGateBypassCase
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
    assert.deepEqual(validateThreatModelReferences().findings, []);

    for (const testCase of WORK_GATE_BYPASS_CASES) {
      assert.equal(testCase.reproducibleSteps.length >= 3, true, `${testCase.id} needs reproducible steps`);
      assert.equal(evaluateBypassCase(testCase.id).protectedByArchitectMcpAlone, false, `${testCase.id} should not be overclaimed`);
    }
  });

  it("fails closed when bypass cases reference unknown source or boundary ids", () => {
    const invalidCase: WorkGateBypassCase = {
      ...WORK_GATE_BYPASS_CASES[0],
      id: "invalid-cross-reference",
      untrustedSources: ["issue-pr-text", "issue-pr-typo"],
      affectedBoundaryIds: ["core-mcp-tools", "missing-boundary"]
    };
    const review = validateThreatModelReferences([invalidCase]);

    assert.equal(review.valid, false);
    assert.deepEqual(review.findings.map((finding) => finding.field), ["untrustedSources", "affectedBoundaryIds"]);
    assert.match(review.findings[0].message, /unknown untrusted input source issue-pr-typo/);
    assert.match(review.findings[1].message, /unknown work-gate boundary missing-boundary/);
  });

  it("includes reference findings in bypass evaluation before claiming coverage", () => {
    const boundaryIds = WORK_GATE_BOUNDARIES
      .filter((boundary) => boundary.id !== "core-mcp-tools")
      .map((boundary) => boundary.id);

    const referenceReview = validateThreatModelReferences(WORK_GATE_BYPASS_CASES, {
      boundaries: WORK_GATE_BOUNDARIES.filter((boundary) => boundary.id !== "core-mcp-tools")
    });

    assert.equal(boundaryIds.includes("core-mcp-tools"), false);
    assert.equal(referenceReview.valid, false);
    assert.equal(referenceReview.findings.some((finding) =>
      finding.field === "affectedBoundaryIds" && finding.id === "core-mcp-tools"
    ), true);
  });

  it("does not overclaim coverage when a bypass case has invalid source references", () => {
    const id = "invalid-source-reference";
    const injectedCase: WorkGateBypassCase = {
      ...WORK_GATE_BYPASS_CASES[0],
      id,
      untrustedSources: ["issue-pr-typo"],
      affectedBoundaryIds: ["tui-promotion", "release-check"]
    };
    WORK_GATE_BYPASS_CASES.push(injectedCase);

    try {
      const evaluation = evaluateBypassCase(id);

      assert.equal(evaluation.referenceReview.valid, false);
      assert.equal(evaluation.protectedByArchitectMcpAlone, false);
      assert.equal(evaluation.requiredControls.includes("fix threat-model reference ids before claiming coverage"), true);
    } finally {
      const removed = WORK_GATE_BYPASS_CASES.pop();
      assert.equal(removed?.id, id);
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
      const row = gateBoundaryRow(doc, boundary);

      assert.match(row, new RegExp(`\\b${boundary.id}\\b`));
      assert.match(row, new RegExp(`\\b${boundary.enforcement}\\b`));
      assert.match(row, expectedBoundaryLimitationPattern(boundary.id));
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

function gateBoundaryRow(doc: string, boundary: WorkGateBoundary): string {
  const row = doc.split("\n").find((line) => line.startsWith(`| \`${boundary.id}\``));

  assert.equal(typeof row, "string", `${boundary.id} must have a dedicated gate-boundary table row`);
  return row as string;
}

function expectedBoundaryLimitationPattern(id: string): RegExp {
  const patterns: Record<string, RegExp> = {
    "core-mcp-tools": /cannot force a client to call every tool or stop editing/,
    "tui-pre-adapter-flow": /cannot stop edits made outside the TUI/,
    "tui-adapter-execution": /separate shell commands outside the TUI/,
    "tui-promotion": /not direct workspace edits/,
    "final-response-review": /cannot prove a command ran unless external evidence is supplied/,
    "release-check": /does not prove manual terminal QA or unconfigured workflows/,
    "memory-policy": /not enforcement unless the host or memory tool enforces/,
    "direct-shell-files": /does not sandbox the shell or filesystem by itself/
  };

  const pattern = patterns[id];
  assert.ok(pattern, `${id} needs an expected limitation pattern`);
  return pattern;
}
