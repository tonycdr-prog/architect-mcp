import { QUESTIONS } from "./intakeQuestions.js";
import type { GrillMeChallenge, ProjectBrief, SpecCompleteness } from "./types.js";

export function getBlockers(brief: ProjectBrief): string[] {
  const blockers: string[] = [];
  if (!brief.idea?.trim()) blockers.push("Missing product idea.");
  if (!brief.users?.trim()) blockers.push("Missing primary user and job-to-be-done.");
  if (!brief.coreFlows || brief.coreFlows.filter((flow) => flow.trim()).length < 2) blockers.push("Need at least two concrete V1 workflows.");
  if (!brief.stack || Object.values(brief.stack).filter(Boolean).length === 0) blockers.push("Missing stack profile.");
  if (!brief.risk?.trim()) blockers.push("Missing expensive or embarrassing failure modes.");
  if (!brief.verification || brief.verification.filter((check) => check.trim()).length === 0) blockers.push("Missing verification commands.");
  return blockers;
}

export function getChallenges(brief: ProjectBrief, missingFields: string[]): GrillMeChallenge[] {
  const challenges: GrillMeChallenge[] = missingFields.map((field) => ({
    field,
    severity: "blocker",
    question: questionForField(field),
    whyItMatters: "Architecture guardrails are only useful when this answer is explicit before files are generated."
  }));

  if (brief.coreFlows && brief.coreFlows.length > 5) {
    challenges.push({
      field: "coreFlows",
      severity: "pressure-test",
      question: "Which 2-3 workflows are truly V1, and which ones should be deferred?",
      whyItMatters: "Too many first-version workflows usually pushes agents toward broad app-level files."
    });
  }
  if (brief.stack?.frontend && brief.stack?.database && !brief.stack.backend) {
    challenges.push({
      field: "stack.backend",
      severity: "blocker",
      question: "What server boundary owns database access so UI code never talks directly to the database?",
      whyItMatters: "Frontend plus database without a named server boundary is a common source of client/server leaks."
    });
  }
  if (brief.stack?.frontend && !mentionsFrontendOwnership(brief)) {
    challenges.push({
      field: "stack.frontend",
      severity: "pressure-test",
      question: "Which frontend feature folders, screens/routes, and state/data hooks own each core workflow?",
      whyItMatters: "Frontend apps become monolithic when screens own fetching, mutation, validation, modal orchestration, and rendering in one file."
    });
  }
  if (brief.stack?.backend && !mentionsBackendBoundaries(brief)) {
    challenges.push({
      field: "stack.backend",
      severity: "pressure-test",
      question: "Where do routes stop and services/use-cases begin?",
      whyItMatters: "Backend agents often create giant route/controller files unless service and adapter boundaries are explicit up front."
    });
  }
  if (brief.stack?.auth && !hasTrustBoundary(brief)) {
    challenges.push({
      field: "stack.auth",
      severity: "blocker",
      question: "Which server-side trust boundary validates sessions, roles, and tenant access?",
      whyItMatters: "Auth without a named trust boundary often leaves authorization checks scattered across UI and route files."
    });
  }
  if (brief.stack?.deployment && /host|railway|vercel|cloud|http|server/i.test(brief.stack.deployment) && /filesystem|local path|scan/i.test(brief.idea)) {
    challenges.push({
      field: "stack.deployment",
      severity: "blocker",
      question: "How will hosted mode avoid reading arbitrary server-local filesystem paths?",
      whyItMatters: "Hosted tools need an explicit safety boundary between client-provided summaries and server-local IO."
    });
  }
  if (brief.storage && /yes|database|persist/i.test(brief.storage) && !brief.dataEntities?.length) {
    challenges.push({
      field: "dataEntities",
      severity: "pressure-test",
      question: "What are the first data entities and which module owns each one?",
      whyItMatters: "Persistence without entity ownership tends to create shared schema and service dumping grounds."
    });
  }
  if (brief.stack?.database && brief.dataEntities?.length && !mentionsDatabaseDiscipline(brief)) {
    challenges.push({
      field: "stack.database",
      severity: "pressure-test",
      question: "What is the migration, schema, seed, and repository/query-module discipline for these entities?",
      whyItMatters: "Database-backed apps drift when schema, migrations, and query ownership are not assigned before implementation."
    });
  }
  if (!mentionsAgentHarness(brief)) {
    challenges.push({
      field: "verification",
      severity: "pressure-test",
      question: "Which repo-owned agent harness files should be generated or updated before coding?",
      whyItMatters: "Future agents need committed instructions, architecture contracts, and review gates rather than relying on chat context."
    });
  }
  if (brief.enforcement && /block|ci|fail/i.test(brief.enforcement) && !brief.verification?.length) {
    challenges.push({
      field: "verification",
      severity: "blocker",
      question: "Which exact commands or review gates should block changes?",
      whyItMatters: "Blocking enforcement needs deterministic checks, not general best-practice advice."
    });
  }
  const blockers = challenges.filter((challenge) => challenge.severity === "blocker");
  const pressureTests = challenges.filter((challenge) => challenge.severity !== "blocker").slice(0, Math.max(0, 8 - blockers.length));
  return [...blockers, ...pressureTests];
}

export function scoreReadiness(answeredCount: number, blockerCount: number, challengeCount: number): number {
  const baseScore = Math.round((answeredCount / QUESTIONS.length) * 100);
  return Math.max(0, Math.min(100, baseScore - blockerCount * 8 - Math.max(0, challengeCount - blockerCount) * 3));
}

export function scoreSpecCompleteness(brief: ProjectBrief, challenges: GrillMeChallenge[]): SpecCompleteness {
  const checks = [
    { id: "users", status: brief.users?.trim() ? "pass" as const : "fail" as const, summary: brief.users?.trim() ? "Primary user and job are named." : "Primary user and job are missing." },
    { id: "workflows", status: brief.coreFlows && brief.coreFlows.filter((flow) => flow.trim()).length >= 2 ? "pass" as const : "fail" as const, summary: brief.coreFlows && brief.coreFlows.filter((flow) => flow.trim()).length >= 2 ? "At least two core workflows are named." : "Need at least two core workflows." },
    { id: "stack", status: brief.stack && Object.values(brief.stack).some(Boolean) ? "pass" as const : "fail" as const, summary: brief.stack && Object.values(brief.stack).some(Boolean) ? "Stack profile is present." : "Stack profile is missing." },
    { id: "data-ownership", status: !brief.stack?.database || mentionsDatabaseDiscipline(brief) ? "pass" as const : "warn" as const, summary: !brief.stack?.database || mentionsDatabaseDiscipline(brief) ? "Data ownership is explicit enough for the selected stack." : "Database is selected but schema/migration/repository ownership is vague." },
    { id: "verification", status: brief.verification?.some((check) => check.trim()) ? "pass" as const : "fail" as const, summary: brief.verification?.some((check) => check.trim()) ? "Verification commands are named." : "Verification commands are missing." },
    { id: "harness", status: mentionsAgentHarness(brief) ? "pass" as const : "warn" as const, summary: mentionsAgentHarness(brief) ? "Agent harness expectations are mentioned." : "Agent harness artifacts are not explicitly mentioned." },
    { id: "risk", status: brief.risk?.trim() ? "pass" as const : "fail" as const, summary: brief.risk?.trim() ? "Expensive failure modes are named." : "Risk/failure modes are missing." },
    { id: "pressure-tests", status: challenges.some((challenge) => challenge.severity === "blocker") ? "fail" as const : challenges.length ? "warn" as const : "pass" as const, summary: challenges.some((challenge) => challenge.severity === "blocker") ? "Blocker pressure tests remain." : challenges.length ? "Pressure-test questions remain." : "No pressure-test questions remain." }
  ];
  return {
    score: Math.round(checks.reduce((total, check) => total + (check.status === "pass" ? 12.5 : check.status === "warn" ? 6 : 0), 0)),
    checks
  };
}

function hasTrustBoundary(brief: ProjectBrief): boolean {
  return /server|api|backend|route|middleware|jwt|session|tenant|role/i.test([
    brief.stack?.backend,
    brief.constraints?.join(" "),
    brief.risk,
    brief.enforcement
  ].filter(Boolean).join(" "));
}

function mentionsFrontendOwnership(brief: ProjectBrief): boolean {
  return /feature|screen|route|component|hook|state|view|page/i.test(searchableBriefText(brief));
}

function mentionsBackendBoundaries(brief: ProjectBrief): boolean {
  return /service|use.?case|route|controller|adapter|repository|handler/i.test(searchableBriefText(brief));
}

function mentionsDatabaseDiscipline(brief: ProjectBrief): boolean {
  return /migration|schema|repository|query|seed|drizzle|prisma|supabase/i.test(searchableBriefText(brief));
}

function mentionsAgentHarness(brief: ProjectBrief): boolean {
  return /AGENTS\.md|agent|harness|architecture contract|review gate|cursor|claude|copilot|instructions/i.test(searchableBriefText(brief));
}

function searchableBriefText(brief: ProjectBrief): string {
  return [
    brief.idea,
    brief.users,
    brief.coreFlows?.join(" "),
    brief.dataEntities?.join(" "),
    Object.values(brief.stack ?? {}).join(" "),
    brief.constraints?.join(" "),
    brief.storage,
    brief.enforcement,
    brief.risk,
    brief.verification?.join(" ")
  ].filter(Boolean).join(" ");
}

function questionForField(field: string): string {
  return QUESTIONS.find((question) => question.id === field)?.question ?? `What should ${field} be?`;
}
