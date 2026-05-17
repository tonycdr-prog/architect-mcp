import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { auditWorkGateCompleteness, workGateSequence } from "../src/domain/workGateCompleteness.js";

const freshTimestamp = "2026-05-17T22:00:00.000Z";

describe("auditWorkGateCompleteness", () => {
  it("fails closed when no work-gate evidence is supplied", () => {
    const report = auditWorkGateCompleteness({ records: [] });

    assert.equal(report.status, "fail");
    assert.equal(report.classification, "no_evidence");
    assert.equal(report.readOnly, true);
    assert.equal(report.summary.present, 0);
    assert.equal(report.findings.some((finding) => finding.code === "WG001_NO_EVIDENCE"), true);
  });

  it("distinguishes partial evidence with missing gate steps", () => {
    const report = auditWorkGateCompleteness({
      records: [
        record("grill_me"),
        record("create_pre_edit_contract")
      ]
    });

    assert.equal(report.status, "fail");
    assert.equal(report.classification, "partial");
    assert.equal(report.summary.present, 2);
    assert.equal(report.findings.some((finding) => finding.code === "WG002_MISSING_GATE"), true);
  });

  it("flags stale evidence without inferring freshness from wording", () => {
    const report = auditWorkGateCompleteness({
      now: "2026-05-17T22:00:00.000Z",
      maxAgeSeconds: 60,
      records: workGateSequence.map((gate) => record(gate, {
        recordedAt: gate === "review_agent_session" ? "2026-05-17T21:00:00.000Z" : freshTimestamp
      }))
    });

    assert.equal(report.status, "fail");
    assert.equal(report.classification, "stale");
    assert.equal(report.summary.stale, 1);
    assert.equal(report.findings.some((finding) => finding.code === "WG004_STALE_EVIDENCE"), true);
  });

  it("passes complete fresh ordered evidence", () => {
    const report = auditWorkGateCompleteness({
      now: freshTimestamp,
      records: workGateSequence.map((gate) => record(gate))
    });

    assert.equal(report.status, "pass");
    assert.equal(report.classification, "complete");
    assert.equal(report.complete, true);
    assert.equal(report.summary.missing, 0);
    assert.equal(report.publicSummaryMarkdown.includes("read-only detection report"), true);
  });

  it("fails closed for unknown or out-of-order gate evidence", () => {
    const unknown = auditWorkGateCompleteness({
      records: [{ gate: "not_a_gate", status: "pass", recordedAt: freshTimestamp }]
    });
    const outOfOrder = auditWorkGateCompleteness({
      records: [
        record("review_agent_session"),
        record("grill_me")
      ]
    });

    assert.equal(unknown.status, "fail");
    assert.equal(unknown.classification, "unknown_gate");
    assert.equal(outOfOrder.status, "fail");
    assert.equal(outOfOrder.classification, "out_of_order");
  });
});

function record(gate: typeof workGateSequence[number], overrides: Partial<ReturnType<typeof recordShape>> = {}) {
  return {
    ...recordShape(gate),
    ...overrides
  };
}

function recordShape(gate: typeof workGateSequence[number]) {
  return {
    gate,
    status: "pass" as const,
    recordedAt: freshTimestamp,
    runId: "run-245"
  };
}
