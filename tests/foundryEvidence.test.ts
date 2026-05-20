import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { normalizeFoundryEvidence } from "../src/domain/foundryEvidence.js";
import { deriveRepoConstitution } from "../src/domain/repoConstitution.js";
import { createReviewReport } from "../src/domain/reviewReport.js";
import type { ReviewViolation } from "../src/domain/types.js";

describe("normalizeFoundryEvidence", () => {
  it("normalizes review, external, verification, constitution, coverage, and suppression evidence", () => {
    const findings: ReviewViolation[] = [
      {
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "medium",
        severity: "warning",
        path: "docs/examples/demo.mdx",
        message: "File has 1200 lines and may be too large.",
        recommendation: "Review whether this docs example should be excluded from actionability."
      },
      {
        code: "ARCH026_REPO_HYGIENE",
        confidence: "medium",
        severity: "warning",
        path: "index.ts",
        message: "Root source entrypoint detected.",
        recommendation: "Check whether this is a conventional package entrypoint before routing."
      }
    ];
    const report = createReviewReport(findings, {
      mode: "audit",
      maxDetailedFindings: 1,
      scan: {
        filesReviewed: 5000,
        maxFiles: 5000,
        truncated: true,
        topScannedDirectories: [{ directory: "docs", files: 3000 }]
      }
    });
    const constitution = deriveRepoConstitution({
      files: [
        { path: ".github/pull_request_template.md", lines: 5 },
        { path: ".github/workflows/ci.yml", lines: 5 }
      ],
      artifacts: [
        { path: ".github/pull_request_template.md", content: "## Summary\n\n## Verification\n" },
        { path: ".github/workflows/ci.yml", content: "name: CI\non: push\n" }
      ]
    });

    const inventory = normalizeFoundryEvidence({
      findings: [findings[1]],
      reviewReports: [report],
      externalFindings: [
        {
          toolName: "secret-scanner",
          ruleId: "secret",
          confidence: "high",
          severity: "error",
          path: "/Users/tony/project/.env",
          message: "Possible token npm_abcdefghijklmnopqrstuvwxyz1234567890 found in raw output.",
          recommendation: "Do not disclose raw scanner payload publicly.",
          rawPayload: { secret: "do-not-return-this" },
          securitySensitive: true
        }
      ],
      verification: [
        { check: "npm test", status: "passed", summary: "301 tests passed", recordedAt: "2026-05-20T00:00:00Z" }
      ],
      repoConstitution: constitution
    });

    assert.equal(inventory.schemaVersion, 1);
    assert.equal(inventory.publicSafety.rawPayloadsIncluded, false);
    assert.equal(inventory.publicSafety.rawRepoContentIncluded, false);
    assert.equal(inventory.publicSafety.localPathsIncluded, false);
    assert.equal(inventory.publicSafety.tokenValuesIncluded, false);
    assert.equal(inventory.publicSafety.mutationAllowed, false);
    assert.equal(JSON.stringify(inventory).includes("do-not-return-this"), false);
    assert.equal(inventory.evidence[0].id, "fev-0001");
    assert.equal(inventory.coverage.scanTruncated, true);
    assert.equal(inventory.coverage.detailedFindingsTruncated, true);
    assert.equal(inventory.coverage.findingHistogram.some((entry) => entry.code === "ARCH001_OVERSIZED_FILE"), true);
    assert.equal(inventory.summary.omittedRawPayloads, 1);
    assert.equal(inventory.summary.bySourceType.architect_review, 2);
    assert.equal(inventory.summary.bySourceType.external_tool, 1);
    assert.equal(inventory.summary.bySourceType.verification, 1);
    assert.equal(inventory.summary.bySourceType.repo_constitution >= 1, true);
    assert.equal(inventory.evidence.some((item) => item.publicSafetyClass === "sensitive"), true);
    assert.equal(inventory.evidence.some((item) => item.path === "[redacted-local-path]"), true);
    assert.equal(inventory.evidence.some((item) => item.suppressionCandidate?.category === "docs_example"), true);
    assert.equal(inventory.evidence.some((item) => item.suppressionCandidate?.category === "conventional_entrypoint"), true);
    assert.equal(inventory.suppressionPrerequisites.some((item) => item.issue.endsWith("/313")), true);
    assert.equal(inventory.suppressionPrerequisites.some((item) => item.category === "repo_profile_mismatch"), true);
  });

  it("redacts caller-controlled source ids and fallback codes", () => {
    const inventory = normalizeFoundryEvidence({
      reviewReports: [
        {
          priorityFindings: [],
          violations: [],
          coverage: {
            scanTruncated: false,
            detailedFindingsTruncated: false,
            filesReviewed: 2,
            maxFiles: 2,
            topScannedDirectories: [
              { directory: "/Users/alice/project/npm_abcdefghijklmnopqrstuvwxyz1234567890", files: 1 },
              { directory: "/Users/bob/project/npm_abcdefghijklmnopqrstuvwxyz1234567890", files: 1 }
            ],
            findingHistogram: [
              { code: "/Users/alice/rule/npm_abcdefghijklmnopqrstuvwxyz1234567890", severity: "warning", count: 1 },
              { code: "/Users/bob/rule/npm_abcdefghijklmnopqrstuvwxyz1234567890", severity: "warning", count: 1 }
            ],
            caveats: [
              "coverage caveat from /Users/alice/project with npm_abcdefghijklmnopqrstuvwxyz1234567890",
              "coverage caveat from /Users/bob/project with npm_abcdefghijklmnopqrstuvwxyz1234567890"
            ]
          }
        } as any
      ],
      externalFindings: [
        {
          toolName: "/Users/alice/secret/npm_abcdefghijklmnopqrstuvwxyz1234567890",
          path: "src/index.ts",
          message: "Tool name and fallback code should be sanitized.",
          rawPayload: { token: "raw payload omitted" }
        }
      ],
      verification: [
        {
          check: "npm test from /Users/alice/project with npm_abcdefghijklmnopqrstuvwxyz1234567890",
          status: "failed",
          summary: "stderr=/Users/alice/private.log"
        }
      ]
    });

    const serialized = JSON.stringify(inventory);
    assert.equal(serialized.includes("/Users/alice"), false);
    assert.equal(serialized.includes("npm_abcdefghijklmnopqrstuvwxyz1234567890"), false);
    assert.equal(serialized.includes("raw payload omitted"), false);
    assert.equal(inventory.summary.redacted >= 2, true);
    assert.equal(inventory.summary.omittedRawPayloads, 1);
    assert.equal(inventory.evidence.some((item) => item.sourceRef.sourceId.includes("[redacted")), true);
    assert.equal(inventory.evidence.some((item) => item.code?.includes("[redacted")), true);
    assert.equal(inventory.coverage.caveats.includes("Some redacted top-scanned directory entries were merged to preserve public safety."), true);
    assert.equal(inventory.coverage.caveats.includes("Some redacted finding-histogram buckets were merged to preserve public safety."), true);
    assert.equal(inventory.coverage.caveats.includes("Some redacted coverage caveats were merged to preserve public safety."), true);
  });

  it("dedupes duplicated review findings across priorityFindings and violations", () => {
    const inventory = normalizeFoundryEvidence({
      reviewReports: [
        {
          priorityFindings: [
            {
              code: "ARCH001_OVERSIZED_FILE",
              confidence: "medium",
              severity: "warning",
              path: "src/index.ts",
              message: "Duplicate finding",
              recommendation: "Review."
            }
          ],
          violations: [
            {
              code: "ARCH001_OVERSIZED_FILE",
              confidence: "medium",
              severity: "warning",
              path: "src/index.ts",
              message: "Duplicate finding",
              recommendation: "Review."
            }
          ]
        } as any
      ]
    });

    assert.equal(inventory.evidence.filter((item) => item.code === "ARCH001_OVERSIZED_FILE").length, 1);
    assert.equal(inventory.summary.bySourceType.architect_review, 1);
  });

  it("ignores malformed repo constitution objects instead of throwing", () => {
    const inventory = normalizeFoundryEvidence({ repoConstitution: {} as any });

    assert.equal(inventory.summary.totalEvidence, 0);
    assert.deepEqual(inventory.summary.bySourceType, {});
  });
});
