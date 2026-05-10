import {
  FUTURE_ADAPTER_ARCHITECTURE_TOOL_NAMES,
  LOCAL_ONLY_ARCHITECTURE_TOOL_NAMES,
  isArchitectureToolRegistered,
  registeredArchitectureToolNames,
  type ToolPolicy
} from "../tools/toolRegistry.js";

const LOCAL_ONLY_TOOLS = new Set<string>(LOCAL_ONLY_ARCHITECTURE_TOOL_NAMES);
const FUTURE_ADAPTER_TOOLS = new Set<string>(FUTURE_ADAPTER_ARCHITECTURE_TOOL_NAMES);

export function classifyToolPolicy(toolNames: string[], options: { knownToolNames?: string[] } = {}) {
  const knownToolNames = new Set(options.knownToolNames ?? registeredArchitectureToolNames(true, "advanced"));
  const tools = [...toolNames].sort().map((name) => {
    const policy: ToolPolicy = !knownToolNames.has(name) || !isArchitectureToolRegistered(name)
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
  if (name === "promote_stack_pack_to_files") return "Can write stack-pack files and manifest updates when writeFiles=true.";
  if (policy === "local-only") return "Reads local filesystem or user config paths.";
  if (policy === "future-adapter") return "Returns stateless proposals now; durable storage waits for an adapter.";
  if (policy === "unknown") return "Tool is not in the registered tool catalog and cannot be assumed safe for hosted use.";
  if (/local_workspace/.test(name)) return "Local workspace access is disabled in hosted mode.";
  return "Operates on provided structured input or bundled package data.";
}
