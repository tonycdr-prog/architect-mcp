import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { reviewAgentFinalResponse } from "../src/domain/finalResponseReview.js";

describe("reviewAgentFinalResponse", () => {
  it("fails responses that omit verification", () => {
    const result = reviewAgentFinalResponse({
      response: "I fixed the issue.",
      requiredChecks: ["npm test"]
    });

    assert.equal(result.status, "fail");
    assert.equal(result.findings.some((finding) => finding.code === "FINAL002_VERIFICATION_MISSING"), true);
  });

  it("passes complete final responses", () => {
    const result = reviewAgentFinalResponse({
      response: "Changed the MCP config scanner. Verified with npm run typecheck, npm test, and npm run build. Assumptions: no new assumptions. Not done: no remaining requested work.",
      requiredChecks: ["npm run typecheck", "npm test", "npm run build"]
    });

    assert.equal(result.status, "pass");
    assert.equal(result.valid, true);
  });
});
