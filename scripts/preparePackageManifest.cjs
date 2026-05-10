const { copyFileSync, readFileSync, writeFileSync } = require("node:fs");

const packagePath = "package.json";
const backupPath = "package.json.prepack-backup";
const repoOnlyScripts = new Set([
  "typecheck",
  "test",
  "audit",
  "docs:get",
  "docs:tool-reference",
  "docs:tool-reference:check",
  "docs:build",
  "docs:preview",
  "pack:dry-run",
  "precheck:v3",
  "check:v3",
  "precheck:v5",
  "check:v5",
  "precheck:v6",
  "check:v6",
  "precheck:v7",
  "check:v7",
  "precheck:v8",
  "check:v8",
  "precheck:v9",
  "check:v9",
  "precheck:v10",
  "check:v10",
  "release:check",
  "ingest:llms",
  "dev",
  "dev:http",
  "prepack",
  "postpack"
]);

copyFileSync(packagePath, backupPath);

const manifest = JSON.parse(readFileSync(packagePath, "utf8"));
manifest.scripts = Object.fromEntries(
  Object.entries(manifest.scripts ?? {}).filter(([name]) => !repoOnlyScripts.has(name))
);

writeFileSync(packagePath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
