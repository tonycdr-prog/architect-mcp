import { publicSafeSummary } from "./publicSafetyText.js";

export function safeForgeList(values: string[]): string[] {
  return values.map(safeForgeSummary).filter((value) => value.length > 0);
}

export function safeForgeText(value: string): string {
  const safe = publicSafeSummary(value).value;
  if (hasRawRepoContent(safe)) return "[redacted-raw-repo-content]";
  return safe;
}

export function safeForgeSummary(value: string): string {
  const safe = publicSafeSummary(value).value;
  if (hasRawRepoContent(safe)) return "[redacted-raw-repo-content]";
  return safe;
}

export function safeForgeScore(value: number): number {
  return Math.max(0, Math.min(100, Math.round(Number.isFinite(value) ? value : 0)));
}

export function forgeBulletList(values: string[]): string {
  return values.length > 0 ? values.map((value) => `- ${safeForgeSummary(value)}`).join("\n") : "- Record maintainer-approved verification before mutation.";
}

export function shortForgeReason(value: string): string {
  const safe = safeForgeSummary(value)
    .replace(/^route\s+[a-z_]+\s+at\s+score\s+\d+\.\s*/i, "")
    .replace(/^primary blocker:\s*/i, "")
    .trim();
  const base = safe.length > 0 ? safe : "review foundry decision";
  return base.length > 74 ? `${base.slice(0, 71).trim()}...` : base;
}

function hasRawRepoContent(value: string): boolean {
  const codeLines = value.split("\n").filter((line) =>
    /^\s*(?:(?:import|export|const|let|var|function|class|interface|type|if|for|while|return|await)\b|[{}]\s*$)/.test(line)
  );
  return codeLines.length >= 2 ||
    /\b(?:import\s+[^;]+?\s+from\s+["']|export\s+(?:const|function|class|type|interface)|(?:const|let|var)\s+\w+\s*=|function\s+\w+\s*\(|class\s+\w+\s*\{|interface\s+\w+\s*\{|type\s+\w+\s*=|process\.env\.)/.test(value);
}
