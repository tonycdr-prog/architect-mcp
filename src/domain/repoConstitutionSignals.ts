import type {
  PackageMetadataSummary,
  PullRequestTemplateSummary,
  RecentPullRequestStyleSummary,
  RepoConstitutionProvenance,
  RepoConstitutionPullRequest,
  WorkflowSummary
} from "./repoConstitutionTypes.js";

export function collectExistingPaths(paths: Set<string>, candidates: string[]): string[] {
  return candidates.filter((path) => paths.has(path)).sort();
}

export function addPathProvenance(provenance: RepoConstitutionProvenance[], source: string, paths: string[], detail: string): void {
  for (const path of paths) {
    provenance.push({ source, path, detail });
  }
}

export function collectPullRequestTemplates(
  paths: Set<string>,
  artifacts: Map<string, string>,
  provenance: RepoConstitutionProvenance[]
): PullRequestTemplateSummary[] {
  const templatePaths = [...paths].filter(isPullRequestTemplatePath).sort();
  addPathProvenance(provenance, "pull-request-template", templatePaths, "Repository pull request template.");
  return templatePaths.map((path) => {
    const contentProvided = artifacts.has(path);
    const content = artifacts.get(path) ?? "";
    const visibleContent = stripHtmlComments(content).trim();
    return {
      path,
      contentProvided,
      hiddenCommentOnly: Boolean(content.trim()) && visibleContent.length === 0,
      headings: extractMarkdownHeadings(visibleContent || content),
      checklistItems: countChecklistItems(content),
      mentionsLinkedIssues: /fix(?:es)?\s+#|close(?:s)?\s+#|linked issues?/i.test(content),
      mentionsReleaseNotes: /release notes?|changelog/i.test(content),
      mentionsVerification: /verification|test plan|testing/i.test(content)
    };
  });
}

export function isPullRequestTemplatePath(path: string): boolean {
  return /^(?:\.github\/|docs\/)?pull_request_template\.md$/i.test(path) ||
    /^(?:\.github\/|docs\/)?PULL_REQUEST_TEMPLATE\/[^/]+\.md$/i.test(path);
}

export function summarizeRecentPullRequestStyle(
  pullRequests: RepoConstitutionPullRequest[],
  provenance: RepoConstitutionProvenance[]
): RecentPullRequestStyleSummary {
  const acceptedMergedSamples = pullRequests.filter((pullRequest) => pullRequest.merged === true);
  const nonMergedOrUnknownSamplesIgnored = pullRequests.length - acceptedMergedSamples.length;
  const botSamplesIgnored = acceptedMergedSamples.filter(isBotPullRequest).length;
  const acceptedSamples = acceptedMergedSamples.filter((pullRequest) => !isBotPullRequest(pullRequest));
  const maintainerAuthoredSamples = acceptedSamples.filter(isMaintainerAuthoredPullRequest).length;
  const headingCounts = new Map<string, number>();
  let checklistObserved = false;
  let releaseNoteObserved = false;
  let linkedIssueObserved = false;

  for (const pullRequest of acceptedSamples) {
    const body = pullRequest.body ?? "";
    for (const heading of extractMarkdownHeadings(body)) {
      headingCounts.set(heading, (headingCounts.get(heading) ?? 0) + 1);
    }
    checklistObserved ||= countChecklistItems(body) > 0;
    releaseNoteObserved ||= /release notes?|changelog/i.test(body);
    linkedIssueObserved ||= /fix(?:es)?\s+#|close(?:s)?\s+#|linked issues?/i.test(body);
    provenance.push({
      source: "accepted-pr-style",
      detail: pullRequest.number === undefined
        ? "Accepted merged PR body style sample."
        : `Accepted merged PR #${pullRequest.number} body style sample.`
    });
  }

  return {
    advisory: true,
    sampleSize: acceptedMergedSamples.length,
    acceptedSamples: acceptedSamples.length,
    maintainerAuthoredSamples,
    botSamplesIgnored,
    nonMergedOrUnknownSamplesIgnored,
    commonHeadings: [...headingCounts.entries()]
      .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
      .slice(0, 8)
      .map(([heading, count]) => ({ heading, count })),
    checklistObserved,
    releaseNoteObserved,
    linkedIssueObserved
  };
}

export function collectWorkflows(
  paths: Set<string>,
  artifacts: Map<string, string>,
  provenance: RepoConstitutionProvenance[]
): WorkflowSummary[] {
  const workflowPaths = [...paths].filter((path) => /^\.github\/workflows\/[^/]+\.ya?ml$/i.test(path)).sort();
  addPathProvenance(provenance, "ci-workflow", workflowPaths, "GitHub Actions workflow.");
  return workflowPaths.map((path) => ({
    path,
    name: firstMatch(artifacts.get(path) ?? "", /^name:\s*["']?([^"'\n]+)["']?/im),
    triggers: extractWorkflowTriggers(artifacts.get(path) ?? "")
  }));
}

export function collectPackageMetadata(
  paths: Set<string>,
  artifacts: Map<string, string>,
  provenance: RepoConstitutionProvenance[]
): PackageMetadataSummary[] {
  const metadata: PackageMetadataSummary[] = [];
  if (paths.has("package.json")) {
    const parsed = parseJsonObject(artifacts.get("package.json") ?? "");
    const scripts = typeof parsed?.scripts === "object" && parsed.scripts ? Object.keys(parsed.scripts as Record<string, unknown>).sort() : [];
    metadata.push({
      path: "package.json",
      manager: packageManagerFromLockfiles(paths),
      name: typeof parsed?.name === "string" ? parsed.name : undefined,
      version: typeof parsed?.version === "string" ? parsed.version : undefined,
      scripts
    });
    provenance.push({ source: "package-metadata", path: "package.json", detail: "Node package metadata." });
  }
  if (paths.has("Cargo.toml")) {
    metadata.push({
      path: "Cargo.toml",
      manager: "cargo",
      name: firstMatch(artifacts.get("Cargo.toml") ?? "", /^name\s*=\s*["']([^"']+)["']/m),
      version: firstMatch(artifacts.get("Cargo.toml") ?? "", /^version\s*=\s*["']([^"']+)["']/m)
    });
    provenance.push({ source: "package-metadata", path: "Cargo.toml", detail: "Rust package metadata." });
  }
  if (paths.has("pyproject.toml")) {
    metadata.push({
      path: "pyproject.toml",
      manager: "python",
      name: firstMatch(artifacts.get("pyproject.toml") ?? "", /^name\s*=\s*["']([^"']+)["']/m),
      version: firstMatch(artifacts.get("pyproject.toml") ?? "", /^version\s*=\s*["']([^"']+)["']/m)
    });
    provenance.push({ source: "package-metadata", path: "pyproject.toml", detail: "Python project metadata." });
  }
  if (paths.has("go.mod")) {
    metadata.push({ path: "go.mod", manager: "go", name: firstMatch(artifacts.get("go.mod") ?? "", /^module\s+(\S+)/m) });
    provenance.push({ source: "package-metadata", path: "go.mod", detail: "Go module metadata." });
  }
  return metadata;
}

export function summarizeLanguages(paths: Set<string>): string[] {
  const counts = new Map<string, number>();
  for (const path of paths) {
    const language = languageForPath(path);
    if (language) counts.set(language, (counts.get(language) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, 5)
    .map(([language]) => language);
}

export function inferRepoShape(input: { filePaths: Set<string>; packageMetadata: PackageMetadataSummary[]; primaryLanguages: string[] }): string {
  if (input.packageMetadata.length > 1) return "polyglot";
  if (input.filePaths.has("package.json") && [...input.filePaths].some((path) => path.startsWith("src/app/") || path.startsWith("pages/"))) return "web-app";
  if (input.filePaths.has("package.json")) return "node-package";
  if (input.filePaths.has("Cargo.toml")) return "rust-project";
  if (input.filePaths.has("pyproject.toml")) return "python-project";
  if (input.filePaths.has("go.mod")) return "go-module";
  return input.primaryLanguages[0] ? `${input.primaryLanguages[0]}-repo` : "unknown";
}

export function extractMarkdownHeadings(content: string): string[] {
  return [...content.matchAll(/^#{1,4}\s+(.+)$/gm)]
    .map((match) => match[1].replace(/[#*_`]/g, "").trim())
    .filter(Boolean)
    .slice(0, 12);
}

export function normalizeHeading(heading: string): string {
  return heading.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function isBotPullRequest(pullRequest: RepoConstitutionPullRequest): boolean {
  return /bot|dependabot|renovate/i.test(`${pullRequest.author ?? ""} ${pullRequest.authorAssociation ?? ""}`);
}

function isMaintainerAuthoredPullRequest(pullRequest: RepoConstitutionPullRequest): boolean {
  return /^(OWNER|MEMBER|COLLABORATOR)$/i.test(pullRequest.authorAssociation ?? "");
}

function packageManagerFromLockfiles(paths: Set<string>): string {
  if (paths.has("pnpm-lock.yaml")) return "pnpm";
  if (paths.has("yarn.lock")) return "yarn";
  if (paths.has("bun.lockb") || paths.has("bun.lock")) return "bun";
  return "npm";
}

function languageForPath(path: string): string | undefined {
  if (/\.(ts|tsx|js|jsx|mts|mjs|cjs|cts)$/i.test(path)) return "typescript-javascript";
  if (/\.py$/i.test(path)) return "python";
  if (/\.rs$/i.test(path)) return "rust";
  if (/\.go$/i.test(path)) return "go";
  if (/\.(java|kt|kts)$/i.test(path)) return "jvm";
  if (/\.cs$/i.test(path)) return "dotnet";
  return undefined;
}

function extractWorkflowTriggers(content: string): string[] {
  const triggers = new Set<string>();
  for (const match of content.matchAll(/^\s*(pull_request|pull_request_target|push|workflow_dispatch|schedule|release):/gm)) {
    triggers.add(match[1]);
  }
  const inline = /^on:\s*\[([^\]]+)\]/im.exec(content)?.[1] ?? "";
  for (const item of inline.split(",").map((value) => value.trim()).filter(Boolean)) {
    triggers.add(item);
  }
  const scalar = /^on:\s*([a-z_]+)\s*$/im.exec(content)?.[1];
  if (scalar) triggers.add(scalar);
  return [...triggers].sort();
}

function countChecklistItems(content: string): number {
  return [...content.matchAll(/^\s*[-*]\s+\[[ xX]\]/gm)].length;
}

function stripHtmlComments(content: string): string {
  return content.replace(/<!--[\s\S]*?-->/g, "");
}

function firstMatch(content: string, pattern: RegExp): string | undefined {
  return pattern.exec(content)?.[1]?.trim();
}

function parseJsonObject(content: string): Record<string, unknown> | undefined {
  try {
    const parsed = JSON.parse(content) as unknown;
    return typeof parsed === "object" && parsed !== null && !Array.isArray(parsed) ? parsed as Record<string, unknown> : undefined;
  } catch {
    return undefined;
  }
}
