export const workGateSequence = [
  "grill_me",
  "create_pre_edit_contract",
  "review_build_plan",
  "review_proposed_file_plan",
  "review_repo_structure",
  "review_implementation_against_contract",
  "review_agent_final_response",
  "review_agent_session"
] as const;

export type WorkGateName = typeof workGateSequence[number];
export type WorkGateEvidenceStatus = "pass" | "warn" | "fail" | "unknown";

export type WorkGateEvidenceRecord = {
  gate: string;
  status?: WorkGateEvidenceStatus;
  recordedAt?: string;
  runId?: string;
};

export type WorkGateCompletenessInput = {
  records?: WorkGateEvidenceRecord[];
  requiredGates?: WorkGateName[];
  now?: string;
  maxAgeSeconds?: number;
};

type WorkGateFinding = {
  code: string;
  severity: "error" | "warning";
  message: string;
  recommendation: string;
};

type WorkGateClassification =
  | "missing"
  | "partial"
  | "stale"
  | "out_of_order"
  | "complete"
  | "unknown_gate"
  | "incomplete";

const DEFAULT_MAX_AGE_SECONDS = 7 * 24 * 60 * 60;

export function auditWorkGateCompleteness(input: WorkGateCompletenessInput = {}) {
  const requiredGates = input.requiredGates?.length ? input.requiredGates : [...workGateSequence];
  const records = input.records ?? [];
  const nowMs = parseTimestamp(input.now) ?? Date.now();
  const maxAgeSeconds = input.maxAgeSeconds ?? DEFAULT_MAX_AGE_SECONDS;
  const findings: WorkGateFinding[] = [];

  const unknownGates = records
    .map((record) => record.gate)
    .filter((gate) => !isWorkGateName(gate));
  if (unknownGates.length > 0) {
    findings.push({
      code: "WG005_UNKNOWN_GATE",
      severity: "error",
      message: `Unknown work-gate evidence records supplied (${new Set(unknownGates).size}).`,
      recommendation: "Use only registered work-gate names so reviewers do not mistake arbitrary labels for gate evidence."
    });
  }

  if (records.length === 0) {
    findings.push({
      code: "WG001_NO_EVIDENCE",
      severity: "error",
      message: "No work-gate evidence records were supplied.",
      recommendation: "Attach public-safe records for the required gate sequence before claiming a complete work-gate run."
    });
  }

  const validRecords = records.filter((record): record is WorkGateEvidenceRecord & { gate: WorkGateName } => isWorkGateName(record.gate));
  const recordsByGate = new Map<WorkGateName, WorkGateEvidenceRecord>();
  for (const record of validRecords) {
    if (!recordsByGate.has(record.gate)) recordsByGate.set(record.gate, record);
  }

  const missing = requiredGates.filter((gate) => !recordsByGate.has(gate));
  for (const gate of missing) {
    findings.push({
      code: "WG002_MISSING_GATE",
      severity: "error",
      message: `Missing required work-gate evidence: ${gate}.`,
      recommendation: "Run the full work-gate sequence or state clearly that this is only partial evidence."
    });
  }

  for (const gate of requiredGates) {
    const record = recordsByGate.get(gate);
    if (!record) continue;
    if (record.status !== "pass") {
      findings.push({
        code: "WG003_GATE_NOT_PASSED",
        severity: "error",
        message: `Work-gate evidence for ${gate} is not a passing record.`,
        recommendation: "Do not treat skipped, failed, warning, missing, or unknown gate results as complete evidence."
      });
    }
    if (record.recordedAt) {
      const recordedAtMs = parseTimestamp(record.recordedAt);
      if (recordedAtMs === undefined || recordedAtMs > nowMs || nowMs - recordedAtMs > maxAgeSeconds * 1000) {
        findings.push({
          code: "WG004_STALE_EVIDENCE",
          severity: "error",
          message: `Work-gate evidence for ${gate} is stale or has an invalid timestamp.`,
          recommendation: "Attach fresh gate evidence from the current run before using it in PR or launch evidence."
        });
      }
    } else if (!record.runId) {
      findings.push({
        code: "WG007_FRESHNESS_UNKNOWN",
        severity: "warning",
        message: `Work-gate evidence for ${gate} has no timestamp or run id.`,
        recommendation: "Include a public-safe timestamp or run id so reviewers can tell whether evidence is fresh."
      });
    }
  }

  if (!isOrdered(validRecords.map((record) => record.gate), requiredGates)) {
    findings.push({
      code: "WG006_OUT_OF_ORDER",
      severity: "error",
      message: "Work-gate evidence appears out of the required order.",
      recommendation: "Record gates in the order they ran, or use the TUI-managed session evidence path."
    });
  }

  const errors = findings.filter((finding) => finding.severity === "error").length;
  const warnings = findings.length - errors;
  const status = errors > 0 ? "fail" : warnings > 0 ? "warn" : "pass";
  const classification = classify(requiredGates.length - missing.length, requiredGates.length, findings, status);
  const gateResults = requiredGates.map((gate) => {
    const record = recordsByGate.get(gate);
    return {
      gate,
      present: Boolean(record),
      status: record?.status ?? "missing",
      recordedAt: record?.recordedAt ? "[redacted]" : undefined,
      runId: record?.runId ? "[redacted]" : undefined
    };
  });

  return {
    status,
    classification,
    readOnly: true,
    enforcement: "detection-only",
    complete: status === "pass",
    summary: {
      required: requiredGates.length,
      supplied: records.length,
      present: requiredGates.length - missing.length,
      missing: missing.length,
      stale: findings.filter((finding) => finding.code === "WG004_STALE_EVIDENCE").length,
      unknown: unknownGates.length,
      errors,
      warnings
    },
    requiredGates,
    gateResults,
    findings,
    publicSummaryMarkdown: `Work-gate completeness: ${status} (${requiredGates.length - missing.length}/${requiredGates.length} required gates present). This is a read-only detection report, not an enforcement sandbox.`
  };
}

function isWorkGateName(value: string): value is WorkGateName {
  return (workGateSequence as readonly string[]).includes(value);
}

function parseTimestamp(value: string | undefined): number | undefined {
  if (!value) return undefined;
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function isOrdered(gates: WorkGateName[], requiredGates: WorkGateName[]): boolean {
  let lastIndex = -1;
  for (const gate of gates) {
    const index = requiredGates.indexOf(gate);
    if (index === -1) continue;
    if (index < lastIndex) return false;
    lastIndex = index;
  }
  return true;
}

function classify(presentCount: number, requiredCount: number, findings: WorkGateFinding[], status: string): WorkGateClassification {
  if (findings.some((finding) => finding.code === "WG005_UNKNOWN_GATE")) return "unknown_gate";
  if (findings.some((finding) => finding.code === "WG006_OUT_OF_ORDER")) return "out_of_order";
  if (findings.some((finding) => finding.code === "WG004_STALE_EVIDENCE")) return "stale";
  if (presentCount === 0 && requiredCount > 0) return "missing";
  if (presentCount < requiredCount) return "partial";
  return status === "pass" ? "complete" : "incomplete";
}
