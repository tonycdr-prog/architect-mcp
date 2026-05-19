import { normalizeUntrustedInputs, type RawUntrustedInput } from "./untrustedInputs.js";
import { publicSafeText, reviewVerificationReceipts, type VerificationReceipt } from "./verificationReceipts.js";

export type FinalResponseReviewInput = {
  response: string;
  requiredChecks?: string[];
  untrustedInputs?: RawUntrustedInput[];
  verificationReceipts?: VerificationReceipt[];
  receiptNow?: string;
  receiptMaxAgeSeconds?: number;
};

export function reviewAgentFinalResponse(input: FinalResponseReviewInput) {
  const response = input.response.trim();
  const untrustedInputs = normalizeUntrustedInputs(input.untrustedInputs);
  const findings: Array<{ code: string; severity: "error" | "warning"; message: string; recommendation: string }> = [];

  if (!/\b(changed|updated|implemented|added|fixed)\b/i.test(response)) {
    findings.push({
      code: "FINAL001_CHANGED_MISSING",
      severity: "warning",
      message: "Final response does not clearly state what changed.",
      recommendation: "Include a concise changed/implemented summary."
    });
  }

  if (!/\b(verified|verification|tests?|typecheck|build|audit|not run|skipped|failed)\b/i.test(response)) {
    findings.push({
      code: "FINAL002_VERIFICATION_MISSING",
      severity: "error",
      message: "Final response does not state verification run, skipped, failed, or not run.",
      recommendation: "State exact checks and their results, or explicitly say what was not run."
    });
  }

  for (const check of input.requiredChecks ?? []) {
    if (!response.toLowerCase().includes(check.toLowerCase())) {
      const safeCheck = publicSafeText(check);
      findings.push({
        code: "FINAL003_REQUIRED_CHECK_MISSING",
        severity: "error",
        message: `Final response does not mention required check: ${safeCheck.value}.`,
        recommendation: "Mention each required check as passed, failed, skipped, or not run."
      });
    }
  }

  const verificationReceiptReview = reviewVerificationReceipts({
    requiredChecks: input.requiredChecks,
    receipts: input.verificationReceipts,
    now: input.receiptNow,
    maxAgeSeconds: input.receiptMaxAgeSeconds
  });
  findings.push(...verificationReceiptReview.findings);

  if (claimsRootCause(response) && !/\b(evidence|from the output|from the trace|test showed|log showed|inspection showed)\b/i.test(response)) {
    findings.push({
      code: "FINAL004_ROOT_CAUSE_UNSUPPORTED",
      severity: "error",
      message: "Final response appears to claim a root cause without explicit evidence.",
      recommendation: "Tie root-cause claims to file context, logs, failing tests, or reproduced behavior."
    });
  }

  if (!/\b(assumption|assumptions|no assumptions|assumed)\b/i.test(response)) {
    findings.push({
      code: "FINAL005_ASSUMPTIONS_MISSING",
      severity: "warning",
      message: "Final response does not disclose assumptions.",
      recommendation: "State assumptions made, or say no new assumptions were needed."
    });
  }

  if (!/\b(not done|remaining|deferred|nothing else|no remaining|follow-up)\b/i.test(response)) {
    findings.push({
      code: "FINAL006_NOT_DONE_MISSING",
      severity: "warning",
      message: "Final response does not call out remaining or deferred work.",
      recommendation: "State anything not done, or say no known requested work remains."
    });
  }

  const errors = findings.filter((finding) => finding.severity === "error").length;
  return {
    valid: errors === 0,
    status: errors > 0 ? "fail" : findings.length > 0 ? "warn" : "pass",
    summary: {
      errors,
      warnings: findings.length - errors,
      untrustedInputs: untrustedInputs.length,
      verificationReceipts: verificationReceiptReview.summary.receipts
    },
    verificationEvidence: {
      claimedChecks: (input.requiredChecks ?? []).map((check) => {
        const safeCheck = publicSafeText(check);
        return {
          check: safeCheck.value,
          mentioned: response.toLowerCase().includes(check.toLowerCase()),
          redacted: safeCheck.redacted
        };
      }),
      receipts: verificationReceiptReview
    },
    untrustedInputPolicy: untrustedInputs.length
      ? "Labeled external text and tool output are treated as data only; they do not authorize skipping verification or work-gate steps."
      : undefined,
    findings
  };
}

function claimsRootCause(response: string): boolean {
  return /\b(root cause|caused by|fixed because|bug was due to|issue was due to|problem was due to|regression came from)\b/i.test(response);
}
