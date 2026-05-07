import type { ProjectBrief } from "./types.js";

export const QUESTIONS = [
  {
    id: "users",
    question: "Who is the primary user, and what job are they trying to get done?",
    recommendedAnswer: "A specific user type plus one concrete job, for example: solo founders need to turn vague app ideas into repo-ready architecture contracts."
  },
  {
    id: "coreFlows",
    question: "What are the 2-3 core workflows V1 must support?",
    recommendedAnswer: "Intake interview, architecture contract generation, source-backed stack standards, implementation review, and evidence-before-completion checks."
  },
  {
    id: "stack",
    question: "Which stack should V1 assume?",
    recommendedAnswer: "TypeScript MCP server, stdio transport first, hosted HTTP transport later, JSON/Markdown outputs."
  },
  {
    id: "storage",
    question: "Does V1 need persistent storage?",
    recommendedAnswer: "No. Keep V1 stateless. Accept project briefs and file trees as input; add saved org/team templates in V2."
  },
  {
    id: "enforcement",
    question: "Should this block agent actions or only advise?",
    recommendedAnswer: "V1 should advise with error/warning findings. Blocking belongs in a later integration with hooks or CI."
  },
  {
    id: "repoLayout",
    question: "Is this a fresh repo or an existing repo with established folders?",
    recommendedAnswer: "For existing repos, provide or infer a repoLayout mapping so stack packs adapt to current paths instead of forcing src/** conventions."
  },
  {
    id: "risk",
    question: "What mistakes would be expensive or embarrassing if the agent made them?",
    recommendedAnswer: "Name the top risks, for example: giant screen files, client/server secret leaks, direct DB access from UI, or missing tests around generated workflows."
  },
  {
    id: "verification",
    question: "What commands prove the generated change is acceptable?",
    recommendedAnswer: "List exact checks such as typecheck, unit tests, lint, architecture review, browser smoke tests, or mobile smoke tests."
  }
] as const;

export function getNextIntakeQuestionForBrief(brief: ProjectBrief) {
  const answered = new Set(Object.keys(brief).filter((key) => isBriefFieldAnswered(brief, key as keyof ProjectBrief)));
  return QUESTIONS.find((question) => !answered.has(question.id)) ?? {
    id: "ready",
    question: "Are we ready to generate the architecture contract?",
    recommendedAnswer: "Yes. Generate a contract now, then review the repo structure against it."
  };
}

export function isBriefFieldAnswered(brief: ProjectBrief, field: keyof ProjectBrief): boolean {
  const value = brief[field];
  if (typeof value === "string") return value.trim().length > 0;
  if (Array.isArray(value)) return value.some((item) => typeof item !== "string" || item.trim().length > 0);
  if (field === "stack") return Boolean(brief.stack && Object.values(brief.stack).some((item) => item?.trim()));
  if (field === "repoLayout") return Boolean(brief.repoLayout && Object.keys(brief.repoLayout.pathMap ?? {}).length > 0);
  if (value && typeof value === "object") return Object.keys(value).length > 0;
  return Boolean(value);
}
