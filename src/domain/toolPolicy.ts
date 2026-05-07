const LOCAL_ONLY_TOOLS = new Set(["review_local_workspace", "scan_mcp_config_files"]);
const FUTURE_ADAPTER_TOOLS = new Set(["extract_harness_memory", "apply_harness_memory", "review_memory_relevance"]);
const KNOWN_HOSTED_SAFE_TOOLS = new Set([
  "review_repo_structure",
  "interpret_implementation_intent",
  "audit_hosted_tool_policy",
  "score_stack_packs",
  "stack_pack_coverage_matrix",
  "review_agent_session",
  "list_client_integration_recipes",
  "review_mcp_config_security",
  "score_agent_artifacts"
]);

type ToolPolicy = "hosted-safe" | "local-only" | "future-adapter" | "unknown";

export function classifyToolPolicy(toolNames: string[], options: { knownToolNames?: string[] } = {}) {
  const knownToolNames = new Set([...(options.knownToolNames ?? []), ...KNOWN_HOSTED_SAFE_TOOLS, ...LOCAL_ONLY_TOOLS, ...FUTURE_ADAPTER_TOOLS]);
  const tools = toolNames.sort().map((name) => {
    const policy: ToolPolicy = !knownToolNames.has(name)
      ? "unknown"
      : LOCAL_ONLY_TOOLS.has(name)
        ? "local-only"
        : FUTURE_ADAPTER_TOOLS.has(name) ? "future-adapter" : "hosted-safe";
    return {
      name,
      policy,
      reason: reasonFor(name, policy)
    };
  });

  return {
    summary: {
      hostedSafe: tools.filter((tool) => tool.policy === "hosted-safe").length,
      localOnly: tools.filter((tool) => tool.policy === "local-only").length,
      futureAdapter: tools.filter((tool) => tool.policy === "future-adapter").length,
      unknown: tools.filter((tool) => tool.policy === "unknown").length
    },
    warnings: tools.filter((tool) => tool.policy === "unknown").map((tool) => `Unknown tool "${tool.name}" was not classified as hosted-safe.`),
    tools
  };
}

function reasonFor(name: string, policy: ToolPolicy): string {
  if (policy === "local-only") return "Reads local filesystem or user config paths.";
  if (policy === "future-adapter") return "Returns stateless proposals now; durable storage waits for an adapter.";
  if (policy === "unknown") return "Tool is not in the registered tool catalog and cannot be assumed safe for hosted use.";
  if (/local_workspace/.test(name)) return "Local workspace access is disabled in hosted mode.";
  return "Operates on provided structured input or bundled package data.";
}
