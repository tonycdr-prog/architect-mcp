import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { reviewVerificationReceipts, type VerificationReceipt } from "../src/domain/verificationReceipts.js";

const now = "2026-05-17T22:30:00.000Z";

describe("reviewVerificationReceipts", () => {
  it("warns when receipts are explicitly expected but missing", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      receipts: [],
      now
    });

    assert.equal(report.status, "warn");
    assert.equal(report.summary.missingRequired, 1);
    assert.equal(report.findings.some((finding) => finding.code === "VERIFY_RECEIPT001_MISSING"), true);
  });

  it("fails stale, failed, skipped, and not-run receipts", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test", "npm run build", "npm run typecheck"],
      now,
      maxAgeSeconds: 60,
      receipts: [
        receipt("npm test", { recordedAt: "2026-05-17T21:00:00.000Z" }),
        receipt("npm run build", { status: "failed" }),
        receipt("npm run typecheck", { status: "skipped" }),
        receipt("npm audit", { status: "not_run" })
      ]
    });

    assert.equal(report.status, "fail");
    assert.equal(report.findings.some((finding) => finding.code === "VERIFY_RECEIPT002_FAILED"), true);
    assert.equal(report.findings.some((finding) => finding.code === "VERIFY_RECEIPT003_INCOMPLETE"), true);
    assert.equal(report.findings.some((finding) => finding.code === "VERIFY_RECEIPT005_STALE"), true);
  });

  it("passes matching fresh receipts and summarizes verification records separately", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      verification: [{ check: "npm test", status: "passed", note: "recorded by session" }],
      receipts: [receipt("npm test")],
      now
    });

    assert.equal(report.status, "pass");
    assert.equal(report.complete, true);
    assert.equal(report.summary.verificationRecords.passed, 1);
    assert.equal(report.summary.matchedRequired, 1);
  });

  it("redacts token-shaped values and local paths from public receipt summaries", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      receipts: [
        receipt("npm test", {
          summary: "Passed in /Users/example/project with npm_123456789012345678901234567890"
        })
      ],
      now
    });

    assert.equal(report.status, "warn");
    assert.equal(report.summary.redacted, 1);
    assert.match(report.receipts[0].publicSafeSummary, /\[redacted-local-path\]/);
    assert.match(report.receipts[0].publicSafeSummary, /\[redacted-token\]/);
    assert.doesNotMatch(JSON.stringify(report), /npm_123456789012345678901234567890|\/Users\/example/);
  });

  it("redacts required checks, claimed checks, records, and receipt timing from public output", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["/Users/example/project/scripts/check npm_123456789012345678901234567890"],
      verification: [{
        check: "npm_123456789012345678901234567890",
        status: "passed",
        note: "local run in /Users/example/project"
      }],
      receipts: [receipt("npm test")],
      now
    });

    const serialized = JSON.stringify(report);
    assert.equal(report.status, "warn");
    assert.equal(report.claimedChecks[0].redacted, true);
    assert.equal(report.verificationRecords[0].redacted, true);
    assert.equal(report.receipts[0].recordedAtPresent, true);
    assert.equal("recordedAt" in report.receipts[0], false);
    assert.match(report.findings.find((finding) => finding.code === "VERIFY_RECEIPT001_MISSING")?.message ?? "", /\[redacted-local-path\]/);
    assert.doesNotMatch(serialized, /npm_123456789012345678901234567890|\/Users\/example|2026-05-17T22:30:00.000Z/);
  });
});

function receipt(command: string, overrides: Partial<VerificationReceipt> = {}) {
  return {
    command,
    status: "passed" as const,
    source: "local_terminal" as const,
    summary: `${command} passed`,
    recordedAt: now,
    ...overrides
  };
}
