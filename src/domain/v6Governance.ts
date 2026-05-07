import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";
import { createReviewReport } from "./reviewReport.js";
import type { ProjectBrief, ReviewReport, ReviewViolation } from "./types.js";

const policyBundleSchema = z.object({
  id: z.string(),
  name: z.string(),
  version: z.string(),
  profile: z.enum(["minimal", "balanced", "strict", "migration", "frontend-heavy", "backend-heavy", "security-focused", "novice-friendly"]),
  rules: z.array(z.string()).min(1),
  recommendedWhen: z.array(z.string()).min(1)
}).strict();

export type PolicyBundle = z.infer<typeof policyBundleSchema>;

export function listPolicyBundles(): { bundles: PolicyBundle[] } {
  return { bundles: loadPolicyBundles() };
}

export function validatePolicyBundles(): { valid: boolean; errors: string[]; bundleCount: number } {
  const errors: string[] = [];
  const ids = new Set<string>();
  try {
    const { bundles } = loadPolicyBundleFiles();
    errors.push(...validatePolicyBundleManifest(bundles));
    for (const { bundle } of bundles) {
      if (ids.has(bundle.id)) errors.push(`Duplicate policy bundle id: ${bundle.id}`);
      ids.add(bundle.id);
      if (!/^\d+\.\d+\.\d+$/.test(bundle.version)) errors.push(`${bundle.id}: version must be semver.`);
    }
  } catch (error) {
    errors.push(error instanceof Error ? error.message : String(error));
  }
  return { valid: errors.length === 0, errors, bundleCount: ids.size };
}

export function previewPolicyBundle(input: { bundleId?: string; brief?: ProjectBrief; findings?: ReviewViolation[] } = {}) {
  const bundles = loadPolicyBundles();
  const selected = input.bundleId ? bundles.find((bundle) => bundle.id === input.bundleId) : recommendBundle(bundles, input.brief);
  const report = createReviewReport(input.findings ?? [], { mode: selected?.profile === "strict" || selected?.profile === "security-focused" ? "strict" : "summary" });
  return {
    bundle: selected,
    report,
    coverage: {
      rules: selected?.rules.length ?? 0,
      recommendedWhen: selected?.recommendedWhen ?? []
    },
    warnings: selected ? [] : ["No policy bundle matched the supplied request."]
  };
}

export function summarizeSessionContinuity(input: { summaries?: string[]; assumptions?: Array<{ statement?: string; createdAt?: string }>; maxItems?: number } = {}) {
  const maxItems = input.maxItems ?? 5;
  const summaries = (input.summaries ?? []).slice(-maxItems);
  const assumptions = (input.assumptions ?? []).slice(-maxItems);
  return {
    continuationContext: [...summaries, ...assumptions.map((assumption) => `Assumption: ${assumption.statement ?? "unspecified"}`)].join("\n"),
    intentDebt: assumptions.length,
    agedAssumptions: assumptions.filter((assumption) => assumption.createdAt && Date.now() - Date.parse(assumption.createdAt) > 7 * 24 * 60 * 60 * 1000),
    handoffQuality: summaries.length > 0 ? "usable" : "thin"
  };
}

export function clusterReviewFindings(input: { findings?: ReviewViolation[]; report?: ReviewReport } = {}) {
  const findings = input.findings ?? input.report?.violations ?? [];
  const clusters = new Map<string, ReviewViolation[]>();
  for (const finding of findings) {
    const key = clusterKey(finding);
    clusters.set(key, [...(clusters.get(key) ?? []), finding]);
  }
  const groups = [...clusters.entries()].map(([key, grouped]) => ({
    key,
    count: grouped.length,
    severity: grouped.some((finding) => finding.severity === "error") ? "error" as const : "warning" as const,
    rootCause: rootCauseFor(key),
    firstFix: grouped[0]?.recommendation ?? "Review this cluster.",
    sampleFindings: grouped.slice(0, 5)
  })).sort((left, right) => right.count - left.count);
  return {
    groups,
    fixOneThingFirst: groups[0]?.firstFix ?? "No findings to fix.",
    noiseRisk: findings.length > 30 ? "high" : findings.length > 10 ? "medium" : "low"
  };
}

export function generateLocalReportArtifact(input: { title?: string; report?: ReviewReport; findings?: ReviewViolation[]; format?: "markdown" | "json" } = {}) {
  const report = input.report ?? createReviewReport(input.findings ?? []);
  const title = input.title ?? "architect-mcp local report";
  const markdown = [
    `# ${title}`,
    "",
    `Gate: ${report.gate.status}`,
    `Score: ${report.score}`,
    `Errors: ${report.summary.errors}`,
    `Warnings: ${report.summary.warnings}`,
    "",
    "## Priority Findings",
    ...report.priorityFindings.map((finding) => `- ${finding.code}: ${finding.message}`)
  ].join("\n");
  return {
    format: input.format ?? "markdown",
    markdown,
    json: report
  };
}

export function analyzeRegressionCoverage(input: { patterns?: string[]; fixtureNames?: string[]; findings?: ReviewViolation[] } = {}) {
  const patterns = input.patterns ?? ["skipped verification", "broad rewrite", "missing evidence", "ui server leak", "rule overreach"];
  const fixtures = new Set(input.fixtureNames ?? []);
  return {
    patterns: patterns.map((pattern) => ({
      pattern,
      hasPositiveFixture: fixtures.has(`${slug(pattern)}-positive`),
      hasNegativeFixture: fixtures.has(`${slug(pattern)}-negative`),
      mappedFindings: (input.findings ?? []).filter((finding) => finding.message.toLowerCase().includes(pattern.split(" ")[0]))
    })),
    gaps: patterns.filter((pattern) => !fixtures.has(`${slug(pattern)}-positive`) || !fixtures.has(`${slug(pattern)}-negative`))
  };
}

export function reviewPatternCard(input: { card: { id?: string; summary?: string; pattern?: string; sensitivity?: string; tokens?: number } }) {
  const findings: string[] = [];
  if (!input.card.id) findings.push("Pattern card needs an id.");
  if (!input.card.summary || input.card.summary.length < 20) findings.push("Pattern card needs a useful summary.");
  if (/secret|token|password/i.test(`${input.card.summary} ${input.card.pattern}`)) findings.push("Pattern card may contain sensitive data.");
  if ((input.card.tokens ?? 0) > 800) findings.push("Pattern card is too large for routine handoff.");
  return { valid: findings.length === 0, findings };
}

export function previewPatternCard(input: { card: { id?: string; summary?: string; pattern?: string }; brief?: ProjectBrief }) {
  const text = `${input.brief?.idea ?? ""} ${input.card.summary ?? ""} ${input.card.pattern ?? ""}`;
  return {
    applies: Boolean(input.card.summary && text.length > 20),
    summary: input.card.summary ?? "No summary supplied.",
    conflicts: /hosted|github|database|persist/i.test(text) ? ["Pattern mentions deferred productization concepts; use only as advisory context."] : [],
    tokenSummary: (input.card.summary ?? "").slice(0, 240)
  };
}

function loadPolicyBundles(): PolicyBundle[] {
  return loadPolicyBundleFiles().bundles.map((entry) => entry.bundle);
}

function loadPolicyBundleFiles(): { directory: string; bundles: Array<{ file: string; content: string; bundle: PolicyBundle }> } {
  const dir = resolvePolicyBundleDirectory();
  const bundles = readdirSync(dir)
    .filter((file) => file.endsWith(".json"))
    .filter((file) => file !== "manifest.json")
    .sort()
    .map((file) => {
      const content = readFileSync(join(dir, file), "utf8");
      return {
        file,
        content,
        bundle: policyBundleSchema.parse(JSON.parse(content))
      };
    });
  return { directory: dir, bundles };
}

function validatePolicyBundleManifest(bundles: Array<{ file: string; content: string; bundle: PolicyBundle }>): string[] {
  const errors: string[] = [];
  const dir = resolvePolicyBundleDirectory();
  try {
    const manifest = JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8")) as {
      entries?: Array<{ id?: string; version?: string; sha256?: string }>;
    };
    const entries = new Map<string | undefined, { id?: string; version?: string; sha256?: string }>();
    for (const entry of manifest.entries ?? []) {
      if (entries.has(entry.id)) errors.push(`Duplicate policy-bundle manifest entry: ${entry.id ?? "missing-id"}`);
      entries.set(entry.id, entry);
    }
    const discoveredIds = new Set<string>();
    for (const { bundle, content } of bundles) {
      discoveredIds.add(bundle.id);
      const entry = entries.get(bundle.id);
      if (!entry) {
        errors.push(`${bundle.id}: missing policy-bundle manifest entry`);
        continue;
      }
      const hash = createHash("sha256").update(content).digest("hex");
      if (entry.version !== bundle.version) errors.push(`${bundle.id}: policy-bundle manifest version ${entry.version} does not match bundle version ${bundle.version}`);
      if (entry.sha256 !== hash) errors.push(`${bundle.id}: policy-bundle content changed without updating policy-bundles/manifest.json`);
    }
    for (const entry of manifest.entries ?? []) {
      if (entry.id && !discoveredIds.has(entry.id)) {
        errors.push(`${entry.id}: stale policy-bundle manifest entry has no matching bundle file`);
      }
    }
  } catch (error) {
    errors.push(`Could not validate policy-bundle manifest: ${error instanceof Error ? error.message : String(error)}`);
  }
  return errors;
}

function validatePolicyBundleManifest(): string[] {
  const errors: string[] = [];
  const dir = resolvePolicyBundleDirectory();
  try {
    const manifest = JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8")) as {
      entries?: Array<{ id?: string; version?: string; sha256?: string }>;
    };
    const entries = new Map((manifest.entries ?? []).map((entry) => [entry.id, entry]));
    const files = readdirSync(dir)
      .filter((file) => file.endsWith(".json"))
      .filter((file) => file !== "manifest.json")
      .sort();
    const seenBundleIds = new Set<string>();

    for (const file of files) {
      const content = readFileSync(join(dir, file), "utf8");
      const bundle = JSON.parse(content) as { id?: string; version?: string };
      if (bundle.id) seenBundleIds.add(bundle.id);
      const entry = entries.get(bundle.id);
      if (!entry) {
        errors.push(`${bundle.id ?? file}: missing policy-bundle manifest entry`);
        continue;
      }
      const hash = createHash("sha256").update(content).digest("hex");
      if (entry.version !== bundle.version) errors.push(`${bundle.id}: policy-bundle manifest version ${entry.version} does not match bundle version ${bundle.version}`);
      if (entry.sha256 !== hash) errors.push(`${bundle.id}: policy-bundle content changed without updating policy-bundles/manifest.json`);
    }

    for (const entry of manifest.entries ?? []) {
      if (entry.id && !seenBundleIds.has(entry.id)) {
        errors.push(`${entry.id}: stale policy-bundle manifest entry has no matching bundle file`);
      }
    }
  } catch (error) {
    errors.push(`Could not validate policy-bundle manifest: ${error instanceof Error ? error.message : String(error)}`);
  }

  return errors;
}

function resolvePolicyBundleDirectory(): string {
  const candidates = [
    resolve(process.cwd(), "policy-bundles"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../policy-bundles"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../../policy-bundles")
  ];
  for (const candidate of candidates) {
    try {
      readdirSync(candidate);
      return candidate;
    } catch {
      // Try the next candidate.
    }
  }
  throw new Error(`Could not find policy-bundles directory. Tried: ${candidates.join(", ")}`);
}

function recommendBundle(bundles: PolicyBundle[], brief?: ProjectBrief): PolicyBundle | undefined {
  const text = `${brief?.idea ?? ""} ${brief?.risk ?? ""} ${brief?.enforcement ?? ""} ${Object.values(brief?.stack ?? {}).join(" ")}`.toLowerCase();
  return bundles.find((bundle) => bundle.recommendedWhen.some((term) => text.includes(term))) ?? bundles.find((bundle) => bundle.profile === "balanced") ?? bundles[0];
}

function clusterKey(finding: ReviewViolation): string {
  if (/DB|DATABASE|ENV|CLIENT|SERVER|IMPORT/.test(finding.code)) return "boundary";
  if (/VERIFICATION|HARNESS|PLAN/.test(finding.code)) return "verification";
  if (/OVERSIZED|MIXED/.test(finding.code)) return "monolith";
  return finding.code;
}

function rootCauseFor(key: string): string {
  if (key === "boundary") return "Architecture boundary is unclear or crossed.";
  if (key === "verification") return "The implementation loop lacks proof or contract discipline.";
  if (key === "monolith") return "Responsibilities are concentrated into too few files.";
  return "Rule-specific issue.";
}

function slug(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}
