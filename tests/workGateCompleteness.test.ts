import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { auditWorkGateCompleteness, workGateSequence } from "../src/domain/workGateCompleteness.js";
import { createWorkGateSequenceReceipt } from "../src/domain/workGateSequenceReceipt.js";

const freshTimestamp = "2026-05-17T22:00:00.000Z";

describe("auditWorkGateCompleteness", () => {
  it("fails closed when no work-gate evidence is supplied", () => {
    const report = auditWorkGateCompleteness({ records: [] });

    assert.equal(report.status, "fail");
    assert.equal(report.classification, "missing");
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

  it("does not treat run ids alone as fresh evidence", () => {
    const report = auditWorkGateCompleteness({
      now: freshTimestamp,
      records: workGateSequence.map((gate) => record(gate, {
        recordedAt: undefined,
        runId: "run-without-timestamp"
      }))
    });

    assert.equal(report.status, "warn");
    assert.equal(report.classification, "incomplete");
    assert.equal(report.complete, false);
    assert.equal(report.summary.warnings, workGateSequence.length);
    assert.equal(report.findings.every((finding) => finding.code === "WG007_FRESHNESS_UNKNOWN"), true);
  });

  it("classifies caller-narrowed required gates as partial evidence", () => {
    const report = auditWorkGateCompleteness({
      now: freshTimestamp,
      requiredGates: ["grill_me", "create_pre_edit_contract"],
      records: [
        record("grill_me"),
        record("create_pre_edit_contract")
      ]
    });

    assert.equal(report.status, "fail");
    assert.equal(report.classification, "partial");
    assert.equal(report.complete, false);
    assert.equal(report.requiredGates.length, workGateSequence.length);
    assert.equal(report.findings.some((finding) => finding.code === "WG012_PARTIAL_REQUIRED_GATES"), true);
  });

  it("reports invalid audit reference times instead of silently falling back", () => {
    const report = auditWorkGateCompleteness({
      now: "not a timestamp",
      records: workGateSequence.map((gate) => record(gate))
    });

    assert.equal(report.status, "fail");
    assert.equal(report.complete, false);
    assert.equal(report.findings.some((finding) => finding.code === "WG013_INVALID_NOW"), true);
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
    assert.equal(report.gateResults.every((result) => result.recordedAt === "[redacted]" && result.runId === "[redacted]"), true);
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
    assert.equal(unknown.findings.some((finding) => finding.message.includes("not_a_gate")), false);
    assert.equal(outOfOrder.status, "fail");
    assert.equal(outOfOrder.classification, "out_of_order");
  });
});

describe("createWorkGateSequenceReceipt", () => {
  it("creates a complete public-safe receipt for ordered gate evidence", () => {
    const receipt = createWorkGateSequenceReceipt({
      now: freshTimestamp,
      records: workGateSequence.map((gate) => receiptRecord(gate))
    });

    assert.equal(receipt.status, "pass");
    assert.equal(receipt.classification, "complete");
    assert.equal(receipt.complete, true);
    assert.equal(receipt.readOnly, true);
    assert.equal(receipt.enforcement, "detection-only");
    assert.equal(receipt.summary.inputsUnconfirmed, 0);
    assert.equal(receipt.summary.evidenceUnconfirmed, 0);
    assert.equal(receipt.receipt.kind, "work_gate_sequence_receipt");
    assert.equal(receipt.steps.every((step) => step.inputsPresent && step.evidencePresent), true);
    assert.equal(receipt.steps.every((step) => step.recordedAtPresent === true && !("recordedAt" in step)), true);
    assert.doesNotMatch(JSON.stringify(receipt.receipt), /2026-05-17T22:00:00.000Z/);
  });

  it("fails when a required gate is missing from the receipt", () => {
    const receipt = createWorkGateSequenceReceipt({
      now: freshTimestamp,
      records: [
        receiptRecord("grill_me"),
        receiptRecord("create_pre_edit_contract")
      ]
    });

    assert.equal(receipt.status, "fail");
    assert.equal(receipt.classification, "partial");
    assert.deepEqual(receipt.missingSteps.slice(0, 2), ["review_build_plan", "review_proposed_file_plan"]);
  });

  it("fails closed for out-of-order, unknown, and stale gate records", () => {
    const outOfOrder = createWorkGateSequenceReceipt({
      records: [
        receiptRecord("review_agent_session"),
        receiptRecord("grill_me")
      ]
    });
    const unknown = createWorkGateSequenceReceipt({
      records: [{
        gate: "not_a_gate",
        status: "pass",
        recordedAt: freshTimestamp,
        inputsPresent: true,
        evidencePresent: true
      }]
    });
    const stale = createWorkGateSequenceReceipt({
      now: freshTimestamp,
      maxAgeSeconds: 60,
      records: workGateSequence.map((gate) => receiptRecord(gate, {
        recordedAt: gate === "review_agent_session" ? "2026-05-17T21:00:00.000Z" : freshTimestamp
      }))
    });

    assert.equal(outOfOrder.status, "fail");
    assert.equal(outOfOrder.classification, "out_of_order");
    assert.equal(unknown.status, "fail");
    assert.equal(unknown.classification, "unknown_gate");
    assert.equal(stale.status, "fail");
    assert.equal(stale.classification, "stale");
  });

  it("requires explicit input and evidence confirmation for each supplied gate", () => {
    const receipt = createWorkGateSequenceReceipt({
      records: workGateSequence.map((gate) => receiptRecord(gate, {
        inputsPresent: gate !== "review_build_plan",
        evidencePresent: gate !== "review_proposed_file_plan"
      }))
    });

    assert.equal(receipt.status, "fail");
    assert.equal(receipt.classification, "incomplete");
    assert.equal(receipt.summary.inputsUnconfirmed, 1);
    assert.equal(receipt.summary.evidenceUnconfirmed, 1);
    assert.equal(receipt.findings.some((finding) => finding.code === "WG008_INPUTS_UNCONFIRMED"), true);
    assert.equal(receipt.findings.some((finding) => finding.code === "WG009_EVIDENCE_UNCONFIRMED"), true);
  });

  it("redacts token-shaped values and local paths from public receipt summaries", () => {
    const receipt = createWorkGateSequenceReceipt({
      records: workGateSequence.map((gate) => receiptRecord(gate, {
        publicSummary: gate === "grill_me"
          ? "Reviewed /Users/example/private/repo with npm_abcdefghijklmnopqrstuvwxyz123456"
          : "Reviewed public-safe evidence."
      }))
    });

    assert.equal(receipt.status, "warn");
    assert.equal(receipt.summary.redacted, 1);
    assert.match(receipt.steps[0].publicSummary ?? "", /\[redacted-local-path\]/);
    assert.match(receipt.steps[0].publicSummary ?? "", /\[redacted-token\]/);
    assert.doesNotMatch(JSON.stringify(receipt), /abcdefghijklmnopqrstuvwxyz123456|\/Users\/example/);
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

function receiptRecord(gate: typeof workGateSequence[number], overrides: Partial<ReturnType<typeof receiptRecordShape>> = {}) {
  return {
    ...receiptRecordShape(gate),
    ...overrides
  };
}

function receiptRecordShape(gate: typeof workGateSequence[number]) {
  return {
    gate,
    status: "pass" as const,
    recordedAt: freshTimestamp,
    runId: "run-247",
    inputsPresent: true,
    evidencePresent: true,
    publicSummary: "Reviewed public-safe gate evidence."
  };
}
