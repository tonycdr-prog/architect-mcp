import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { StackPack } from "./types.js";

export function readPackManifest(packDirectory = resolve(process.cwd(), "packs")) {
  const manifestPath = resolve(packDirectory, "manifest.json");
  return JSON.parse(readFileSync(manifestPath, "utf8")) as {
    generatedBy?: string;
    entries?: Array<{ id: string; version: string; sha256: string }>;
  };
}

export function nextPackManifest(pack: StackPack, packContent: string, manifest = readPackManifest()) {
  const entry = {
    id: pack.id,
    version: pack.version,
    sha256: createHash("sha256").update(packContent).digest("hex")
  };
  const entries = (manifest.entries ?? []).filter((candidate) => candidate.id !== pack.id);
  entries.push(entry);
  entries.sort((left, right) => left.id.localeCompare(right.id));
  return {
    generatedBy: manifest.generatedBy ?? "architect-mcp",
    entries
  };
}

export function diffManifestEntries(before: Array<{ id: string; version: string; sha256: string }>, after: Array<{ id: string; version: string; sha256: string }>) {
  const beforeById = new Map(before.map((entry) => [entry.id, entry]));
  return after
    .filter((entry) => {
      const previous = beforeById.get(entry.id);
      return !previous || previous.version !== entry.version || previous.sha256 !== entry.sha256;
    })
    .map((entry) => {
      const previous = beforeById.get(entry.id);
      return {
        id: entry.id,
        beforeVersion: previous?.version,
        afterVersion: entry.version,
        hashChanged: previous?.sha256 !== entry.sha256
      };
    });
}

export function bumpVersion(version: string, bump: "patch" | "minor" | "major"): string {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) throw new Error(`Cannot bump invalid semver version: ${version}`);
  const major = Number(match[1]);
  const minor = Number(match[2]);
  const patch = Number(match[3]);
  if (bump === "major") return `${major + 1}.0.0`;
  if (bump === "minor") return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}
