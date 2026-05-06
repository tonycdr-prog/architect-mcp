import type { BlastRadius, ChangeType, HarnessDecision, HarnessMode, StackProfile, Stoplight } from "./types.js";

export const ESCALATION_TERMS = [
  "best practices",
  "properly",
  "production ready",
  "clean",
  "fix everything",
  "refactor",
  "optimize",
  "secure",
  "make better",
  "sort this"
];

export function classifyChangeType(text: string): ChangeType {
  const value = text.toLowerCase();
  if (/auth|secure|permission|role|session|secret|tenant|csrf|xss|injection/.test(value)) return "security";
  if (/database|schema|migration|sql|postgres|supabase|prisma|drizzle|mongo/.test(value)) return "data_schema";
  if (/dependency|package|upgrade|version|npm install|pnpm add|yarn add/.test(value)) return "dependency_config";
  if (/performance|optimi[sz]e|slow|latency|speed/.test(value)) return "performance";
  if (/\bui\b|style|professional|design|layout|spacing|responsive|accessibility|a11y/.test(value)) return "styling";
  if (/refactor|clean|split|organize|architecture|structure/.test(value)) return "refactor";
  if (/bug|fix|error|broken|failing|crash|issue|problem/.test(value)) return "bug_fix";
  if (/add|build|create|feature|implement/.test(value)) return "feature";
  return "unknown";
}

export function findEscalationTerms(text: string): string[] {
  const value = text.toLowerCase();
  return ESCALATION_TERMS.filter((term) => value.includes(term));
}

export function isVagueRequest(text: string): boolean {
  const value = text.trim().toLowerCase();
  return value.length < 28 ||
    /^(fix|improve|clean|sort|secure|optimi[sz]e|refactor|make).*(this|it|things|stuff|problem|issue)?$/.test(value) ||
    /(best practices|properly|make better|production ready|clean this up|fix this|sort this)/.test(value);
}

export function classifyBlastRadius(text: string, filesCount = 0): BlastRadius {
  const value = text.toLowerCase();
  if (/reset --hard|force push|rm -rf|drop table|delete .*database|delete all|delete .*folder|delete .*files?|remove all|migration apply|bulk delete/.test(value)) return "critical";
  if (/payment|billing|checkout|invoice|subscription|auth|permission|role|session|secret|migration|schema|public api|breaking api/.test(value)) return "high";
  if (/database|supabase|postgres|prisma|drizzle|mongo|security|secure|dependency|upgrade|refactor|architecture|backend|api|hono|trpc|server|delete|multi-file|many files/.test(value) || filesCount > 6) return "medium";
  return "low";
}

export function decideHarnessAction(input: { mode: HarnessMode; vague: boolean; blastRadius: BlastRadius; evidenceCount: number; assumptionCount?: number }): HarnessDecision {
  if (input.blastRadius === "critical") return "block_until_clarified";
  if (input.vague && input.evidenceCount <= 4) return "confirm_before_edit";
  if (input.assumptionCount && input.assumptionCount >= 3 && input.vague) return "confirm_before_edit";
  if (input.mode === "strict" && input.vague) return "confirm_before_edit";
  if (input.mode === "full-yolo") {
    return input.blastRadius === "high" && input.vague ? "confirm_before_edit" : "proceed_with_assumptions";
  }
  if (input.vague && (input.blastRadius === "high" || input.blastRadius === "medium")) return "confirm_before_edit";
  if (input.vague || input.evidenceCount === 0) return "proceed_with_assumptions";
  return "proceed";
}

export function stoplightForDecision(decision: HarnessDecision): Stoplight {
  if (decision === "proceed") return "green";
  if (decision === "proceed_with_assumptions") return "yellow";
  return "red";
}

export function termsToPlainOptions(text: string, stack?: StackProfile): string[] {
  const value = `${text} ${Object.values(stack ?? {}).join(" ")}`.toLowerCase();
  const options = new Set<string>();
  if (/best practices|properly|clean|refactor|architecture/.test(value)) options.add("split responsibilities so UI, data access, and workflow logic are not all in one place");
  if (/secure|auth|permission|session|secret/.test(value)) options.add("check authorization, secrets, validation, and session boundaries");
  if (/optimi[sz]e|performance|slow/.test(value)) options.add("measure the slow path first, then make the smallest targeted performance change");
  if (/\bui\b|professional|style|design/.test(value)) options.add("improve spacing, hierarchy, contrast, responsive behavior, and accessibility");
  if (/database|schema|migration|query/.test(value)) options.add("keep schema, migrations, repositories, and UI access separated");
  if (/test|verify|failing|bug|fix/.test(value)) options.add("prove the fix with the narrowest failing check or reproduction");
  return [...options];
}
