import type { AppArchetype, ProjectBrief } from "./types.js";

export function inferAppArchetype(brief: ProjectBrief): AppArchetype {
  const text = [
    brief.idea,
    brief.users,
    brief.coreFlows?.join(" "),
    Object.values(brief.stack ?? {}).join(" ")
  ].filter(Boolean).join(" ").toLowerCase();

  if (/field|inspection|mobile|engineer|offline|expo|react native/.test(text)) return "mobile-field-app";
  if (/admin|internal|ops|backoffice|crm/.test(text)) return "internal-admin";
  if (/marketplace|seller|buyer|listing|checkout/.test(text)) return "marketplace";
  if (/ai|agent|workflow|automation|prompt|llm/.test(text)) return "ai-workflow-tool";
  if (/docs|content|blog|knowledge base|documentation/.test(text)) return "content-docs-site";
  if (/saas|dashboard|billing|subscription|team/.test(text)) return "saas-dashboard";
  return "custom-app";
}

export function archetypeQuestions(archetype: AppArchetype): string[] {
  switch (archetype) {
    case "saas-dashboard":
      return ["Which team/account boundary owns authorization?", "Which billing/admin workflows are V1 versus later?"];
    case "marketplace":
      return ["Which actor owns each workflow: buyer, seller, admin, or operator?", "Where do payments, disputes, and listings live?"];
    case "internal-admin":
      return ["Which tables/forms are operational workflows versus reporting?", "Which admin actions need audit trails?"];
    case "mobile-field-app":
      return ["Which flows must work offline or in poor connectivity?", "Which sync boundary owns local versus server state?"];
    case "ai-workflow-tool":
      return ["Where do prompts, evals, traces, and model adapters live?", "Which outputs need review before user-visible action?"];
    case "content-docs-site":
      return ["Which content source owns pages and metadata?", "Which build-time checks prevent broken content?"];
    default:
      return ["Which module owns each core workflow?", "Which verification command proves the first slice works?"];
  }
}
