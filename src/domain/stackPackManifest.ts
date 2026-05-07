import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

export function validatePackManifest(packDirectory: string): string[] {
  const errors: string[] = [];
  const manifestPath = join(packDirectory, "manifest.json");

  try {
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as {
      entries?: Array<{ id?: string; version?: string; sha256?: string }>;
    };
    const entries = new Map<string | undefined, { id?: string; version?: string; sha256?: string }>();
    for (const entry of manifest.entries ?? []) {
      if (entries.has(entry.id)) errors.push(`Duplicate pack manifest entry: ${entry.id ?? "missing-id"}`);
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
        errors.push(`${pack.id ?? file}: missing pack manifest entry`);
        continue;
      }

      const hash = createHash("sha256").update(content).digest("hex");
      if (entry.version !== pack.version) errors.push(`${pack.id}: pack manifest version ${entry.version} does not match pack version ${pack.version}`);
      if (entry.sha256 !== hash) errors.push(`${pack.id}: pack content changed without updating packs/manifest.json; bump the pack version when rule behavior changes`);
    }

    for (const entry of manifest.entries ?? []) {
      if (entry.id && !discoveredPackIds.has(entry.id)) {
        errors.push(`${entry.id}: stale pack manifest entry has no matching pack file`);
      }
    }
  } catch (error) {
    errors.push(`Could not validate pack manifest: ${error instanceof Error ? error.message : String(error)}`);
  }

  return errors;
}
