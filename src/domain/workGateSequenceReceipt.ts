import {
  auditWorkGateCompleteness,
  workGateSequence,
  type WorkGateEvidenceStatus,
  type WorkGateName
} from "./workGateCompleteness.js";

export type WorkGateSequenceReceiptRecord = {
  gate: string;
  status?: WorkGateEvidenceStatus;
  recordedAt?: string;
  runId?: string;
  inputsPresent?: boolean;
  evidencePresent?: boolean;
  publicSummary?: string;
};

export type WorkGateSequenceReceiptInput = {
  records?: WorkGateSequenceReceiptRecord[];
  requiredGates?: WorkGateName[];
  now?: string;
  maxAgeSeconds?: number;
};

type SequenceReceiptFinding = {
  code: string;
  severity: "error" | "warning";
  message: string;
  recommendation: string;
};

export function createWorkGateSequenceReceipt(input: WorkGateSequenceReceiptInput = {}) {
  const records = input.records ?? [];
  const audit = auditWorkGateCompleteness({
    records,
    requiredGates: input.requiredGates,
    now: input.now,
    maxAgeSeconds: input.maxAgeSeconds
  });
  const requiredGates = audit.requiredGates;
  const recordsByGate = new Map<string, WorkGateSequenceReceiptRecord>();
  const duplicateGates = new Set<string>();
  for (const record of records) {
    if (recordsByGate.has(record.gate)) duplicateGates.add(record.gate);
    if (!recordsByGate.has(record.gate)) recordsByGate.set(record.gate, record);
  }

  const findings: SequenceReceiptFinding[] = [...audit.findings];
  for (const gate of requiredGates) {
    const record = recordsByGate.get(gate);
    if (!record) continue;
    if (record.inputsPresent !== true) {
      findings.push({
        code: "WG008_INPUTS_UNCONFIRMED",
        severity: "error",
        message: `Work-gate receipt for ${gate} does not confirm required inputs were present.`,
        recommendation: "Record explicit input presence for each gate before claiming a complete sequence receipt."
      });
    }
    if (record.evidencePresent !== true) {
      findings.push({
        code: "WG009_EVIDENCE_UNCONFIRMED",
        severity: "error",
        message: `Work-gate receipt for ${gate} does not confirm review evidence was present.`,
        recommendation: "Record explicit evidence presence for each gate before claiming a complete sequence receipt."
      });
    }
  }

  for (const gate of duplicateGates) {
    if (!isWorkGateName(gate)) continue;
    findings.push({
      code: "WG010_DUPLICATE_GATE",
      severity: "warning",
      message: `Work-gate receipt includes duplicate records for ${gate}; only the first record is summarized.`,
      recommendation: "Send one public-safe receipt record per gate, in the order the gates ran."
    });
  }

  const steps = requiredGates.map((gate, index) => {
    const record = recordsByGate.get(gate);
    const safeSummary = sanitizePublicSummary(record?.publicSummary);
    return {
      gate,
      order: index + 1,
      present: Boolean(record),
      status: record?.status ?? "missing",
      inputsPresent: record?.inputsPresent === true,
      evidencePresent: record?.evidencePresent === true,
      recordedAt: record?.recordedAt,
      runIdPresent: Boolean(record?.runId),
      publicSummary: safeSummary.value,
      redacted: safeSummary.redacted
    };
  });

  const redacted = steps.filter((step) => step.redacted).length;
  if (redacted > 0) {
    findings.push({
      code: "WG011_PUBLIC_SUMMARY_REDACTED",
      severity: "warning",
      message: "One or more work-gate receipt summaries contained token-shaped values or local paths that were redacted.",
      recommendation: "Keep public receipt summaries short and avoid raw payloads, local paths, tokens, stdout, or stderr."
    });
  }

  const errors = findings.filter((finding) => finding.severity === "error").length;
  const warnings = findings.length - errors;
  const status = errors > 0 ? "fail" : warnings > 0 ? "warn" : "pass";
  const classification = status === "pass" ? "complete" : audit.classification === "complete" ? "incomplete" : audit.classification;

  return {
    status,
    classification,
    readOnly: true,
    enforcement: "detection-only",
    publicSafe: true,
    complete: status === "pass",
    summary: {
      ...audit.summary,
      supplied: records.length,
      required: requiredGates.length,
      missing: steps.filter((step) => !step.present).length,
      inputsUnconfirmed: steps.filter((step) => step.present && !step.inputsPresent).length,
      evidenceUnconfirmed: steps.filter((step) => step.present && !step.evidencePresent).length,
      redacted,
      errors,
      warnings
    },
    requiredGates,
    missingSteps: steps.filter((step) => !step.present).map((step) => step.gate),
    steps,
    findings,
    receipt: {
      kind: "work_gate_sequence_receipt",
      version: "1.0",
      publicSafe: true,
      readOnly: true,
      enforcement: "detection-only",
      requiredGates,
      steps
    },
    publicSummaryMarkdown: `Work-gate sequence receipt: ${status} (${steps.filter((step) => step.present).length}/${requiredGates.length} required gates present, ${steps.filter((step) => step.inputsPresent && step.evidencePresent).length}/${requiredGates.length} with confirmed inputs and evidence). This is a read-only detection report, not an enforcement sandbox.`
  };
}

function isWorkGateName(value: string): value is WorkGateName {
  return (workGateSequence as readonly string[]).includes(value);
}

function sanitizePublicSummary(value: string | undefined): { value: string | undefined; redacted: boolean } {
  if (!value) return { value: undefined, redacted: false };
  let redacted = false;
  let safe = value.trim();
  const replacements: Array<[RegExp, string]> = [
    [/\b(?:npm|gh[pousr]|github_pat|sk|xox[baprs])_[A-Za-z0-9_=-]{16,}\b/g, "[redacted-token]"],
    [/\b[A-Za-z0-9_-]{48,}\b/g, "[redacted-token]"],
    [/(?:\/Users|\/home|\/private\/tmp|\/tmp|\/var\/folders|\/Volumes)\/[^\s,;)"']+/g, "[redacted-local-path]"],
    [/\b[A-Za-z]:\\[^\s,;)"']+/g, "[redacted-local-path]"]
  ];
  for (const [pattern, replacement] of replacements) {
    safe = safe.replace(pattern, () => {
      redacted = true;
      return replacement;
    });
  }
  return { value: safe, redacted };
}
