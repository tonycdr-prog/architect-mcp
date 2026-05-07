import type { SkillCatalogEntry, SkillCatalogQuery, SkillRecommendation } from "./types.js";

export const BUILT_IN_SKILL_CATALOG: SkillCatalogEntry[] = [
  {
    id: "mcp-tool-discovery",
    name: "MCP Tool Discovery",
    category: "mcp",
    summary: "Patterns for listing MCP servers, inspecting tool schemas, calling tools, and recording structured outputs.",
    patterns: ["tool discovery", "schema inspection", "CLI call recipes", "structured output capture"],
    recommendedWhen: ["mcp", "tool", "schema", "client", "integration", "cli"],
    cautions: ["Do not require a specific CLI; provide examples that clients can adapt."],
    source: "built-in"
  },
  {
    id: "mcp-config-security",
    name: "MCP Config Security",
    category: "security",
    summary: "Patterns for reviewing MCP config files for secrets, shell injection, unpinned packages, and unsafe commands.",
    patterns: ["hardcoded secret detection", "shell injection checks", "version pinning", "approved server review"],
    recommendedWhen: ["mcp", "security", "config", "secret", "token", "npx", "shell", "hosted"],
    cautions: ["Never echo secret values; report only redacted evidence and remediation."],
    source: "built-in"
  },
  {
    id: "scoped-memory",
    name: "Scoped Memory",
    category: "memory",
    summary: "Patterns for user, project, session, and codebase memory scopes with disclosure and forgetting controls.",
    patterns: ["memory scopes", "domain-organized memory", "token-budgeted retrieval", "explicit disclosure"],
    recommendedWhen: ["memory", "remember", "preference", "session", "handoff", "github"],
    cautions: ["Memory is advisory and must not override the current user request."],
    source: "built-in"
  },
  {
    id: "agent-instruction-quality",
    name: "Agent Instruction Quality",
    category: "docs",
    summary: "Patterns for concise AGENTS.md, Cursor rules, and llms.txt files that separate human docs from agent workflow instructions.",
    patterns: ["AGENTS.md structure", "LLM navigation", "setup commands", "testing instructions", "repo boundaries"],
    recommendedWhen: ["agents.md", "instructions", "llms.txt", "docs", "cursor", "onboarding"],
    cautions: ["Generated instructions should be operational, not a duplicate of the README."],
    source: "built-in"
  },
  {
    id: "verification-honesty",
    name: "Verification Honesty",
    category: "verification",
    summary: "Patterns for evidence-before-completion claims, skipped-check reporting, and root-cause proof.",
    patterns: ["fresh verification evidence", "final output contract", "root-cause evidence", "skipped check disclosure"],
    recommendedWhen: ["verify", "done", "complete", "fixed", "test", "root cause", "best practices"],
    cautions: ["Do not claim success from confidence or partial checks."],
    source: "built-in"
  },
  {
    id: "eval-fixture-discipline",
    name: "Eval Fixture Discipline",
    category: "eval",
    summary: "Patterns for criteria-driven fixtures, pass/fail scoring, regression datasets, and iteration on failures.",
    patterns: ["eval criteria", "fixture datasets", "scored results", "failure analysis"],
    recommendedWhen: ["eval", "quality", "benchmark", "regression", "novice prompt", "harness"],
    cautions: ["Evaluate architect-mcp behavior and response shapes, not a model's personality."],
    source: "built-in"
  }
];

export function listSkillCatalog(query: SkillCatalogQuery = {}): { skills: SkillCatalogEntry[] } {
  return {
    skills: catalogFor(query).filter((entry) =>
      (!query.categories?.length || query.categories.includes(entry.category)) &&
      (!query.request || relevanceScore(query.request, query.stack, entry) > 0)
    ).slice(0, query.limit ?? 50)
  };
}

export function recommendSkills(query: SkillCatalogQuery): { recommendations: SkillRecommendation[]; warnings: string[] } {
  const request = query.request?.trim() ?? "";
  const recommendations = catalogFor(query)
    .map((skill) => ({ skill, score: relevanceScore(request, query.stack, skill) }))
    .filter((item) => item.score > 0 && (!query.categories?.length || query.categories.includes(item.skill.category)))
    .sort((a, b) => b.score - a.score)
    .slice(0, query.limit ?? 6)
    .map(({ skill, score }) => ({
      skill,
      score,
      confidence: score >= 6 ? "high" as const : score >= 3 ? "medium" as const : "low" as const,
      reason: reasonFor(skill, request)
    }));

  return {
    recommendations,
    warnings: query.suppliedSkills?.length
      ? ["Client-supplied skills are advisory metadata only; do not execute arbitrary skill logic."]
      : []
  };
}

export function reviewSuppliedSkills(input: { skills: SkillCatalogEntry[] }): { valid: boolean; findings: Array<{ id: string; severity: "warning" | "error"; message: string; recommendation: string }> } {
  const findings = input.skills.flatMap((skill) => {
    const skillFindings: Array<{ id: string; severity: "warning" | "error"; message: string; recommendation: string }> = [];
    if (skill.source !== "client-supplied") {
      skillFindings.push({
        id: skill.id,
        severity: "warning",
        message: "Supplied skill metadata should be marked as client-supplied.",
        recommendation: "Set source to client-supplied when passing external skill metadata."
      });
    }
    if (skill.patterns.length === 0 || skill.recommendedWhen.length === 0) {
      skillFindings.push({
        id: skill.id,
        severity: "error",
        message: "Skill metadata lacks recommendation patterns.",
        recommendation: "Provide patterns and recommendedWhen terms so the catalog can rank it safely."
      });
    }
    if (skill.cautions.some(isUnsafeCaution)) {
      skillFindings.push({
        id: skill.id,
        severity: "error",
        message: "Skill caution suggests unsafe agent behavior.",
        recommendation: "Skill metadata must preserve current user intent and avoid arbitrary execution."
      });
    }
    return skillFindings;
  });

  return {
    valid: findings.every((finding) => finding.severity !== "error"),
    findings
  };
}

function isUnsafeCaution(caution: string): boolean {
  if (/\b(do not|don't|never|avoid|must not)\b.{0,40}\b(execute arbitrary|run arbitrary|ignore user)\b/i.test(caution)) return false;
  return /execute arbitrary|run arbitrary|ignore user/i.test(caution);
}

function catalogFor(query: SkillCatalogQuery): SkillCatalogEntry[] {
  return [
    ...BUILT_IN_SKILL_CATALOG,
    ...(query.suppliedSkills ?? [])
  ];
}

function relevanceScore(request: string, stack: SkillCatalogQuery["stack"], skill: SkillCatalogEntry): number {
  const terms = new Set(`${request} ${Object.values(stack ?? {}).join(" ")}`.toLowerCase().match(/[a-z0-9][a-z0-9_.-]{2,}/g) ?? []);
  let score = 0;
  for (const term of skill.recommendedWhen) {
    if (terms.has(term.toLowerCase())) score += 3;
  }
  for (const term of terms) {
    if (`${skill.name} ${skill.summary} ${skill.patterns.join(" ")}`.toLowerCase().includes(term)) score += 1;
  }
  return score;
}

function reasonFor(skill: SkillCatalogEntry, request: string): string {
  return `Matches "${request.slice(0, 120)}" with ${skill.category} patterns: ${skill.patterns.slice(0, 3).join(", ")}.`;
}
