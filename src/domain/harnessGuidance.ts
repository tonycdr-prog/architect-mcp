import { deriveStackPackFromIngestedSource } from "./stackPackWorkflow.js";
import { discoverLlmsSources, listIngestedLlmsSources } from "./llmsSources.js";
import type { StackProfile, TriggeredStackGuidance } from "./types.js";

export function loadTriggeredStackGuidance(input: { request: string; stack?: StackProfile; selectedSourceIds?: string[]; limit?: number }): {
  guidance: TriggeredStackGuidance[];
  warnings: string[];
} {
  const sourceIds = selectSourceIds(input);
  const ingested = new Map(listIngestedLlmsSources().sources.map((source) => [source.id, source]));
  const known = new Map(discoverLlmsSources().map((source) => [source.id, source]));
  const guidance: TriggeredStackGuidance[] = [];
  const warnings: string[] = [];

  for (const sourceId of sourceIds.slice(0, input.limit ?? 6)) {
    const localSource = ingested.get(sourceId);
    const knownSource = known.get(sourceId);
    if (localSource?.status === "ok") {
      try {
        const derived = deriveStackPackFromIngestedSource(sourceId);
        guidance.push({
          sourceId,
          stack: derived.source.stack,
          category: derived.source.category,
          status: "loaded",
          summary: `Loaded ${derived.source.stack} guidance from local llms.txt snapshot.`,
          evidence: derived.matchedEvidence.slice(0, 5),
          ruleNames: derived.candidate.fileRules.map((rule) => rule.name)
        });
      } catch (error) {
        const warning = error instanceof Error ? error.message : String(error);
        warnings.push(warning);
        guidance.push(metadataGuidance(sourceId, knownSource, warning));
      }
    } else {
      const warning = localSource?.error ?? "No local ingested llms.txt snapshot available.";
      warnings.push(`${sourceId}: ${warning}`);
      guidance.push(metadataGuidance(sourceId, knownSource, warning));
    }
  }

  return { guidance, warnings };
}

function selectSourceIds(input: { request: string; stack?: StackProfile; selectedSourceIds?: string[] }): string[] {
  const value = `${input.request} ${Object.values(input.stack ?? {}).join(" ")}`.toLowerCase();
  const ids = new Set(input.selectedSourceIds ?? []);
  for (const source of discoverLlmsSources()) {
    if (value.includes(source.id) || value.includes(source.stack.toLowerCase())) ids.add(source.id);
  }
  if (/react|component|ui|frontend|professional|accessibility/.test(value)) ["react", "vite", "nextjs", "shadcn-ui"].forEach((id) => ids.add(id));
  if (/auth|session|permission|secure|security/.test(value)) ["auth0", "better-auth"].forEach((id) => ids.add(id));
  if (/database|schema|migration|query|sql|postgres|supabase/.test(value)) ["supabase", "prisma", "drizzle", "mongodb"].forEach((id) => ids.add(id));
  if (/api|backend|route|server|hono|trpc/.test(value)) ["hono", "trpc", "zod"].forEach((id) => ids.add(id));
  if (/deploy|production|host|railway|vercel|render|cloudflare/.test(value)) ["vercel", "railway", "render", "cloudflare"].forEach((id) => ids.add(id));
  if (/ai|agent|llm|openai|langchain/.test(value)) ["openai", "ai-sdk", "langchain"].forEach((id) => ids.add(id));
  if (ids.size === 0 && /best practices|properly|clean|fix|improve|optimi[sz]e|secure/.test(value)) {
    ["react", "hono", "supabase"].forEach((id) => ids.add(id));
  }
  return [...ids];
}

function metadataGuidance(sourceId: string, source: ReturnType<typeof discoverLlmsSources>[number] | undefined, warning: string): TriggeredStackGuidance {
  return {
    sourceId,
    stack: source?.stack ?? sourceId,
    category: source?.category ?? "platform",
    status: source ? "metadata-only" : "failed",
    summary: source?.notes ?? "No known llms.txt metadata for this source.",
    evidence: [],
    ruleNames: [],
    warning
  };
}
