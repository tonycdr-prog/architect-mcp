const { copyFileSync, readFileSync, writeFileSync } = require("node:fs");

const packagePath = "package.json";
const backupPath = "package.json.prepack-backup";
const repoOnlyScripts = new Set([
  "typecheck",
  "test",
  "audit",
  "pack:dry-run",
  "check:v3",
  "check:v5",
  "check:v6",
  "check:v7",
  "check:v8",
  "check:v9",
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
