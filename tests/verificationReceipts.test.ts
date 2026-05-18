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
    assert.equal(report.summary.freshMatchedRequired, 1);
    assert.equal(report.summary.evidenceTiers.supplied, 2);
    assert.equal(report.receipts[0].evidenceTier, "supplied");
    assert.equal(report.receipts[0].freshness.status, "fresh");
    assert.equal(report.claimedChecks[0].freshReceiptSupplied, true);
  });

  it("downgrades supplied local receipts without recordedAt from complete fresh evidence", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      receipts: [receipt("npm test", { recordedAt: undefined })],
      now
    });

    assert.equal(report.status, "warn");
    assert.equal(report.complete, false);
    assert.equal(report.summary.matchedRequired, 1);
    assert.equal(report.summary.freshMatchedRequired, 0);
    assert.equal(report.summary.missingFreshRequired, 1);
    assert.equal(report.receipts[0].evidenceTier, "supplied");
    assert.equal(report.receipts[0].freshness.status, "missing");
    assert.equal(report.receipts[0].freshness.satisfied, false);
    assert.equal(report.claimedChecks[0].receiptSupplied, true);
    assert.equal(report.claimedChecks[0].freshReceiptSupplied, false);
    assert.equal(report.findings.some((finding) => finding.code === "VERIFY_RECEIPT004_FRESHNESS_UNKNOWN"), true);
  });

  it("does not treat arbitrary local run ids as independent proof", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      receipts: [receipt("npm test", { recordedAt: undefined, runId: "local-run-284" })],
      now
    });

    assert.equal(report.status, "warn");
    assert.equal(report.complete, false);
    assert.equal(report.summary.freshMatchedRequired, 0);
    assert.equal(report.receipts[0].runIdPresent, true);
    assert.equal(report.receipts[0].independentlyResolvable, false);
    assert.equal(report.receipts[0].freshness.status, "missing");
    assert.match(report.findings.find((finding) => finding.code === "VERIFY_RECEIPT004_FRESHNESS_UNKNOWN")?.message ?? "", /not independently resolvable/);
  });

  it("treats CI run id receipts as independent evidence when timestamps are absent", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      receipts: [receipt("npm test", {
        source: "ci",
        recordedAt: undefined,
        runId: "ci-run-284",
        summary: "CI job passed"
      })],
      now
    });

    assert.equal(report.status, "pass");
    assert.equal(report.complete, true);
    assert.equal(report.summary.freshMatchedRequired, 1);
    assert.equal(report.summary.evidenceTiers.independent, 1);
    assert.equal(report.receipts[0].evidenceTier, "independent");
    assert.equal(report.receipts[0].independentlyResolvable, true);
    assert.equal(report.receipts[0].freshness.status, "independent_run_id");
    assert.equal(report.claimedChecks[0].independentReceiptSupplied, true);
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

  it("redacts fenced raw output and stdout/stderr markers from receipt summaries", () => {
    const report = reviewVerificationReceipts({
      requiredChecks: ["npm test"],
      receipts: [
        receipt("npm test", {
          summary: "stdout:\n```json\n{\"private\":\"payload\"}\n```"
        })
      ],
      now
    });

    assert.equal(report.status, "warn");
    assert.equal(report.receipts[0].publicSafeSummary, "[redacted-raw-output]");
    assert.equal(report.receipts[0].redacted, true);
    assert.doesNotMatch(JSON.stringify(report), /```json/);
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
