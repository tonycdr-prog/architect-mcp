import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { createFinding } from "./findingMetadata.js";
import { extractEvidenceLines } from "./llmsEvidence.js";
import { fetchLlmsSource, listIngestedLlmsSources } from "./llmsSources.js";
import { candidateRulesForText, slugify, titleize } from "./stackPackCandidateRules.js";
import { analyzeStackPackConflicts } from "./stackPackConflicts.js";
import { bumpVersion, diffManifestEntries, nextPackManifest, readPackManifest } from "./stackPackPromotionFiles.js";
import { clearStackPackCache, listStackPacks } from "./stackPacks.js";
import type { IngestedLlmsSource, LlmsSourceSnapshot, ReviewViolation, StackPack, StackPackCandidate, StackPackCandidateInput, StackPackConflict } from "./types.js";

export { analyzeStackPackConflicts } from "./stackPackConflicts.js";

export function stackPackExpansionStrategy() {
  return {
    priority: ["nextjs", "react", "node-api", "postgres", "supabase", "expo-react-native"],
    ruleContract: [
      "Every pack needs concrete directories, file rules, module boundaries, tests, agent instructions, sources, and good/bad examples.",
      "Executable rules need triggerKind and detector metadata.",
      "Derived rules need source snapshot provenance, a concrete trigger condition, a detectable violation pattern, and actionable fix guidance.",
      "Overlapping stack packs must report duplicate rules and path/trigger collisions before promotion.",
      "Candidate rules are proposed from local snapshots or llms.txt-style source text, then reviewed before promotion.",
      "Live web fetching should stay outside promotion until source snapshots are reviewed."
    ]
  };
}

export function proposeStackPackRules(input: StackPackCandidateInput): StackPackCandidate {
  const stackName = input.stackName.trim();
  const normalized = slugify(stackName);
  const text = input.sourceText;
  const isFrontend = /react|next|vite|expo|native|ui|component|route|page/i.test(`${stackName} ${text}`);
  const isBackend = /api|server|route|controller|service|node|express|fastify|hono/i.test(`${stackName} ${text}`);
  const isDatabase = /postgres|database|schema|migration|query|sql|supabase|prisma|drizzle/i.test(`${stackName} ${text}`);
  const rules = candidateRulesForText(text, normalized);

  return {
    id: normalized,
    name: titleize(stackName),
    version: "0.1.0",
    rationale: `${titleize(stackName)} projects need explicit architecture rules so generated code follows documented boundaries instead of vague best-practice advice.`,
    sources: [{
      label: input.sourceLabel ?? `${titleize(stackName)} local source snapshot`,
      url: input.sourceUrl,
      note: "Candidate generated from caller-provided source text; review before promotion."
    }],
    appliesTo: [
      ...(isFrontend ? ["frontend" as const] : []),
      ...(isBackend ? ["backend" as const] : []),
      ...(isDatabase ? ["database" as const] : [])
    ].slice(0, 2),
    aliases: [stackName.toLowerCase()],
    directories: [
      ...(isFrontend ? [{ path: "src/features", purpose: "Feature-owned UI, hooks, and tests.", required: true }] : []),
      ...(isBackend ? [{ path: "src/server/services", purpose: "Use-cases and service orchestration.", required: true }] : []),
      ...(isDatabase ? [{ path: "src/db", purpose: "Schema, migrations, repositories, and seed data.", required: true }] : [])
    ],
    fileRules: rules,
    moduleBoundaries: [
      "Entry files compose modules; workflow logic belongs in feature, service, or database-owned modules.",
      "Shared code must not depend on feature code.",
      ...(isFrontend ? ["UI must not import database/server-only modules directly."] : []),
      ...(isBackend ? ["Routes/controllers call services/use-cases instead of owning workflows."] : []),
      ...(isDatabase ? ["Schema/migration changes require explicit repository/query ownership."] : [])
    ],
    testingExpectations: ["Add behavior-level tests for every generated workflow slice."],
    agentInstructions: ["Use this candidate as review input; do not promote without review findings passing."],
    confidence: rules.length >= 2 ? "medium" : "low",
    reviewNotes: ["Candidate rules are heuristic and require human review before pack promotion."]
  };
}

export async function ingestLlmsTxt(sourceIdOrUrl: string, options: { preferFull?: boolean; maxBytes?: number } = {}): Promise<{ snapshot: LlmsSourceSnapshot; candidate: StackPackCandidate }> {
  const snapshot = await fetchLlmsSource(sourceIdOrUrl, options);
  return {
    snapshot,
    candidate: proposeStackPackRules({
      stackName: snapshot.source.stack,
      sourceText: snapshot.content,
      sourceLabel: `${snapshot.source.stack} llms.txt snapshot`,
      sourceUrl: snapshot.source.url
    })
  };
}

export function deriveStackPackFromIngestedSource(sourceId: string): {
  source: IngestedLlmsSource;
  matchedEvidence: string[];
  candidate: StackPackCandidate;
  review: { valid: boolean; violations: ReviewViolation[]; conflicts: StackPackConflict[] };
} {
  const ingested = listIngestedLlmsSources().sources.find((source) => source.id === sourceId);
  if (!ingested) {
    throw new Error(`No ingested llms.txt source found for ${sourceId}. Run ingest:llms or list_ingested_llms_sources first.`);
  }
  if (ingested.status !== "ok" || !ingested.path) {
    throw new Error(`Ingested llms.txt source ${sourceId} is not usable: ${ingested.error ?? "missing local snapshot path"}.`);
  }

  const sourceText = readFileSync(resolve(process.cwd(), ingested.path), "utf8");
  const matchedEvidence = extractEvidenceLines(sourceText);
  const candidate = proposeStackPackRules({
    stackName: `${ingested.stack} LLMs Derived`,
    sourceText,
    sourceLabel: `${ingested.stack} ingested llms.txt snapshot`,
    sourceUrl: ingested.url
  });
  candidate.sources = [{
    label: `${ingested.stack} ingested llms.txt snapshot`,
    url: ingested.url,
    note: `Derived from local snapshot ${ingested.path} with sha256 ${ingested.sha256 ?? "unknown"} and evidence headings ${matchedEvidence.slice(0, 5).join("; ")}.`
  }];
  candidate.reviewNotes = [
    ...candidate.reviewNotes,
    `Derived from ${ingested.id} at ${ingested.fetchedAt}.`,
    `Matched evidence: ${matchedEvidence.slice(0, 8).join("; ")}.`
  ];

  return {
    source: ingested,
    matchedEvidence,
    candidate,
    review: reviewStackPackCandidate(candidate, { compareWithExisting: true })
  };
}

export function reviewStackPackCandidate(candidate: StackPackCandidate, options: { compareWithExisting?: boolean; allowExistingId?: boolean } = {}): { valid: boolean; violations: ReviewViolation[]; conflicts: StackPackConflict[] } {
  const violations: ReviewViolation[] = [];
  const knownIds = new Set(listStackPacks().map((pack) => pack.id));

  if (knownIds.has(candidate.id) && !options.allowExistingId) {
    violations.push(candidateFinding(`Candidate id already exists: ${candidate.id}.`, "Use diff_stack_pack_versions for existing packs or choose a new id."));
  }

  if (candidate.rationale.length < 80) {
    violations.push(candidateFinding("Candidate rationale is too vague.", "Explain the concrete failure mode this stack pack prevents."));
  }

  if (!candidate.sources.length || candidate.sources.some((source) => !source.url && !source.note)) {
    violations.push(candidateFinding("Candidate source metadata is incomplete.", "Every source needs a URL or reviewed local snapshot note."));
  }

  if (!candidate.fileRules.length) {
    violations.push(candidateFinding("Candidate has no file rules.", "Add enforceable file rules with triggers, detectors, and examples."));
  }

  for (const rule of candidate.fileRules) {
    if (isVagueRule(rule.rule) || isVagueRule(rule.trigger ?? "") || isVagueRule(rule.recommendation ?? "")) {
      violations.push(candidateFinding(`Rule ${rule.name} is too generic to enforce.`, "Replace vague best-practice language with a trigger condition, violation pattern, and specific remediation."));
    }
    if (!rule.trigger || !rule.recommendation || !rule.goodExample || !rule.badExample) {
      violations.push(candidateFinding(`Rule ${rule.name} is missing review metadata.`, "Rules need trigger, recommendation, goodExample, and badExample."));
    }
    if (!rule.triggerKind || !rule.detectors?.length) {
      violations.push(candidateFinding(`Rule ${rule.name} has no structured detector metadata.`, "Add triggerKind and detector descriptions."));
    }
    if (rule.triggerKind === "manual-review") {
      violations.push(candidateFinding(`Rule ${rule.name} is manual-only.`, "Derived packs need at least one concrete detector before promotion; keep manual-only rules as review notes."));
    }
  }

  const conflicts = options.compareWithExisting ? analyzeStackPackConflicts([...listStackPacks(), candidate]) : [];

  return {
    valid: violations.length === 0 && !conflicts.some((conflict) => conflict.severity === "error"),
    violations,
    conflicts
  };
}

export function promoteStackPackCandidate(candidate: StackPackCandidate, options: { allowExistingId?: boolean } = {}): { pack: StackPack; warnings: string[] } {
  const review = reviewStackPackCandidate(candidate, {
    allowExistingId: options.allowExistingId
  });
  if (!review.valid) {
    throw new Error(`Cannot promote stack pack candidate: ${review.violations.map((violation) => violation.message).join("; ")}`);
  }

  const { confidence: _confidence, reviewNotes: _reviewNotes, ...pack } = candidate;
  return {
    pack,
    warnings: ["Promotion is local-only: write this pack to packs/<id>.json, update packs/manifest.json, and run validate_stack_packs."]
  };
}

export function promoteStackPackCandidateToFiles(candidate: StackPackCandidate, options: { writeFiles?: boolean; allowOverwrite?: boolean; bump?: "none" | "patch" | "minor" | "major"; packDirectory?: string } = {}) {
  const requestedBump = options.bump ?? "none";
  const bumpedCandidate = requestedBump === "none" ? candidate : { ...candidate, version: bumpVersion(candidate.version, requestedBump) };
  const promotion = promoteStackPackCandidate(bumpedCandidate, {
    allowExistingId: true
  });
  const packDirectory = options.packDirectory ?? "packs";
  const packPath = `${packDirectory.replace(/\/$/, "")}/${promotion.pack.id}.json`;
  const manifestPath = `${packDirectory.replace(/\/$/, "")}/manifest.json`;
  const absolutePackDirectory = resolve(process.cwd(), packDirectory);
  const absolutePackPath = resolve(absolutePackDirectory, `${promotion.pack.id}.json`);
  const existingPack = packDirectory === "packs" ? listStackPacks().find((pack) => pack.id === promotion.pack.id) : undefined;
  const exists = existsSync(absolutePackPath);
  const versionChanged = existingPack ? existingPack.version !== promotion.pack.version : true;
  const packContent = `${JSON.stringify(promotion.pack, null, 2)}\n`;
  const manifestBefore = readPackManifest(absolutePackDirectory);
  const manifestAfter = nextPackManifest(promotion.pack, packContent, manifestBefore);
  const manifestContent = `${JSON.stringify(manifestAfter, null, 2)}\n`;
  const manifestDiff = diffManifestEntries(manifestBefore.entries ?? [], manifestAfter.entries);
  const readiness = {
    status: exists && !options.allowOverwrite ? "blocked" as const : "ready" as const,
    existingPack: exists,
    versionChanged,
    manifestChanges: manifestDiff,
    requiredNextChecks: ["validate_stack_packs", "npm test"]
  };
  const files = [
    { path: packPath, action: "write" as const, content: packContent },
    { path: manifestPath, action: "update" as const, content: manifestContent }
  ];
  const warnings = [
    ...promotion.warnings,
    "Dry-run by default. Set writeFiles=true to write packs/<id>.json and update packs/manifest.json."
  ];

  if (exists && !versionChanged && options.allowOverwrite) {
    warnings.push("Overwriting an existing pack without a version change. Prefer bump=patch, bump=minor, or bump=major.");
  }

  if (options.writeFiles) {
    if (existsSync(absolutePackPath) && !options.allowOverwrite) {
      throw new Error(`Refusing to overwrite ${packPath}. Set allowOverwrite=true to replace an existing pack.`);
    }
    writeFileSync(absolutePackPath, packContent, "utf8");
    writeFileSync(resolve(absolutePackDirectory, "manifest.json"), manifestContent, "utf8");
    clearStackPackCache();
    warnings.push("Wrote stack-pack files. Run validate_stack_packs before using the promoted pack.");
  }

  return {
    pack: promotion.pack,
    dryRun: !options.writeFiles,
    readiness,
    files,
    warnings
  };
}

export function diffStackPackVersions(before: StackPack, after: StackPack) {
  const changes = [
    ...diffArray("directory", before.directories.map((directory) => directory.path), after.directories.map((directory) => directory.path)),
    ...diffArray("file-rule", before.fileRules.map((rule) => rule.name), after.fileRules.map((rule) => rule.name)),
    ...diffArray("boundary", before.moduleBoundaries, after.moduleBoundaries),
    ...diffArray("test-expectation", before.testingExpectations, after.testingExpectations)
  ];

  return {
    breaking: changes.some((change) => change.kind.endsWith("removed") || change.kind === "file-rule-added"),
    changes
  };
}

function isVagueRule(value: string): boolean {
  const normalized = value.trim().toLowerCase();
  return normalized.length < 30 || /^(use|follow|apply)\s+(best practices|clean code|good architecture)/i.test(normalized);
}

function diffArray(kind: string, before: string[], after: string[]) {
  const beforeSet = new Set(before);
  const afterSet = new Set(after);
  return [
    ...after.filter((value) => !beforeSet.has(value)).map((value) => ({ kind: `${kind}-added`, value })),
    ...before.filter((value) => !afterSet.has(value)).map((value) => ({ kind: `${kind}-removed`, value }))
  ];
}

function candidateFinding(message: string, recommendation: string): ReviewViolation {
  return createFinding({
    code: "ARCH023_STACK_PACK_CANDIDATE",
    severity: "error",
    message,
    recommendation
  });
}
