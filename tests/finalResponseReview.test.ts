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

  it("fails when required checks or root-cause evidence are missing without treating every because as causal proof", () => {
    const missingCheck = reviewAgentFinalResponse({
      response: "Changed code. Verified with npm test. Assumptions: none. Not done: no remaining work.",
      requiredChecks: ["npm run typecheck"]
    });
    const unsupportedCause = reviewAgentFinalResponse({
      response: "Changed the cache code. Verified with npm test. Root cause: the cache was stale. Assumptions: none. Not done: no remaining work."
    });
    const nonCausalBecause = reviewAgentFinalResponse({
      response: "Changed code comments. Because of time, typecheck was not run. Assumptions: none. Not done: typecheck remains."
    });

    assert.equal(missingCheck.status, "fail");
    assert.equal(unsupportedCause.status, "fail");
    assert.equal(nonCausalBecause.findings.some((finding) => finding.code === "FINAL004_ROOT_CAUSE_UNSUPPORTED"), false);
  });
});
