import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createPreEditContract, interpretImplementationIntent, reviewImplementationAgainstContract } from "../src/domain/harness.js";
import { loadTriggeredStackGuidance } from "../src/domain/harnessGuidance.js";

describe("guided yolo harness", () => {
  it("blocks blank requests instead of allowing yolo assumptions", () => {
    const result = interpretImplementationIntent({
      request: "   "
    });

    assert.equal(result.decision, "block_until_clarified");
    assert.equal(result.stoplight, "red");
    assert.equal(result.confirmationPrompt?.includes("What problem"), true);
  });

  it("asks for confirmation when vague best-practice intent has unclear scope", () => {
    const result = interpretImplementationIntent({
      request: "fix this problem using best practices",
      stack: {
        frontend: "React",
        backend: "Hono",
        database: "Supabase"
      }
    });

    assert.equal(result.mode, "guided-yolo");
    assert.equal(result.decision, "confirm_before_edit");
    assert.equal(result.stoplight, "red");
    assert.equal(result.escalationTerms.includes("best practices"), true);
    assert.equal(result.confirmationPrompt?.includes("I think you mean"), true);
    assert.equal(result.triggeredGuidance.some((guidance) => guidance.sourceId === "react"), true);
  });

  it("asks for confirmation for generic best-practice prompts without stack hints", () => {
    const result = interpretImplementationIntent({
      request: "fix this with best practices"
    });

    assert.equal(result.decision, "confirm_before_edit");
    assert.equal(result.confirmationPrompt?.includes("I think you mean"), true);
  });

  it("proceeds with assumptions for isolated low-risk cleanup", () => {
    const result = interpretImplementationIntent({
      request: "clean this component up",
      files: [{ path: "src/features/profile/ProfileCard.tsx", lines: 120 }]
    });

    assert.equal(result.decision, "proceed_with_assumptions");
    assert.equal(result.stoplight, "yellow");
    assert.equal(result.blastRadius, "low");
    assert.equal(result.assumptions.length, 1);
  });

  it("treats auth, database optimization, security, destructive commands, and dependency upgrades as risky", () => {
    assert.equal(interpretImplementationIntent({ request: "make auth better" }).decision, "confirm_before_edit");
    assert.equal(interpretImplementationIntent({ request: "optimize database properly" }).blastRadius, "medium");
    assert.equal(interpretImplementationIntent({ request: "secure it" }).decision, "confirm_before_edit");
    assert.equal(interpretImplementationIntent({ request: "run rm -rf and reset --hard to fix it" }).decision, "block_until_clarified");
    assert.equal(interpretImplementationIntent({ request: "delete all files" }).decision, "block_until_clarified");
    assert.equal(interpretImplementationIntent({ request: "delete src folder" }).decision, "block_until_clarified");
    assert.equal(interpretImplementationIntent({ request: "upgrade packages to fix this" }).warnings.some((warning) => warning.includes("Dependency upgrade")), true);
  });

  it("maps novice terminology into plain options", () => {
    const result = interpretImplementationIntent({
      request: "make UI professional and secure it"
    });

    assert.equal(result.plainLanguageOptions.some((option) => option.includes("spacing")), true);
    assert.equal(result.plainLanguageOptions.some((option) => option.includes("authorization")), true);
  });

  it("does not classify words containing ui as styling", () => {
    const result = interpretImplementationIntent({
      request: "audit and harden the agent harness implementation using best practices",
      stack: {
        backend: "TypeScript MCP server"
      }
    });

    assert.notEqual(result.changeType, "styling");
  });

  it("requires evidence before root-cause claims", () => {
    const result = interpretImplementationIntent({
      request: "the root cause is bad state, fix it"
    });

    assert.equal(result.warnings.some((warning) => warning.includes("Root-cause claim needs evidence")), true);
  });

  it("loads triggered llms.txt guidance without failing on stale sources", () => {
    const result = loadTriggeredStackGuidance({
      request: "use Astro best practices",
      selectedSourceIds: ["astro", "hono"]
    });

    assert.equal(result.guidance.some((guidance) => guidance.sourceId === "hono" && guidance.status === "loaded"), true);
    assert.equal(result.guidance.some((guidance) => guidance.sourceId === "astro" && guidance.status !== "loaded"), true);
    assert.equal(result.warnings.some((warning) => warning.includes("astro")), true);
  });

  it("creates pre-edit contracts and catches drift, skipped checks, and monolith plans", () => {
    const intent = interpretImplementationIntent({
      request: "fix this with best practices",
      stack: {
        frontend: "React"
      }
    });
    const contract = createPreEditContract({
      intent,
      likelyFiles: ["src/features/profile/ProfileCard.tsx"]
    });
    const review = reviewImplementationAgainstContract({
      contract,
      changedFiles: [
        { path: "src/features/profile/ProfileCard.tsx", lines: 140 },
        { path: "src/auth/session.ts", lines: 90 }
      ],
      proposedPlan: {
        files: [{
          path: "src/App.tsx",
          purpose: "Own all UI, auth, data, routing, validation, and state.",
          responsibilities: ["ui", "auth", "data", "routing", "validation", "state"]
        }]
      },
      verification: [{ check: "npm test", status: "skipped" }]
    });

    assert.equal(review.valid, false);
    assert.equal(review.violations.some((violation) => violation.message.includes("outside the pre-edit contract")), true);
    assert.equal(review.violations.some((violation) => violation.message.includes("Verification")), true);
    assert.equal(review.violations.some((violation) => violation.message.includes("entry file")), true);
  });

  it("anchors likely-file glob matching to the full path", () => {
    const intent = interpretImplementationIntent({
      request: "clean this component up",
      files: [{ path: "src/features/profile/ProfileCard.tsx", lines: 120 }]
    });
    const contract = createPreEditContract({
      intent,
      likelyFiles: ["src/**/*.tsx"]
    });
    const review = reviewImplementationAgainstContract({
      contract,
      changedFiles: [{ path: "other/ProfileCard.tsx", lines: 80 }],
      verification: [{ check: "npm test", status: "passed" }]
    });

    assert.equal(review.valid, false);
    assert.equal(review.violations.some((violation) => violation.message.includes("outside the pre-edit contract")), true);
  });
});
