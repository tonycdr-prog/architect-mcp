export const untrustedInputSources = [
  "issue_pr_text",
  "repo_doc",
  "web_snippet",
  "mcp_response",
  "tool_output",
  "adapter_output",
  "test_log",
  "dependency_output",
  "memory"
] as const;

export type UntrustedInputSource = typeof untrustedInputSources[number];
export type RawUntrustedInput = { source?: string; label?: string; handling?: string };
export type NormalizedUntrustedInput = { source: UntrustedInputSource; label: string };

const sourceLabels: Record<UntrustedInputSource, string> = {
  issue_pr_text: "issue/pr text",
  repo_doc: "repo docs",
  web_snippet: "web snippet",
  mcp_response: "mcp response",
  tool_output: "tool output",
  adapter_output: "adapter output",
  test_log: "test log",
  dependency_output: "dependency output",
  memory: "memory/repo context"
};

export function normalizeUntrustedInputs(inputs: RawUntrustedInput[] | undefined): NormalizedUntrustedInput[] {
  const seen = new Set<UntrustedInputSource>();
  const normalized: NormalizedUntrustedInput[] = [];
  for (const input of inputs ?? []) {
    const source = input.source;
    if (!isUntrustedInputSource(source) || seen.has(source)) continue;
    seen.add(source);
    normalized.push({ source, label: sourceLabels[source] });
  }
  return normalized;
}

export function isUntrustedInputSource(value: unknown): value is UntrustedInputSource {
  return typeof value === "string" && (untrustedInputSources as readonly string[]).includes(value);
}
