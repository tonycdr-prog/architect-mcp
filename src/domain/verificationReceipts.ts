export const verificationReceiptSources = [
  "local_terminal",
  "ci",
  "tui",
  "adapter",
  "manual"
] as const;

export type VerificationReceiptSource = typeof verificationReceiptSources[number];
export type VerificationReceiptStatus = "passed" | "failed" | "skipped" | "not_run";

export type VerificationReceipt = {
  command: string;
  status: VerificationReceiptStatus;
  source: VerificationReceiptSource;
  summary: string;
  recordedAt?: string;
  runId?: string;
};

export type VerificationRecord = {
  check: string;
  status: VerificationReceiptStatus;
  note?: string;
};

export type VerificationReceiptReviewInput = {
  requiredChecks?: string[];
  verification?: VerificationRecord[];
  receipts?: VerificationReceipt[];
  now?: string;
  maxAgeSeconds?: number;
};

type ReceiptFinding = {
  code: string;
  severity: "error" | "warning";
  message: string;
  recommendation: string;
};

const DEFAULT_MAX_AGE_SECONDS = 7 * 24 * 60 * 60;

export function reviewVerificationReceipts(input: VerificationReceiptReviewInput = {}) {
  const requiredChecks = uniqueTrimmed(input.requiredChecks ?? []);
  const verificationRecords = input.verification ?? [];
  const receipts = input.receipts ?? [];
  const receiptsWereSupplied = input.receipts !== undefined;
  const nowMs = parseTimestamp(input.now) ?? Date.now();
  const maxAgeSeconds = input.maxAgeSeconds ?? DEFAULT_MAX_AGE_SECONDS;
  const findings: ReceiptFinding[] = [];
  const reviewedReceipts = receipts.map((receipt) => {
    const safeCommand = publicSafeText(receipt.command);
    const safeSummary = publicSafeText(receipt.summary);
    return {
      command: safeCommand.value,
      status: receipt.status,
      source: receipt.source,
      recordedAt: receipt.recordedAt,
      runId: receipt.runId,
      publicSafeSummary: safeSummary.value,
      redacted: safeCommand.redacted || safeSummary.redacted
    };
  });

  const receiptsByCommand = new Map<string, VerificationReceipt>();
  for (const receipt of receipts) {
    const key = normalizeKey(receipt.command);
    if (key && !receiptsByCommand.has(key)) receiptsByCommand.set(key, receipt);
  }

  const requiredReceiptMatches = requiredChecks.map((check) => {
    const receipt = receiptsByCommand.get(normalizeKey(check));
    return { check, receipt };
  });

  if (receiptsWereSupplied) {
    for (const { check, receipt } of requiredReceiptMatches) {
      if (!receipt) {
        findings.push({
          code: "VERIFY_RECEIPT001_MISSING",
          severity: "warning",
          message: `No verification receipt was supplied for required check: ${check}.`,
          recommendation: "Attach a public-safe command receipt, CI link summary, or run id before treating wording as execution evidence."
        });
      }
    }
  }

  for (const receipt of receipts) {
    const safeCommand = publicSafeText(receipt.command).value;
    if (receipt.status === "failed") {
      findings.push({
        code: "VERIFY_RECEIPT002_FAILED",
        severity: "error",
        message: `Verification receipt reports a failed command: ${safeCommand}.`,
        recommendation: "Do not present failed verification as passing evidence."
      });
    }
    if (receipt.status === "skipped" || receipt.status === "not_run") {
      findings.push({
        code: "VERIFY_RECEIPT003_INCOMPLETE",
        severity: "error",
        message: `Verification receipt reports a skipped or not-run command: ${safeCommand}.`,
        recommendation: "State the command as skipped/not run, or run it before claiming it passed."
      });
    }
    if (receipt.recordedAt) {
      const recordedAtMs = parseTimestamp(receipt.recordedAt);
      if (recordedAtMs === undefined || recordedAtMs > nowMs || nowMs - recordedAtMs > maxAgeSeconds * 1000) {
        findings.push({
          code: "VERIFY_RECEIPT005_STALE",
          severity: "error",
          message: `Verification receipt is stale or has an invalid timestamp: ${safeCommand}.`,
          recommendation: "Attach fresh evidence from the current run, or use a current run id when timestamp redaction is required."
        });
      }
    } else if (!receipt.runId) {
      findings.push({
        code: "VERIFY_RECEIPT004_FRESHNESS_UNKNOWN",
        severity: "warning",
        message: `Verification receipt has no timestamp or run id: ${safeCommand}.`,
        recommendation: "Include a public-safe timestamp or run id so reviewers can tell whether evidence is fresh."
      });
    }
  }

  const redactedCount = reviewedReceipts.filter((receipt) => receipt.redacted).length;
  if (redactedCount > 0) {
    findings.push({
      code: "VERIFY_RECEIPT006_PUBLIC_SUMMARY_REDACTED",
      severity: "warning",
      message: "One or more verification receipts contained token-shaped values or local paths that were redacted.",
      recommendation: "Keep public receipt summaries short and avoid raw paths, tokens, stdout, or stderr."
    });
  }

  const errors = findings.filter((finding) => finding.severity === "error").length;
  const status = errors > 0 ? "fail" : findings.length > 0 ? "warn" : "pass";
  const matchedRequired = requiredReceiptMatches.filter((match) => match.receipt).length;
  const verificationSummary = summarizeVerificationRecords(verificationRecords);

  return {
    status,
    valid: errors === 0,
    complete: requiredChecks.length > 0 && matchedRequired === requiredChecks.length && status === "pass",
    summary: {
      required: requiredChecks.length,
      receipts: receipts.length,
      matchedRequired,
      missingRequired: requiredChecks.length - matchedRequired,
      verificationRecords: verificationSummary,
      redacted: redactedCount,
      errors,
      warnings: findings.length - errors
    },
    claimedChecks: requiredChecks.map((check) => ({ check, receiptSupplied: Boolean(receiptsByCommand.get(normalizeKey(check))) })),
    verificationRecords,
    receipts: reviewedReceipts,
    findings
  };
}

function uniqueTrimmed(values: string[]): string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}

function normalizeKey(value: string): string {
  return value.trim().toLowerCase();
}

function parseTimestamp(value: string | undefined): number | undefined {
  if (!value) return undefined;
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function summarizeVerificationRecords(records: VerificationRecord[]) {
  return {
    total: records.length,
    passed: records.filter((record) => record.status === "passed").length,
    failed: records.filter((record) => record.status === "failed").length,
    incomplete: records.filter((record) => record.status === "skipped" || record.status === "not_run").length
  };
}

function publicSafeText(value: string): { value: string; redacted: boolean } {
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
