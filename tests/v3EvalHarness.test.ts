import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runV3EvalHarness } from "../src/domain/v3EvalHarness.js";

const fixture = JSON.parse(readFileSync("tests/fixtures/v3/eval-cases.json", "utf8")) as {
  suites: Array<"harness" | "memory" | "mcp-security" | "artifact-quality" | "stack-pack">;
  expected: { status: string; minimumCases: number };
};

describe("runV3EvalHarness", () => {
  it("runs the full V3 suite from fixture expectations", () => {
    const report = runV3EvalHarness({ suites: fixture.suites });

    assert.equal(report.status, fixture.expected.status);
    assert.equal(report.summary.total >= fixture.expected.minimumCases, true);
    assert.equal(report.summary.failed, 0);
  });

  it("can run one focused suite", () => {
    const report = runV3EvalHarness({ suites: ["mcp-security"] });

    assert.equal(report.status, "pass");
    assert.deepEqual(report.cases.map((testCase) => testCase.suite), ["mcp-security"]);
  });

  it("does not pass with zero executed cases and runs stack-pack workflow assertions", () => {
    const empty = runV3EvalHarness({ suites: [] });
    const stackPack = runV3EvalHarness({ suites: ["stack-pack"] });

    assert.equal(empty.status, "pass");
    assert.equal(empty.summary.total >= fixture.expected.minimumCases, true);
    assert.equal(stackPack.status, "pass");
    assert.equal(stackPack.summary.total, 1);
    assert.match(stackPack.cases[0]?.actual ?? "", /dryRun/);
  });
});
