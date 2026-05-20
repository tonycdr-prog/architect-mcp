import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { scoreFoundryActionability } from "../src/domain/foundryActionability.js";
import { normalizeFoundryEvidence } from "../src/domain/foundryEvidence.js";
import { deriveRepoConstitution } from "../src/domain/repoConstitution.js";
import type { ReviewViolation } from "../src/domain/types.js";

describe("scoreFoundryActionability", () => {
  it("scores high-evidence low-blast findings as PR-preview candidates", () => {
    const finding: ReviewViolation = {
      code: "ARCH008_ENV_SCATTER",
      confidence: "high",
      severity: "error",
      path: "src/auth/session.ts",
      message: "Environment access is scattered through a server module.",
      recommendation: "Centralize environment parsing behind the existing server boundary."
    };
    const inventory = normalizeFoundryEvidence({
      reviewReports: [{
        priorityFindings: [],
        violations: [],
        coverage: {
          scanTruncated: false,
          detailedFindingsTruncated: true,
          filesReviewed: 50,
          maxFiles: 1000,
          topScannedDirectories: [],
          findingHistogram: []
        }
      } as any],
      findings: [finding],
      verification: [{ check: "npm test", status: "passed", summary: "passed" }],
      repoConstitution: deriveRepoConstitution({
        files: [{ path: ".github/pull_request_template.md", lines: 4 }],
        artifacts: [{ path: ".github/pull_request_template.md", content: "## Summary\n\n## Verification\n" }]
      })
    });

    const report = scoreFoundryActionability({ inventory });
    const assessment = report.assessments.find((item) => item.code === "ARCH008_ENV_SCATTER");

    assert.equal(report.schemaVersion, 1);
    assert.equal(report.publicSafety.mutationAllowed, false);
    assert.equal(assessment?.decision, "pr_preview_candidate");
    assert.equal((assessment?.score ?? 0) >= 70, true);
    assert.equal(assessment?.requiredVerification.some((item) => item.includes("npm test")), true);
    assert.equal(report.summary.prPreviewCandidates, 1);
  });

  it("routes noisy suppressible findings away from PR previews", () => {
    const inventory = normalizeFoundryEvidence({
      findings: [{
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "medium",
        severity: "warning",
        path: "docs/examples/demo.mdx",
        message: "Documentation example is oversized.",
        recommendation: "Check whether docs examples should be excluded from actionability."
      }]
    });

    const report = scoreFoundryActionability({ inventory });

    assert.equal(report.assessments[0].decision, "exception_candidate");
    assert.equal(report.assessments[0].blockers.some((item) => item.includes("docs_example")), true);
    assert.equal(report.summary.exceptionCandidates, 1);
  });

  it("keeps security-sensitive and redacted findings behind human review", () => {
    const inventory = normalizeFoundryEvidence({
      externalFindings: [{
        toolName: "/Users/alice/scanner/npm_abcdefghijklmnopqrstuvwxyz1234567890",
        severity: "error",
        confidence: "high",
        path: "/Users/alice/project/.env",
        message: "Secret-shaped value found in raw scanner output.",
        rawPayload: { secret: "npm_abcdefghijklmnopqrstuvwxyz1234567890" },
        securitySensitive: true
      }],
      verification: [{ check: "security review", status: "passed" }]
    });

    const report = scoreFoundryActionability({
      inventory,
      verificationHints: ["/Users/alice/project npm_abcdefghijklmnopqrstuvwxyz1234567890"]
    });
    const serialized = JSON.stringify(report);

    assert.equal(report.assessments[0].decision, "ask_human");
    assert.equal(report.summary.publicSafetyHolds, 1);
    assert.equal(serialized.includes("/Users/alice"), false);
    assert.equal(serialized.includes("npm_abcdefghijklmnopqrstuvwxyz1234567890"), false);
  });

  it("classifies low maintainer-value findings as no-op candidates", () => {
    const inventory = normalizeFoundryEvidence({
      externalFindings: [{
        toolName: "lint",
        ruleId: "style-note",
        severity: "info",
        confidence: "high",
        path: "src/style.ts",
        message: "Minor style preference.",
        recommendation: "Do not interrupt maintainers for this alone."
      }],
      verification: [{ check: "npm test", status: "passed" }]
    });

    const report = scoreFoundryActionability({ inventory });

    assert.equal(report.assessments[0].decision, "no_op_candidate");
    assert.equal(report.summary.noOpCandidates, 1);
  });
});
