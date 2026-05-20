import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { routeFoundryDecisions } from "../src/domain/foundryDecisionLedger.js";
import { scoreFoundryActionability } from "../src/domain/foundryActionability.js";
import { normalizeFoundryEvidence } from "../src/domain/foundryEvidence.js";
import type { FoundryActionabilityReport } from "../src/domain/foundryActionabilityTypes.js";

const publicSafety = {
  rawPayloadsIncluded: false,
  rawRepoContentIncluded: false,
  localPathsIncluded: false,
  tokenValuesIncluded: false,
  mutationAllowed: false,
  publicRecommendationsOnly: true
} as const;

describe("routeFoundryDecisions", () => {
  it("routes scored findings to every ledger route deterministically", () => {
    const ledger = routeFoundryDecisions({
      ledgerId: "slice-322",
      recordedAt: "2026-05-20T00:00:00.000Z",
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 5, byDecision: {}, prPreviewCandidates: 1, askHuman: 2, exceptionCandidates: 1, noOpCandidates: 1, publicSafetyHolds: 1 },
        assessments: [
          assessment("fev-pr", "pr_preview_candidate", 82, 85, 85),
          assessment("fev-issue", "ask_human", 68, 85, 85, ["A passing verification path is missing."]),
          assessment("fev-exception", "exception_candidate", 52, 85, 45, ["Suppression prerequisite docs_example should be resolved before routing."]),
          assessment("fev-noop", "no_op_candidate", 42, 85, 35),
          assessment("fev-human", "ask_human", 66, 20, 85, ["Security-sensitive evidence needs human disclosure review before public recommendations."])
        ],
        publicSafety
      }
    });

    assert.equal(ledger.schemaVersion, 1);
    assert.equal(ledger.ledgerId, "foundry-ledger");
    assert.deepEqual(ledger.summary.byRoute, {
      pr_preview: 1,
      architect_issue: 1,
      exception: 1,
      no_op: 1,
      ask_human: 1
    });
    assert.equal(ledger.entries[0].approvalState, "approval_required");
    assert.equal(ledger.entries[1].route, "architect_issue");
    assert.equal(ledger.entries[3].approvalState, "not_required");
    assert.equal(ledger.entries[4].approvalState, "human_required");
    assert.equal(ledger.publicSafety.mutationAllowed, false);
    assert.equal(ledger.summary.serverWritesPerformed, 0);
  });

  it("routes real scored actionability output without mutation", () => {
    const inventory = normalizeFoundryEvidence({
      findings: [{
        code: "ARCH008_ENV_SCATTER",
        confidence: "high",
        severity: "error",
        path: "src/config/env.ts",
        message: "Environment access is scattered.",
        recommendation: "Centralize environment parsing."
      }],
      verification: [{ check: "npm test", status: "passed" }]
    });
    const actionability = scoreFoundryActionability({ inventory });
    const ledger = routeFoundryDecisions({ actionability });

    assert.equal(ledger.entries[0].route, "pr_preview");
    assert.equal(ledger.entries[0].mutation.serverMutationAllowed, false);
    assert.equal(ledger.entries[0].nextAction.includes("do not write to GitHub"), true);
  });

  it("redacts unsafe caller-controlled ledger fields", () => {
    const ledger = routeFoundryDecisions({
      ledgerId: "/Users/alice/project/npm_abcdefghijklmnopqrstuvwxyz1234567890",
      recordedAt: "stdout: private timestamp",
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 1, byDecision: {}, prPreviewCandidates: 0, askHuman: 1, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 1 },
        assessments: [
          assessment(
            "/Users/alice/project/npm_abcdefghijklmnopqrstuvwxyz1234567890",
            "ask_human",
            66,
            45,
            85,
            ["payload: private blocker"],
            ["stderr: private verification"]
          )
        ],
        publicSafety
      }
    });
    const serialized = JSON.stringify(ledger);

    assert.equal(ledger.ledgerId, "foundry-ledger");
    assert.deepEqual(ledger.entries[0].evidenceIds, ["fde-001"]);
    assert.equal(serialized.includes("/Users/alice"), false);
    assert.equal(serialized.includes("npm_abcdefghijklmnopqrstuvwxyz1234567890"), false);
    assert.equal(serialized.includes("private"), false);
    assert.equal(ledger.entries[0].redactionState, "redacted");
  });

  it("routes sensitive upstream candidates to human review before candidate routes", () => {
    const ledger = routeFoundryDecisions({
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 1, byDecision: {}, prPreviewCandidates: 1, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 1 },
        assessments: [
          assessment(
            "fev-sensitive-pr",
            "pr_preview_candidate",
            82,
            10,
            85,
            ["Security-sensitive evidence needs human disclosure review before public recommendations."]
          )
        ],
        publicSafety
      }
    });

    assert.equal(ledger.entries[0].route, "ask_human");
    assert.equal(ledger.entries[0].approvalState, "human_required");
    assert.equal(ledger.entries[0].redactionState, "sensitive");
  });

  it("redacts source-like raw repo content from public ledger summaries", () => {
    const ledger = routeFoundryDecisions({
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 1, byDecision: {}, prPreviewCandidates: 1, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 0 },
        assessments: [{
          ...assessment("fev-raw-source", "pr_preview_candidate", 80, 85, 85),
          publicRationale: [
            "Raw source context: export const privateSetting = process.env.PRIVATE_VALUE;"
          ],
          requiredVerification: [
            "Inspect function leakSecret() { return privateSetting; } before merge.",
            "Review brace-heavy snippet:\n{\n  privateToken\n}"
          ],
          blockers: [
            "Caller supplied raw source: const privateToken = getSecret();",
            "Brace-only raw source:\n{\n  privateBranch\n}"
          ]
        }],
        publicSafety
      }
    });
    const serialized = JSON.stringify(ledger);

    assert.equal(ledger.entries[0].route, "ask_human");
    assert.equal(ledger.entries[0].redactionState, "redacted");
    assert.equal(serialized.includes("privateSetting"), false);
    assert.equal(serialized.includes("privateToken"), false);
    assert.equal(serialized.includes("privateBranch"), false);
    assert.equal(serialized.includes("leakSecret"), false);
    assert.equal(serialized.includes("[redacted-raw-repo-content]"), true);
  });

  it("routes source-like verification strings to ask_human and omits raw ids and source fields", () => {
    const ledger = routeFoundryDecisions({
      ledgerId: "slice-322",
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 1, byDecision: {}, prPreviewCandidates: 1, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 0 },
        assessments: [{
          ...assessment("fev-verification-source", "pr_preview_candidate", 84, 85, 85),
          requiredVerification: [
            "Review export const repoSecret = process.env.REPO_SECRET before merge."
          ]
        }],
        publicSafety
      }
    });
    const serialized = JSON.stringify(ledger);

    assert.equal(ledger.ledgerId, "foundry-ledger");
    assert.equal(ledger.entries[0].route, "ask_human");
    assert.deepEqual(ledger.entries[0].evidenceIds, ["fde-001"]);
    assert.deepEqual(ledger.entries[0].source, { score: 84 });
    assert.equal(Object.hasOwn(ledger.entries[0].source, "actionabilityDecision"), false);
    assert.equal(serialized.includes("slice-322"), false);
    assert.equal(serialized.includes("fev-verification-source"), false);
    assert.equal(serialized.includes("REPO_SECRET"), false);
    assert.equal(serialized.includes("[redacted-raw-repo-content]"), true);
  });

  it("fails closed to ask_human when routing factors are missing or duplicated", () => {
    const base = assessment("fev-factors", "pr_preview_candidate", 84, 85, 85);
    const ledger = routeFoundryDecisions({
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 2, byDecision: {}, prPreviewCandidates: 2, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 0 },
        assessments: [
          {
            ...base,
            evidenceId: "fev-missing-factor",
            factors: base.factors.filter((factor) => factor.name !== "maintainer_value")
          },
          {
            ...base,
            evidenceId: "fev-duplicate-factor",
            factors: [...base.factors, factor("public_safety_risk", 95)]
          }
        ],
        publicSafety
      }
    });

    assert.deepEqual(ledger.entries.map((entry) => entry.route), ["ask_human", "ask_human"]);
    assert.deepEqual(ledger.entries.map((entry) => entry.approvalState), ["human_required", "human_required"]);
  });

  it("falls back to an empty ledger for malformed reports", () => {
    const ledger = routeFoundryDecisions({
      actionability: {
        schemaVersion: 1,
        summary: { totalFindings: 1, byDecision: {}, prPreviewCandidates: 1, askHuman: 0, exceptionCandidates: 0, noOpCandidates: 0, publicSafetyHolds: 0 },
        assessments: [{
          ...assessment("fev-bad-factor", "pr_preview_candidate", 80, 85, 85),
          factors: [{ name: "not_a_real_factor", score: 90, status: "strong", rationale: "bad" }]
        }]
      } as any
    });

    assert.equal(ledger.summary.totalEntries, 0);
    assert.deepEqual(ledger.entries, []);
  });
});

function assessment(
  evidenceId: string,
  decision: FoundryActionabilityReport["assessments"][number]["decision"],
  score: number,
  publicSafetyRisk: number,
  maintainerValue: number,
  blockers: string[] = [],
  requiredVerification: string[] = ["Re-run npm test before mutation approval."]
): FoundryActionabilityReport["assessments"][number] {
  return {
    evidenceId,
    sourceType: "architect_review",
    code: "ARCH008_ENV_SCATTER",
    path: "src/config/env.ts",
    decision,
    score,
    publicRationale: [`Decision ${decision} with score ${score}.`],
    requiredVerification,
    blockers,
    factors: [
      factor("public_safety_risk", publicSafetyRisk),
      factor("maintainer_value", maintainerValue),
      factor("verification_path", blockers.some((item) => item.includes("verification path")) ? 45 : 85)
    ]
  };
}

function factor(name: string, score: number) {
  return {
    name: name as any,
    score,
    status: score >= 75 ? "strong" as const : score >= 55 ? "mixed" as const : score >= 35 ? "weak" as const : "blocked" as const,
    rationale: `${name} rationale`
  };
}
