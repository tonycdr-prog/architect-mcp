import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { hasGlobSyntax } from "./pathRules.js";
import { validatePackManifest } from "./stackPackManifest.js";
import { stackPackSchema, type RawStackPack } from "./stackPackSchemas.js";
import type { StackPack, StackProfile } from "./types.js";

let cachedStackPacks: StackPack[] | undefined;

export function listStackPacks(): StackPack[] {
  cachedStackPacks ??= loadStackPacks();
  return cachedStackPacks;
}

export function clearStackPackCache(): void {
  cachedStackPacks = undefined;
}

export function resolveStackPacks(stack: StackProfile, requestedPackIds: string[] = []): StackPack[] {
  const stackPacks = listStackPacks();
  const normalizedRequestedIds = new Set(requestedPackIds.map((id) => id.toLowerCase()));
  const knownIds = new Set(stackPacks.map((pack) => pack.id));
  const unknownIds = [...normalizedRequestedIds].filter((id) => !knownIds.has(id));
  if (unknownIds.length > 0) {
    throw new Error(`Unknown stack pack id${unknownIds.length === 1 ? "" : "s"}: ${unknownIds.join(", ")}`);
  }
  const stackValues = Object.values(stack)
    .filter(Boolean)
    .map((value) => value?.toLowerCase() ?? "");

  return stackPacks.filter((pack) => {
    if (normalizedRequestedIds.has(pack.id)) return true;
    if (pack.id === "mcp-server" && stackValues.some((value) => value.includes("mcp"))) return true;
    if (pack.id === "node-api") return matchesNodeApi(stackValues);

    const aliases = getAliases(pack);
    return stackValues.some((value) => aliases.some((alias) => value.includes(alias)));
  });
}

export function validateStackPacks(): { valid: boolean; errors: string[]; packCount: number } {
  const errors: string[] = [];

  try {
    const packs = loadStackPacks();
    errors.push(...validatePackManifest(resolvePackDirectory()));
    const ids = new Set<string>();
    const aliases = new Map<string, string>();
    for (const pack of packs) {
      if (ids.has(pack.id)) errors.push(`Duplicate pack id: ${pack.id}`);
      ids.add(pack.id);

      for (const alias of pack.aliases ?? []) {
        const normalizedAlias = alias.toLowerCase();
        const existingPackId = aliases.get(normalizedAlias);
        if (existingPackId && existingPackId !== pack.id) {
          errors.push(`Duplicate alias "${alias}" in ${existingPackId} and ${pack.id}`);
        }
        aliases.set(normalizedAlias, pack.id);
      }

      for (const rule of pack.fileRules) {
        if (!rule.trigger) errors.push(`${pack.id}/${rule.name}: missing trigger`);
        if (!rule.recommendation) errors.push(`${pack.id}/${rule.name}: missing recommendation`);
        if (!rule.appliesToPaths?.length) errors.push(`${pack.id}/${rule.name}: missing appliesToPaths`);
        if (!rule.goodExample || !rule.badExample) {
          errors.push(`${pack.id}/${rule.name}: missing goodExample or badExample`);
        }

        for (const pathPattern of rule.appliesToPaths ?? []) {
          if (!hasGlobSyntax(pathPattern)) {
            errors.push(`${pack.id}/${rule.name}: invalid path pattern ${pathPattern}`);
          }
        }

        const triggerKind = rule.triggerKind ?? inferTriggerKind(rule.name);
        if (triggerKind !== "manual-review" && !isSupportedTriggerKind(triggerKind)) {
          errors.push(`${pack.id}/${rule.name}: triggerKind ${triggerKind} is not mapped to an executable reviewer detector`);
        }

        if (!rule.detectors?.length) {
          errors.push(`${pack.id}/${rule.name}: missing structured detectors`);
        }
      }

      if (pack.rationale.trim().length < 80) errors.push(`${pack.id}: rationale must explain the concrete failure mode in at least 80 characters`);
      if (!pack.sources.length) errors.push(`${pack.id}: missing sources`);
      for (const source of pack.sources) {
        if (!source.url && !source.note) errors.push(`${pack.id}: source "${source.label}" needs either a URL or a note`);
        if (source.note && source.note.trim().length < 40) errors.push(`${pack.id}: source "${source.label}" note is too vague`);
        if (/best practices?/i.test(source.label) || /best practices?/i.test(source.note ?? "")) {
          errors.push(`${pack.id}: source "${source.label}" must be more specific than generic best practices`);
        }
      }
    }

    return { valid: errors.length === 0, errors, packCount: packs.length };
  } catch (error) {
    return {
      valid: false,
      errors: [error instanceof Error ? error.message : String(error)],
      packCount: 0
    };
  }
}

function isSupportedTriggerKind(triggerKind: string): boolean {
  return [
    "line-threshold",
    "import-boundary",
    "direct-db-access",
    "env-access",
    "client-boundary",
    "route-thinness",
    "migration-discipline",
    "hosted-filesystem",
    "auth-boundary",
    "payment-boundary",
    "test-policy",
    "validation-boundary",
    "ai-tool-safety"
  ].includes(triggerKind);
}

export function inferTriggerKind(ruleName: string): NonNullable<StackPack["fileRules"][number]["triggerKind"]> {
  const normalizedRuleName = ruleName.toLowerCase();
  if (normalizedRuleName.includes("thin route") || normalizedRuleName.includes("thin controller")) return "route-thinness";
  if (normalizedRuleName.includes("transport-neutral tools")) return "import-boundary";
  if (normalizedRuleName.includes("god component")) return "line-threshold";
  if (normalizedRuleName.includes("client boundaries") || normalizedRuleName.includes("supabase split")) return "client-boundary";
  if (normalizedRuleName.includes("ui queries")) return "direct-db-access";
  if (normalizedRuleName.includes("env")) return "env-access";
  if (normalizedRuleName.includes("migration discipline")) return "migration-discipline";
  if (normalizedRuleName.includes("hosted filesystem scanning")) return "hosted-filesystem";
  if (normalizedRuleName.includes("auth")) return "auth-boundary";
  if (normalizedRuleName.includes("payment") || normalizedRuleName.includes("stripe")) return "payment-boundary";
  if (normalizedRuleName.includes("test") || normalizedRuleName.includes("vitest")) return "test-policy";
  if (normalizedRuleName.includes("validation") || normalizedRuleName.includes("zod")) return "validation-boundary";
  if (normalizedRuleName.includes("ai tool") || normalizedRuleName.includes("model call")) return "ai-tool-safety";
  return "manual-review";
}

function loadStackPacks(): StackPack[] {
  const packDirectory = resolvePackDirectory();
  const packFiles = readdirSync(packDirectory)
    .filter((file) => file.endsWith(".json"))
    .filter((file) => file !== "manifest.json")
    .sort();

  return packFiles.map((file) => {
    const raw = JSON.parse(readFileSync(join(packDirectory, file), "utf8")) as unknown;
    const pack = stackPackSchema.parse(raw) as RawStackPack;
    return {
      ...pack,
      fileRules: pack.fileRules.map((rule) => ({
        ...rule,
        triggerKind: rule.triggerKind ?? inferTriggerKind(rule.name),
        detectors: rule.detectors ?? [{
          kind: rule.triggerKind ?? inferTriggerKind(rule.name),
          description: rule.trigger
        }]
      }))
    };
  });
}

function resolvePackDirectory(): string {
  const candidates = [
    resolve(process.cwd(), "packs"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../packs"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../../packs")
  ];

  for (const candidate of candidates) {
    try {
      readdirSync(candidate);
      return candidate;
    } catch {
      // Try the next candidate.
    }
  }

  throw new Error(`Could not find packs directory. Tried: ${candidates.join(", ")}`);
}

function getAliases(pack: StackPack): string[] {
  const packWithAliases = pack as StackPack & { aliases?: string[] };
  return [
    pack.id,
    pack.name.toLowerCase(),
    pack.name.toLowerCase().split(" ")[0] ?? pack.id,
    ...(packWithAliases.aliases ?? []).map((alias) => alias.toLowerCase())
  ];
}

function matchesNodeApi(stackValues: string[]): boolean {
  return stackValues.some((value) =>
    value.includes("node api") ||
    value.includes("express") ||
    value.includes("fastify") ||
    value.includes("hono")
  );
}
