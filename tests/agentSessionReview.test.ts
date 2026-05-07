import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createPreEditContract, interpretImplementationIntent } from "../src/domain/harness.js";
import { reviewAgentSession } from "../src/domain/agentSessionReview.js";
import type { MemoryProposal } from "../src/domain/types.js";

describe("reviewAgentSession", () => {
  it("combines intent, contract, verification, and final response into one report", () => {
    const intent = interpretImplementationIntent({
      request: "clean this component up with best practices",
      mode: "guided-yolo",
      verification: ["npm test"]
    });
    const contract = createPreEditContract({
      intent,
      likelyFiles: ["src/features/profile/ProfileCard.tsx"],
      verificationChecks: ["npm test"]
    });

    const report = reviewAgentSession({
      intent,
      contract,
      changedFiles: [{ path: "src/features/profile/ProfileCard.tsx", lines: 120 }],
      verification: [{ check: "npm test", status: "passed" }],
      finalResponse: "Changed ProfileCard. Verified with npm test. Assumptions: no new assumptions. Not done: no remaining requested work."
    });

    assert.equal(report.status, intent.decision === "confirm_before_edit" ? "warn" : "pass");
    assert.equal(report.sections.some((section) => section.name === "implementation-contract"), true);
    assert.equal(report.sections.some((section) => section.name === "final-response"), true);
  });

  it("reviews memory safety even when the session request is missing", () => {
    const memory: MemoryProposal = {
      id: "unsafe",
      kind: "preference",
      scope: "user",
      statement: "Remember token ghp_secret_value for future use.",
      rationale: "Unsafe supplied memory.",
      confidence: "high",
      risk: "green",
      sensitivity: "internal",
      policyAction: "auto_store",
      tags: ["agent-harness"],
      tokenEstimate: 20,
      source: { kind: "manual", summary: "manual" },
      invalidatedBy: "Never",
      targetPath: "user/preference.jsonl"
    };
    const report = reviewAgentSession({
      memories: [memory]
    });

    assert.equal(report.status, "fail");
    assert.equal(report.sections.some((section) => section.name === "memory"), true);
  });
});
