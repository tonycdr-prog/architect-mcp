import { createHash } from "node:crypto";
import { createFinding } from "./findingMetadata.js";
import { loadTriggeredStackGuidance } from "./harnessGuidance.js";
import { classifyBlastRadius, classifyChangeType, decideHarnessAction, findEscalationTerms, isVagueRequest, stoplightForDecision, termsToPlainOptions } from "./harnessSignals.js";
import type { AssumptionLedgerEntry, BlastRadius, FileSummary, HarnessEvidence, HarnessIntentInput, HarnessIntentResult, HarnessMode, PreEditContract, ProposedFilePlan, ReviewViolation } from "./types.js";

export function interpretImplementationIntent(input: HarnessIntentInput): HarnessIntentResult {
  const mode = input.mode ?? "guided-yolo";
  const request = input.request.trim();
  if (!request) {
    return emptyIntent(mode);
  }
  const contextText = `${request} ${Object.values(input.stack ?? {}).join(" ")}`;
  const changeType = classifyChangeType(contextText);
  const vague = isVagueRequest(request);
  const evidence = collectEvidence(input);
  const blastRadius = classifyBlastRadius(contextText, input.files?.length ?? input.proposedPlan?.files.length ?? 0);
  const escalationTerms = findEscalationTerms(request);
  const guidanceResult = shouldLoadGuidance(request, escalationTerms) ? loadTriggeredStackGuidance(input) : { guidance: [], warnings: [] };
  const decision = decideHarnessAction({ mode, vague, blastRadius, evidenceCount: evidence.length + guidanceResult.guidance.length, assumptionCount: input.assumptionCount });
  const assumptions = buildAssumptions(request, blastRadius, vague, guidanceResult.guidance.length > 0);
  const verification = selectVerification(input, changeType, blastRadius);
  const interpretedProblem = interpretProblem(request, changeType);
  const intendedFix = intendedFixFor(changeType, blastRadius);
  const nonGoals = nonGoalsFor(changeType, blastRadius);

  return {
    mode,
    decision,
    stoplight: stoplightForDecision(decision),
    blastRadius,
    changeType,
    confidence: confidenceFor(vague, evidence.length, blastRadius),
    interpretedProblem,
    intendedFix,
    plan: planFor(changeType, blastRadius),
    assumptions,
    nonGoals,
    verification,
    evidence,
    triggeredGuidance: guidanceResult.guidance,
    escalationTerms,
    plainLanguageOptions: termsToPlainOptions(request, input.stack),
    confirmationPrompt: confirmationPromptFor(decision, interpretedProblem, intendedFix, nonGoals),
    warnings: [
      ...guidanceResult.warnings,
      ...harnessWarnings(request, vague, evidence, blastRadius, input.proposedPlan)
    ],
    handoffSummary: `Intent: ${interpretedProblem} Decision: ${decision}. Verify with: ${verification.join(", ")}.`
  };
}

export function classifyAmbiguityRisk(input: HarnessIntentInput) {
  const interpreted = interpretImplementationIntent(input);
  return {
    mode: interpreted.mode,
    decision: interpreted.decision,
    stoplight: interpreted.stoplight,
    blastRadius: interpreted.blastRadius,
    changeType: interpreted.changeType,
    confidence: interpreted.confidence,
    escalationTerms: interpreted.escalationTerms,
    warnings: interpreted.warnings,
    confirmationPrompt: interpreted.confirmationPrompt
  };
}

export function createPreEditContract(input: { intent: HarnessIntentResult; likelyFiles?: string[]; verificationChecks?: string[] }): PreEditContract {
  const likelyFiles = input.likelyFiles?.length ? input.likelyFiles : inferLikelyFiles(input.intent);
  return {
    id: createHash("sha256").update(`${input.intent.interpretedProblem}:${Date.now()}`).digest("hex").slice(0, 12),
    createdAt: new Date().toISOString(),
    mode: input.intent.mode,
    decision: input.intent.decision,
    interpretedProblem: input.intent.interpretedProblem,
    intendedBehavior: input.intent.intendedFix,
    changeType: input.intent.changeType,
    blastRadius: input.intent.blastRadius,
    likelyFiles,
    nonGoals: input.intent.nonGoals,
    assumptions: input.intent.assumptions,
    evidence: input.intent.evidence,
    triggeredGuidance: input.intent.triggeredGuidance,
    verificationChecks: input.verificationChecks?.length ? input.verificationChecks : input.intent.verification,
    rollbackPlan: rollbackPlanFor(input.intent.blastRadius),
    outputContract: ["Summarize what changed.", "State verification run and results.", "List assumptions made.", "Call out anything not done."]
  };
}

export function recordAssumption(input: AssumptionLedgerEntry): { assumption: AssumptionLedgerEntry } {
  return { assumption: input };
}

export function reviewImplementationAgainstContract(input: {
  contract: PreEditContract;
  changedFiles?: FileSummary[];
  proposedPlan?: ProposedFilePlan;
  verification?: Array<{ check: string; status: "not_run" | "passed" | "failed" | "skipped"; note?: string }>;
}): { valid: boolean; violations: ReviewViolation[] } {
  const violations: ReviewViolation[] = [];
  const changedPaths = (input.changedFiles?.map((file) => file.path) ?? input.proposedPlan?.files.map((file) => file.path) ?? []);
  const unrelated = changedPaths.filter((path) => input.contract.likelyFiles.length > 0 && !matchesLikelyFile(path, input.contract.likelyFiles));
  if (unrelated.length > 0) {
    violations.push(harnessFinding(`Changes touched files outside the pre-edit contract: ${unrelated.join(", ")}.`, "Confirm the widened scope or update the contract before proceeding."));
  }
  if (input.contract.blastRadius !== "low" && changedPaths.length > Math.max(6, input.contract.likelyFiles.length + 3)) {
    violations.push(harnessFinding("The implementation appears broader than the promised plan.", "Narrow the change or ask for confirmation before accepting the expanded blast radius."));
  }
  const verification = input.verification ?? [];
  if (verification.length === 0 || verification.some((check) => check.status === "not_run" || check.status === "skipped" || check.status === "failed")) {
    violations.push(harnessFinding("Verification was not completed honestly for the pre-edit contract.", "Report skipped/failed checks and do not present the work as complete until required proof passes.", "warning"));
  }
  if (input.proposedPlan?.files.some((file) => /app\.(tsx|jsx)$|index\.(ts|js)$|server\.(ts|js)$/i.test(file.path) && (file.responsibilities?.length ?? 0) > 3)) {
    violations.push(harnessFinding("Proposed implementation concentrates too many responsibilities in an entry file.", "Split UI, data access, workflow orchestration, and validation into owned modules."));
  }
  return {
    valid: violations.filter((violation) => violation.severity === "error").length === 0,
    violations
  };
}

function collectEvidence(input: HarnessIntentInput): HarnessEvidence[] {
  return [
    ...(input.currentError ? [{ kind: "error_output" as const, summary: input.currentError.slice(0, 240), source: "currentError" }] : []),
    ...(input.files?.slice(0, 5).map((file) => ({ kind: "file_context" as const, summary: `${file.path}${file.lines ? ` (${file.lines} lines)` : ""}`, source: file.path })) ?? []),
    { kind: "user_statement" as const, summary: input.request.slice(0, 240), source: "request" }
  ];
}

function shouldLoadGuidance(request: string, terms: string[]): boolean {
  return terms.length > 0 || /auth|database|schema|frontend|backend|api|ui|secure|performance|deploy|best practices/i.test(request);
}

function buildAssumptions(request: string, risk: BlastRadius, vague: boolean, hasGuidance: boolean): AssumptionLedgerEntry[] {
  if (!vague) return [];
  return [{
    statement: `The request "${request}" should be treated as a narrow implementation-intent problem, not permission for unrelated cleanup.`,
    reason: hasGuidance ? "Triggered stack guidance is available but the user did not name exact success criteria." : "The request uses vague language without concrete success criteria.",
    confidence: risk === "low" ? "medium" : "low",
    risk,
    invalidatedBy: "The user confirms a broader refactor, schema/API change, or different meaning of best practices.",
    affectedArea: "implementation scope"
  }];
}

function selectVerification(input: HarnessIntentInput, changeType: string, risk: BlastRadius): string[] {
  if (input.verification?.length) return input.verification;
  const checks = ["Run the narrowest relevant existing test or reproduction."];
  if (changeType === "styling") checks.push("Run a UI smoke check for the changed screen.");
  if (changeType === "security" || risk === "high" || risk === "critical") checks.push("Run architecture review and security-sensitive regression checks.");
  checks.push("State any checks that were not run.");
  return checks;
}

function interpretProblem(request: string, changeType: string): string {
  if (/best practices|properly|clean|make better/i.test(request)) return `The user likely wants a ${changeType.replace("_", " ")} handled with maintainable boundaries instead of a broad guess.`;
  return `The user wants to handle: ${request}`;
}

function intendedFixFor(changeType: string, risk: BlastRadius): string {
  if (risk === "critical") return "Do not edit yet; clarify the destructive or hard-to-reverse operation first.";
  if (changeType === "security") return "Identify the exact security boundary, then make the smallest verified change around authorization, secrets, validation, or session handling.";
  if (changeType === "data_schema") return "Keep schema/query changes behind server-owned data boundaries and confirm before migrations.";
  if (changeType === "styling") return "Improve the targeted UI surface without changing unrelated behavior.";
  return "Make the smallest change that solves the interpreted problem, then verify it.";
}

function planFor(changeType: string, risk: BlastRadius): string[] {
  return [
    "Restate the user intent in plain English.",
    risk === "low" ? "Inspect the smallest relevant files or error output." : "Confirm the blast radius before editing.",
    changeType === "data_schema" ? "Keep data access behind server-owned modules." : "Avoid unrelated cleanup.",
    "Run or report the relevant verification checks."
  ];
}

function nonGoalsFor(changeType: string, risk: BlastRadius): string[] {
  return [
    "Do not rewrite unrelated files.",
    "Do not weaken types or swallow errors to make checks pass.",
    ...(risk === "high" || risk === "critical" ? ["Do not change schemas, auth/session behavior, payments, public APIs, or dependencies without confirmation."] : []),
    ...(changeType === "styling" ? ["Do not change data flow or business logic."] : [])
  ];
}

function confidenceFor(vague: boolean, evidenceCount: number, risk: BlastRadius): HarnessIntentResult["confidence"] {
  if (vague || risk === "critical") return "low";
  return evidenceCount > 1 ? "high" : "medium";
}

function confirmationPromptFor(decision: string, problem: string, fix: string, nonGoals: string[]): string | undefined {
  if (decision === "proceed" || decision === "proceed_with_assumptions") return undefined;
  if (/^Do not edit yet/i.test(fix)) {
    return `I think you mean: ${problem}. This looks hard to reverse, so I will not edit yet. Is that right, or did you mean something else?`;
  }
  return `I think you mean: ${problem}. If so, I plan to ${fix.charAt(0).toLowerCase()}${fix.slice(1)} I will avoid: ${nonGoals[0]}. Is that right, or did you mean something else?`;
}

function harnessWarnings(request: string, vague: boolean, evidence: HarnessEvidence[], risk: BlastRadius, proposedPlan?: ProposedFilePlan): string[] {
  const warnings: string[] = [];
  if (vague) warnings.push("Request is vague; do not silently expand scope.");
  if (/root cause|because|the issue is/i.test(request) && evidence.length <= 1) warnings.push("Root-cause claim needs evidence before implementation.");
  if (risk === "critical") warnings.push("One-way or destructive operation requires clarification.");
  if (proposedPlan && proposedPlan.files.length > 8) warnings.push("Proposed plan has broad file scope; confirm before editing.");
  if (/upgrade|dependency|package/i.test(request)) warnings.push("Dependency upgrade requires reason, compatibility risk, and rollback plan.");
  return warnings;
}

function inferLikelyFiles(intent: HarnessIntentResult): string[] {
  if (intent.changeType === "styling") return ["src/**/*.tsx", "src/**/*.css"];
  if (intent.changeType === "data_schema") return ["src/db/**/*", "src/server/**/*"];
  if (intent.changeType === "security") return ["src/server/**/*", "src/auth/**/*"];
  return ["src/**/*"];
}

function rollbackPlanFor(risk: BlastRadius): string {
  return risk === "low" ? "Revert the isolated changed files if verification fails." : "Keep changes reviewable as a small patch and revert all contract-listed files if verification or confirmation fails.";
}

function matchesLikelyFile(path: string, patterns: string[]): boolean {
  return patterns.some((pattern) => path === pattern || globToRegExp(pattern).test(path));
}

function harnessFinding(message: string, recommendation: string, severity: ReviewViolation["severity"] = "error"): ReviewViolation {
  return createFinding({ code: "ARCH024_AGENT_HARNESS", severity, message, recommendation, confidence: "medium" });
}

function emptyIntent(mode: HarnessMode): HarnessIntentResult {
  const interpretedProblem = "The user did not provide an implementation request.";
  const intendedFix = "Do not edit yet; ask for the problem, target area, and success criteria.";
  return {
    mode,
    decision: "block_until_clarified",
    stoplight: "red",
    blastRadius: "critical",
    changeType: "unknown",
    confidence: "low",
    interpretedProblem,
    intendedFix,
    plan: ["Ask what problem should be solved.", "Ask what should be true after the fix.", "Do not edit files until intent exists."],
    assumptions: [],
    nonGoals: ["Do not infer a task from an empty request."],
    verification: ["No verification can be selected until the request is clarified."],
    evidence: [],
    triggeredGuidance: [],
    escalationTerms: [],
    plainLanguageOptions: [],
    confirmationPrompt: "What problem do you want the agent to fix, and what should be true when it is done?",
    warnings: ["Blank implementation request; blocked until clarified."],
    handoffSummary: `Intent: ${interpretedProblem} Decision: block_until_clarified.`
  };
}

function globToRegExp(pattern: string): RegExp {
  let regex = "^";
  for (let index = 0; index < pattern.length; index += 1) {
    const char = pattern[index];
    const next = pattern[index + 1];
    if (char === "*" && next === "*") {
      const after = pattern[index + 2];
      regex += after === "/" ? "(?:.*/)?" : ".*";
      index += after === "/" ? 2 : 1;
    } else if (char === "*") {
      regex += "[^/]*";
    } else if (char === "?") {
      regex += "[^/]";
    } else {
      regex += escapeRegExp(char);
    }
  }
  return new RegExp(`${regex}$`);
}

function escapeRegExp(value: string): string {
  return value.replace(/[|\\{}()[\]^$+?.]/g, "\\$&");
}
