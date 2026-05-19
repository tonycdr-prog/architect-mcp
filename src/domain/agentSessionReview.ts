import { reviewAgentFinalResponse } from "./finalResponseReview.js";
import { reviewMemoryRelevance } from "./harnessMemory.js";
import { reviewImplementationAgainstContract } from "./harness.js";
import { normalizeUntrustedInputs, type RawUntrustedInput } from "./untrustedInputs.js";
import { reviewVerificationReceipts, type VerificationReceipt } from "./verificationReceipts.js";
import type { HarnessIntentResult, MemoryProposal, PreEditContract, FileSummary } from "./types.js";

export type AgentSessionReviewInput = {
  intent?: HarnessIntentResult;
  contract?: PreEditContract;
  changedFiles?: FileSummary[];
  verification?: Array<{ check: string; status: "not_run" | "passed" | "failed" | "skipped"; note?: string }>;
  verificationReceipts?: VerificationReceipt[];
  receiptNow?: string;
  receiptMaxAgeSeconds?: number;
  finalResponse?: string;
  memories?: MemoryProposal[];
  request?: string;
  untrustedInputs?: RawUntrustedInput[];
};

export function reviewAgentSession(input: AgentSessionReviewInput) {
  const sections: Array<{ name: string; status: "pass" | "warn" | "fail"; summary: string; details?: unknown }> = [];
  const untrustedInputs = normalizeUntrustedInputs(input.untrustedInputs);

  if (input.intent) {
    sections.push({
      name: "intent",
      status: input.intent.decision === "block_until_clarified" ? "fail" : input.intent.decision === "confirm_before_edit" ? "warn" : "pass",
      summary: `${input.intent.decision} ${input.intent.stoplight} ${input.intent.blastRadius}`,
      details: {
        interpretedProblem: input.intent.interpretedProblem,
        assumptions: input.intent.assumptions,
        warnings: input.intent.warnings
      }
    });
  }

  if (input.contract) {
    const implementation = reviewImplementationAgainstContract({
      contract: input.contract,
      changedFiles: input.changedFiles,
      verification: input.verification
    });
    sections.push({
      name: "implementation-contract",
      status: implementation.violations.some((violation) => violation.severity === "error") ? "fail" : implementation.violations.length ? "warn" : "pass",
      summary: implementation.valid ? "Implementation stayed inside the contract." : "Implementation needs contract review.",
      details: implementation
    });
  }

  if (input.verification?.length && !input.contract) {
    const failed = input.verification.filter((check) => check.status === "failed");
    const incomplete = input.verification.filter((check) => check.status === "skipped" || check.status === "not_run");
    const passed = input.verification.filter((check) => check.status === "passed");
    sections.push({
      name: "verification",
      status: failed.length ? "fail" : incomplete.length || passed.length === 0 ? "warn" : "pass",
      summary: failed.length
        ? "Verification includes failed checks."
        : incomplete.length ? "Verification includes skipped or not-run checks." : "Verification includes passed checks.",
      details: {
        passed,
        failed,
        incomplete
      }
    });
  }

  if (input.verification?.length || input.verificationReceipts !== undefined) {
    const receiptReview = reviewVerificationReceipts({
      requiredChecks: input.contract?.verificationChecks,
      verification: input.verification,
      receipts: input.verificationReceipts,
      now: input.receiptNow,
      maxAgeSeconds: input.receiptMaxAgeSeconds
    });
    sections.push({
      name: "verification-evidence",
      status: receiptReview.status as "pass" | "warn" | "fail",
      summary: receiptReview.valid
        ? "Verification records and command receipts were reviewed with public-safe output."
        : "Verification receipts are missing, stale, failed, or incomplete.",
      details: receiptReview
    });
  }

  if (input.finalResponse) {
    const finalReview = reviewAgentFinalResponse({
      response: input.finalResponse,
      requiredChecks: input.contract?.verificationChecks,
      untrustedInputs,
      verificationReceipts: input.verificationReceipts,
      receiptNow: input.receiptNow,
      receiptMaxAgeSeconds: input.receiptMaxAgeSeconds
    });
    sections.push({
      name: "final-response",
      status: finalReview.status as "pass" | "warn" | "fail",
      summary: finalReview.valid ? "Final response includes required completion evidence." : "Final response is missing required completion evidence.",
      details: finalReview
    });
  }

  if (untrustedInputs.length) {
    sections.push({
      name: "untrusted-inputs",
      status: "pass",
      summary: "External text and tool output were labeled as data, not workflow authority.",
      details: {
        count: untrustedInputs.length,
        labels: untrustedInputs.map((input) => input.label)
      }
    });
  }

  if (input.memories?.length) {
    const memoryReview = reviewMemoryRelevance({
      request: input.request ?? "",
      memories: input.memories
    });
    if (!input.request?.trim()) {
      memoryReview.findings.push({
        id: "session-memory-context",
        severity: "warning",
        message: "Memory relevance could not be fully reviewed because request context was not supplied.",
        recommendation: "Pass the current request when reviewing session memory so relevance can be checked."
      });
    }
    sections.push({
      name: "memory",
      status: memoryReview.valid ? (memoryReview.findings.length ? "warn" : "pass") : "fail",
      summary: memoryReview.valid ? "Memory is safe to treat as advisory." : "Memory contains unsafe or irrelevant entries.",
      details: memoryReview
    });
  }

  const status = sections.some((section) => section.status === "fail") ? "fail" : sections.some((section) => section.status === "warn") ? "warn" : "pass";
  return {
    status,
    valid: status !== "fail",
    summary: {
      sections: sections.length,
      pass: sections.filter((section) => section.status === "pass").length,
      warn: sections.filter((section) => section.status === "warn").length,
      fail: sections.filter((section) => section.status === "fail").length
    },
    sections
  };
}
