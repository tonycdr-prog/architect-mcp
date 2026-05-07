const LOCAL_ONLY_TOOLS = new Set(["review_local_workspace", "scan_mcp_config_files"]);
const FUTURE_ADAPTER_TOOLS = new Set(["extract_harness_memory", "apply_harness_memory", "review_memory_relevance"]);

export function classifyToolPolicy(toolNames: string[]) {
  const tools = [...toolNames].sort().map((name) => {
    const policy = LOCAL_ONLY_TOOLS.has(name)
      ? "local-only" as const
      : FUTURE_ADAPTER_TOOLS.has(name) ? "future-adapter" as const : "hosted-safe" as const;
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
      futureAdapter: tools.filter((tool) => tool.policy === "future-adapter").length
    },
    tools
  };
}

function reasonFor(name: string, policy: "hosted-safe" | "local-only" | "future-adapter"): string {
  if (policy === "local-only") return "Reads local filesystem or user config paths.";
  if (policy === "future-adapter") return "Returns stateless proposals now; durable storage waits for an adapter.";
  if (/local_workspace/.test(name)) return "Local workspace access is disabled in hosted mode.";
  return "Operates on provided structured input or bundled package data.";
}
