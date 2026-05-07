import { readdirSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";
import type { FoundationPack } from "./types.js";

const foundationPackSchema = z.object({
  id: z.enum(["agent-harness", "testing", "repo-structure", "ci-gates", "repo-hygiene"]),
  name: z.string().min(1),
  version: z.string().regex(/^\d+\.\d+\.\d+$/),
  rationale: z.string().min(40),
  sources: z.array(z.object({
    label: z.string().min(1),
    url: z.string().url().optional(),
    note: z.string().min(30).optional()
  })).min(1),
  rules: z.array(z.string().min(1)).min(1),
  artifacts: z.array(z.string().min(1)).min(1),
  reviewQuestions: z.array(z.string().min(1)).min(1)
});

let cachedFoundationPacks: FoundationPack[] | undefined;

export function listFoundationPacks(): FoundationPack[] {
  cachedFoundationPacks ??= loadFoundationPacks();
  return cachedFoundationPacks;
}

export function validateFoundationPacks(): { valid: boolean; errors: string[]; packCount: number } {
  const errors: string[] = [];

  try {
    const packs = loadFoundationPacks();
    errors.push(...validateFoundationPackManifest());
    const ids = new Set<string>();
    for (const pack of packs) {
      if (ids.has(pack.id)) errors.push(`Duplicate foundation pack id: ${pack.id}`);
      ids.add(pack.id);
      if (pack.rules.length < 2) errors.push(`${pack.id}: expected at least two concrete rules`);
      if (pack.artifacts.length === 0) errors.push(`${pack.id}: expected at least one artifact`);
    }
  } catch (error) {
    errors.push(error instanceof Error ? error.message : String(error));
  }

  return {
    valid: errors.length === 0,
    errors,
    packCount: errors.length === 0 ? loadFoundationPacks().length : 0
  };
}

function validateFoundationPackManifest(): string[] {
  const errors: string[] = [];
  const packDirectory = resolveFoundationPackDirectory();
  const manifestPath = join(packDirectory, "manifest.json");

  try {
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as {
      entries?: Array<{ id?: string; version?: string; sha256?: string }>;
    };
    const entries = new Map<string | undefined, { id?: string; version?: string; sha256?: string }>();
    for (const entry of manifest.entries ?? []) {
      if (entries.has(entry.id)) errors.push(`Duplicate foundation-pack manifest entry: ${entry.id ?? "missing-id"}`);
      entries.set(entry.id, entry);
    }
    const packFiles = readdirSync(packDirectory)
      .filter((file) => file.endsWith(".json"))
      .filter((file) => file !== "manifest.json")
      .sort();
    const discoveredPackIds = new Set<string>();

    for (const file of packFiles) {
      const content = readFileSync(join(packDirectory, file), "utf8");
      const pack = JSON.parse(content) as { id?: string; version?: string };
      if (pack.id) discoveredPackIds.add(pack.id);
      const entry = entries.get(pack.id);
      if (!entry) {
        errors.push(`${pack.id ?? file}: missing foundation-pack manifest entry`);
        continue;
      }

      const hash = createHash("sha256").update(content).digest("hex");
      if (entry.version !== pack.version) errors.push(`${pack.id}: foundation-pack manifest version ${entry.version} does not match pack version ${pack.version}`);
      if (entry.sha256 !== hash) errors.push(`${pack.id}: foundation-pack content changed without updating foundation-packs/manifest.json`);
    }

    for (const entry of manifest.entries ?? []) {
      if (entry.id && !discoveredPackIds.has(entry.id)) {
        errors.push(`${entry.id}: stale foundation-pack manifest entry has no matching pack file`);
      }
    }
  } catch (error) {
    errors.push(`Could not validate foundation-pack manifest: ${error instanceof Error ? error.message : String(error)}`);
  }

  return errors;
}

function loadFoundationPacks(): FoundationPack[] {
  const packDirectory = resolveFoundationPackDirectory();
  return readdirSync(packDirectory)
    .filter((file) => file.endsWith(".json"))
    .filter((file) => file !== "manifest.json")
    .sort()
    .map((file) => foundationPackSchema.parse(JSON.parse(readFileSync(join(packDirectory, file), "utf8"))) as FoundationPack);
}

function resolveFoundationPackDirectory(): string {
  const candidates = [
    resolve(process.cwd(), "foundation-packs"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../foundation-packs"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../../foundation-packs")
  ];

  for (const candidate of candidates) {
    try {
      readdirSync(candidate);
      return candidate;
    } catch {
      // Try the next candidate.
    }
  }

  throw new Error(`Could not find foundation-packs directory. Tried: ${candidates.join(", ")}`);
}
