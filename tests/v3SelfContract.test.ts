import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPreEditContract, interpretImplementationIntent, reviewImplementationAgainstContract } from "../src/domain/harness.js";
import { scoreAgentInstructions, scoreLlmsTxt } from "../src/domain/artifactQuality.js";
import { runV3EvalHarness } from "../src/domain/v3EvalHarness.js";

describe("V3 self-contract loop", () => {
  it("runs the MCP harness on the V3 implementation surface", () => {
    const intent = interpretImplementationIntent({
      request: "make the V3 MCP implementation bulletproof with best practices",
      mode: "guided-yolo",
      stack: {
        backend: "TypeScript MCP server"
      },
      selectedSourceIds: ["zod", "hono"],
      verification: ["npm run typecheck", "npm test", "npm run build"]
    });

    const contract = createPreEditContract({
      intent,
      likelyFiles: [
        "src/domain/*.ts",
        "src/tools/*.ts",
        "src/tools/schemas/*.ts",
        "tests/*.test.ts",
        "tests/fixtures/v3/*.json",
        "README.md",
        "llms.txt",
        "AGENTS.md"
      ],
      verificationChecks: ["npm run typecheck", "npm test", "npm run build"]
    });

    const drift = reviewImplementationAgainstContract({
      contract,
      changedFiles: [
        { path: "src/tools/schemas/commonSchemas.ts", lines: 202 },
        { path: "src/domain/mcpSecurity.ts", lines: 180 },
        { path: "tests/v3SelfContract.test.ts", lines: 70 },
        { path: "llms.txt", lines: 120 }
      ],
      verification: [
        { check: "npm run typecheck", status: "passed" },
        { check: "npm test", status: "passed" },
        { check: "npm run build", status: "passed" }
      ]
    });

    const evals = runV3EvalHarness();
    const artifactQuality = {
      agents: scoreAgentInstructions(readFileSync("AGENTS.md", "utf8")),
      llms: scoreLlmsTxt(readFileSync("llms.txt", "utf8"))
    };

    assert.equal(intent.decision, "confirm_before_edit");
    assert.equal(contract.outputContract.includes("State verification run and results."), true);
    assert.equal(drift.valid, true);
    assert.equal(evals.status, "pass");
    assert.equal(artifactQuality.agents.status, "pass");
    assert.equal(artifactQuality.llms.status, "pass");
  });
});
