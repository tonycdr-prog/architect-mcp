import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { listFoundationPacks, validateFoundationPacks } from "../domain/foundationPacks.js";
import { discoverLlmsSources, fetchLlmsSource, listIngestedLlmsSources } from "../domain/llmsSources.js";
import { analyzeStackPackConflicts, deriveStackPackFromIngestedSource, diffStackPackVersions, ingestLlmsTxt, promoteStackPackCandidate, proposeStackPackRules, reviewStackPackCandidate, stackPackExpansionStrategy } from "../domain/stackPackWorkflow.js";
import { listStackPacks, validateStackPacks } from "../domain/stackPacks.js";
import type { StackPack, StackPackCandidate } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { derivedStackPackSchema, genericObjectOutputSchema, llmsFetchSchema, llmsSourceQuerySchema, stackPackCandidateInputSchema, stackPackCandidateSchema, validationOutputSchema } from "./schemas.js";

export function registerPackTools(server: McpServer): void {
  server.registerTool(
    "discover_llms_sources",
    {
      title: "Discover LLMs.txt Sources",
      description: "List known upstream llms.txt sources for stack-pack rule generation.",
      inputSchema: {
        query: llmsSourceQuerySchema.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ query }) => safeJsonResponse(() => ({ sources: discoverLlmsSources(query) }))
  );

  server.registerTool(
    "fetch_llms_source",
    {
      title: "Fetch LLMs.txt Source",
      description: "Fetch an upstream llms.txt source and return a hash-stamped snapshot.",
      inputSchema: {
        request: llmsFetchSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => fetchLlmsSource(request.source, {
      preferFull: request.preferFull,
      maxBytes: request.maxBytes
    }))
  );

  server.registerTool(
    "list_ingested_llms_sources",
    {
      title: "List Ingested LLMs.txt Sources",
      description: "List locally ingested llms.txt snapshots and their hashes/status.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => listIngestedLlmsSources())
  );

  server.registerTool(
    "ingest_llms_txt",
    {
      title: "Ingest LLMs.txt",
      description: "Fetch an upstream llms.txt source and produce a reviewed stack-pack candidate input snapshot.",
      inputSchema: {
        request: llmsFetchSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => ingestLlmsTxt(request.source, {
      preferFull: request.preferFull,
      maxBytes: request.maxBytes
    }))
  );

  server.registerTool(
    "stack_pack_expansion_strategy",
    {
      title: "Stack Pack Expansion Strategy",
      description: "Return the priority stacks and quality contract for stack-pack expansion.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => stackPackExpansionStrategy())
  );

  server.registerTool(
    "derive_stack_pack_from_llms_source",
    {
      title: "Derive Stack Pack From LLMs.txt Source",
      description: "Create a source-backed stack-pack candidate from a locally ingested llms.txt snapshot and run quality/conflict review.",
      inputSchema: {
        request: derivedStackPackSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ request }) => safeJsonResponse(() => deriveStackPackFromIngestedSource(request.sourceId))
  );

  server.registerTool(
    "propose_stack_pack_rules",
    {
      title: "Propose Stack Pack Rules",
      description: "Propose candidate stack-pack rules from local source text or an llms.txt-style snapshot.",
      inputSchema: {
        input: stackPackCandidateInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => ({ candidate: proposeStackPackRules(input) }))
  );

  server.registerTool(
    "review_stack_pack_candidate",
    {
      title: "Review Stack Pack Candidate",
      description: "Review candidate stack-pack rules before promotion.",
      inputSchema: {
        candidate: stackPackCandidateSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ candidate }) => safeJsonResponse(() => reviewStackPackCandidate(candidate as StackPackCandidate))
  );

  server.registerTool(
    "promote_stack_pack_candidate",
    {
      title: "Promote Stack Pack Candidate",
      description: "Validate and return a promotable stack-pack object. Does not write files.",
      inputSchema: {
        candidate: stackPackCandidateSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ candidate }) => safeJsonResponse(() => promoteStackPackCandidate(candidate as StackPackCandidate))
  );

  server.registerTool(
    "diff_stack_pack_versions",
    {
      title: "Diff Stack Pack Versions",
      description: "Diff two stack-pack versions and flag potentially breaking rule changes.",
      inputSchema: {
        before: stackPackCandidateSchema.or(genericObjectOutputSchema),
        after: stackPackCandidateSchema.or(genericObjectOutputSchema)
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ before, after }) => safeJsonResponse(() => diffStackPackVersions(before as StackPack, after as StackPack))
  );

  server.registerTool(
    "analyze_stack_pack_conflicts",
    {
      title: "Analyze Stack Pack Conflicts",
      description: "Report duplicate or overlapping rules across selected stack-pack objects before promotion.",
      inputSchema: {
        stackPacks: stackPackCandidateSchema.or(genericObjectOutputSchema).array()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ stackPacks }) => safeJsonResponse(() => ({ conflicts: analyzeStackPackConflicts(stackPacks as StackPack[]) }))
  );

  server.registerTool(
    "list_foundation_packs",
    {
      title: "List Foundation Packs",
      description: "List versioned foundation packs that apply before framework-specific stack packs.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => ({ foundationPacks: listFoundationPacks() }))
  );

  server.registerTool(
    "validate_foundation_packs",
    {
      title: "Validate Foundation Packs",
      description: "Validate versioned foundation packs.",
      inputSchema: {},
      outputSchema: validationOutputSchema
    },
    async () => safeJsonResponse(() => validateFoundationPacks())
  );

  server.registerTool(
    "list_stack_packs",
    {
      title: "List Stack Packs",
      description: "List available stack-specific architecture rule packs.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => ({ stackPacks: listStackPacks() }))
  );

  server.registerTool(
    "validate_stack_packs",
    {
      title: "Validate Stack Packs",
      description: "Validate stack-pack files against the pack quality bar.",
      inputSchema: {},
      outputSchema: validationOutputSchema
    },
    async () => safeJsonResponse(() => validateStackPacks())
  );
}
