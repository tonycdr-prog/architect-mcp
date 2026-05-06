import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { scoreAgentInstructions, scoreLlmsTxt } from "../src/domain/artifactQuality.js";

const fixtures = JSON.parse(readFileSync("tests/fixtures/v3/artifacts.json", "utf8")) as {
  weakAgentsMd: string;
  validLlmsTxt: string;
};

describe("artifact quality scoring", () => {
  it("fails weak agent instructions that lack commands, boundaries, and proof", () => {
    const result = scoreAgentInstructions(fixtures.weakAgentsMd);

    assert.equal(result.status, "fail");
    assert.equal(result.findings.some((finding) => finding.code === "ARTIFACT002_MISSING_COMMAND"), true);
    assert.equal(result.findings.some((finding) => finding.code === "ARTIFACT004_MISSING_PROOF"), true);
  });

  it("passes current repo AGENTS.md and llms.txt", () => {
    assert.equal(scoreAgentInstructions(readFileSync("AGENTS.md", "utf8")).status, "pass");
    assert.equal(scoreLlmsTxt(readFileSync("llms.txt", "utf8")).status, "pass");
  });

  it("accepts a valid llms.txt navigation fixture", () => {
    const result = scoreLlmsTxt(fixtures.validLlmsTxt);

    assert.equal(result.status, "pass");
    assert.equal(result.findings.length, 0);
  });
});
