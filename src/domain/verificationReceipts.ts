import { publicSafeOptionalText, publicSafeSummary, publicSafeText } from "./publicSafetyText.js";
export { publicSafeText } from "./publicSafetyText.js";

export const verificationReceiptSources = ["local_terminal", "ci", "tui", "adapter", "manual"] as const;

export type VerificationReceiptSource = typeof verificationReceiptSources[number];
export type VerificationReceiptStatus = "passed" | "failed" | "skipped" | "not_run";
export type VerificationEvidenceTier = "supplied" | "independent";
export type VerificationFreshnessStatus = "fresh" | "independent_run_id" | "missing" | "stale" | "future" | "invalid";

export type VerificationReceipt = {
  command: string;
  status: VerificationReceiptStatus;
  source: VerificationReceiptSource;
  summary: string;
  recordedAt?: string;
  runId?: string;
};

export type VerificationRecord = { check: string; status: VerificationReceiptStatus; note?: string };

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

type ReviewedReceipt = {
  sourceReceipt: VerificationReceipt;
  command: string;
  status: VerificationReceiptStatus;
  source: VerificationReceiptSource;
  evidenceTier: VerificationEvidenceTier;
  independentlyResolvable: boolean;
  freshness: {
    status: VerificationFreshnessStatus;
    satisfied: boolean;
    recordedAtPresent: boolean;
    runIdPresent: boolean;
  };
  recordedAtPresent: boolean;
  runIdPresent: boolean;
  publicSafeSummary: string;
  redacted: boolean;
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
  const reviewedReceipts: ReviewedReceipt[] = receipts.map((receipt) => {
    const safeCommand = publicSafeText(receipt.command);
    const safeSummary = publicSafeSummary(receipt.summary);
    const freshness = reviewFreshness(receipt, nowMs, maxAgeSeconds);
    const independentlyResolvable = hasIndependentHandle(receipt, freshness);
    const evidenceTier = receiptEvidenceTier(independentlyResolvable);
    return {
      sourceReceipt: receipt,
      command: safeCommand.value,
      status: receipt.status,
      source: receipt.source,
      evidenceTier,
      independentlyResolvable,
      freshness,
      recordedAtPresent: freshness.recordedAtPresent,
      runIdPresent: freshness.runIdPresent,
      publicSafeSummary: safeSummary.value,
      redacted: safeCommand.redacted || safeSummary.redacted
    };
  });
  const reviewedVerificationRecords = verificationRecords.map((record) => {
    const safeCheck = publicSafeText(record.check);
    const safeNote = publicSafeOptionalText(record.note);
    return {
      check: safeCheck.value,
      status: record.status,
      note: safeNote.value,
      redacted: safeCheck.redacted || safeNote.redacted
    };
  });

  const receiptsByCommand = new Map<string, ReviewedReceipt[]>();
  for (const receipt of reviewedReceipts) {
    const key = normalizeKey(receipt.sourceReceipt.command);
    if (!key) continue;
    const matches = receiptsByCommand.get(key) ?? [];
    matches.push(receipt);
    receiptsByCommand.set(key, matches);
  }
  const freshPassedCommands = new Set(
    reviewedReceipts
      .filter((receipt) => isFreshPassedReceipt(receipt))
      .map((receipt) => normalizeKey(receipt.sourceReceipt.command))
      .filter(Boolean)
  );

  const requiredReceiptMatches = requiredChecks.map((check) => {
    const receipts = receiptsByCommand.get(normalizeKey(check)) ?? [];
    return { check, receipts };
  });

  if (receiptsWereSupplied) {
    for (const { check, receipts } of requiredReceiptMatches) {
      if (receipts.length === 0) {
        const safeCheck = publicSafeText(check);
        findings.push({
          code: "VERIFY_RECEIPT001_MISSING",
          severity: "warning",
          message: `No verification receipt was supplied for required check: ${safeCheck.value}.`,
          recommendation: "Attach a public-safe command receipt, CI link summary, or run id before treating wording as execution evidence."
        });
      }
    }
  }

  for (const receipt of reviewedReceipts) {
    const safeCommand = receipt.command;
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
    const hasFreshPassedSibling = freshPassedCommands.has(normalizeKey(receipt.sourceReceipt.command));
    if (receipt.freshness.status === "missing" && !hasFreshPassedSibling) {
      findings.push({
        code: "VERIFY_RECEIPT004_FRESHNESS_UNKNOWN",
        severity: "warning",
        message: receipt.runIdPresent
          ? `Verification receipt run id is not independently resolvable for its source: ${safeCommand}.`
          : `Verification receipt has no timestamp or independently resolvable run id: ${safeCommand}.`,
        recommendation: "Include a public-safe timestamp for local, TUI, adapter, or manual receipts; run ids alone only prove freshness for independently resolvable sources such as CI."
      });
    }
    if ((receipt.freshness.status === "invalid" || receipt.freshness.status === "future" || receipt.freshness.status === "stale") && !hasFreshPassedSibling) {
      findings.push({
        code: "VERIFY_RECEIPT005_STALE",
        severity: "error",
        message: `Verification receipt is stale or has an invalid timestamp: ${safeCommand}.`,
        recommendation: "Attach fresh evidence from the current run, or attach an independently resolvable CI receipt."
      });
    }
  }

  const redactedCount = reviewedReceipts.filter((receipt) => receipt.redacted).length
    + reviewedVerificationRecords.filter((record) => record.redacted).length
    + requiredChecks.filter((check) => publicSafeText(check).redacted).length;
  if (redactedCount > 0) {
    findings.push({
      code: "VERIFY_RECEIPT006_PUBLIC_SUMMARY_REDACTED",
      severity: "warning",
      message: "One or more verification receipts, checks, or records contained token-shaped values or local paths that were redacted.",
      recommendation: "Keep public receipt summaries and check labels short, and avoid raw paths, tokens, stdout, stderr, or private notes."
    });
  }

  const errors = findings.filter((finding) => finding.severity === "error").length;
  const status = errors > 0 ? "fail" : findings.length > 0 ? "warn" : "pass";
  const matchedRequired = requiredReceiptMatches.filter((match) => match.receipts.length > 0).length;
  const freshMatchedRequired = requiredReceiptMatches.filter((match) => match.receipts.some(isFreshPassedReceipt)).length;
  const verificationSummary = summarizeVerificationRecords(verificationRecords);
  const suppliedReceipts = reviewedReceipts.filter((receipt) => receipt.evidenceTier === "supplied").length;
  const independentReceipts = reviewedReceipts.filter((receipt) => receipt.evidenceTier === "independent").length;

  return {
    status,
    valid: errors === 0,
    complete: requiredChecks.length > 0 && freshMatchedRequired === requiredChecks.length && status === "pass",
    summary: {
      required: requiredChecks.length,
      receipts: receipts.length,
      matchedRequired,
      freshMatchedRequired,
      missingRequired: requiredChecks.length - matchedRequired,
      missingFreshRequired: requiredChecks.length - freshMatchedRequired,
      verificationRecords: verificationSummary,
      evidenceTiers: {
        claimed: requiredChecks.length,
        supplied: verificationRecords.length + suppliedReceipts,
        independent: independentReceipts
      },
      redacted: redactedCount,
      errors,
      warnings: findings.length - errors
    },
    claimedChecks: requiredChecks.map((check) => {
      const safeCheck = publicSafeText(check);
      return {
        check: safeCheck.value,
        receiptSupplied: (receiptsByCommand.get(normalizeKey(check)) ?? []).length > 0,
        freshReceiptSupplied: (receiptsByCommand.get(normalizeKey(check)) ?? []).some(isFreshPassedReceipt),
        independentReceiptSupplied: (receiptsByCommand.get(normalizeKey(check)) ?? []).some((receipt) => receipt.independentlyResolvable),
        redacted: safeCheck.redacted
      };
    }),
    verificationRecords: reviewedVerificationRecords,
    receipts: reviewedReceipts.map((receipt) => ({
      command: receipt.command,
      status: receipt.status,
      source: receipt.source,
      evidenceTier: receipt.evidenceTier,
      independentlyResolvable: receipt.independentlyResolvable,
      freshness: receipt.freshness,
      recordedAtPresent: receipt.recordedAtPresent,
      runIdPresent: receipt.runIdPresent,
      publicSafeSummary: receipt.publicSafeSummary,
      redacted: receipt.redacted
    })),
    findings
  };
}

function receiptEvidenceTier(independentlyResolvable: boolean): VerificationEvidenceTier {
  return independentlyResolvable ? "independent" : "supplied";
}

function reviewFreshness(receipt: VerificationReceipt, nowMs: number, maxAgeSeconds: number): ReviewedReceipt["freshness"] {
  const recordedAtPresent = Boolean(receipt.recordedAt);
  const runIdPresent = Boolean(receipt.runId);
  if (receipt.recordedAt) {
    const recordedAtMs = parseTimestamp(receipt.recordedAt);
    if (recordedAtMs === undefined) {
      return { status: "invalid", satisfied: false, recordedAtPresent, runIdPresent };
    }
    if (recordedAtMs > nowMs) {
      return { status: "future", satisfied: false, recordedAtPresent, runIdPresent };
    }
    if (nowMs - recordedAtMs > maxAgeSeconds * 1000) {
      return { status: "stale", satisfied: false, recordedAtPresent, runIdPresent };
    }
    return { status: "fresh", satisfied: true, recordedAtPresent, runIdPresent };
  }
  if (runIdPresent && hasCiRunId(receipt)) {
    return { status: "independent_run_id", satisfied: true, recordedAtPresent, runIdPresent };
  }
  return { status: "missing", satisfied: false, recordedAtPresent, runIdPresent };
}

function hasIndependentHandle(receipt: VerificationReceipt, freshness: ReviewedReceipt["freshness"]): boolean {
  return hasCiRunId(receipt) || (receipt.source === "ci" && freshness.status === "fresh");
}

function hasCiRunId(receipt: VerificationReceipt): boolean {
  return receipt.source === "ci" && Boolean(receipt.runId?.trim());
}

function isFreshPassedReceipt(receipt: ReviewedReceipt | undefined): boolean {
  return Boolean(receipt && receipt.status === "passed" && receipt.freshness.satisfied);
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
