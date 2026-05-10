import { execFileSync } from "node:child_process";
import { readFileSync, rmSync } from "node:fs";

type Step = {
  name: string;
  command: string;
  args: string[];
};

const steps: Step[] = [
  { name: "typecheck", command: "npm", args: ["run", "typecheck"] },
  { name: "test", command: "npm", args: ["test"] },
  { name: "build", command: "npm", args: ["run", "build"] },
  { name: "docs build", command: "npm", args: ["run", "docs:build"] },
  { name: "audit", command: "npm", args: ["audit"] }
];

for (const step of steps) {
  run(step);
}

const packOutput = execFileSync("npm", ["pack", "--dry-run", "--json"], {
  encoding: "utf8",
  stdio: ["ignore", "pipe", "inherit"]
});
const packed = JSON.parse(packOutput) as Array<{ files?: Array<{ path: string }> }>;
const packedPaths = new Set(packed[0]?.files?.map((file) => file.path) ?? []);
const requiredPackageFiles = [
  "docs/use-on-a-repo.md",
  "docs/hosted-api-shape.md",
  "examples/client-wrapper.ts",
  "packs/hono.json",
  "packs/stripe.json",
  "packs/auth0.json",
  "packs/expo.json",
  "packs/ai-sdk.json",
  "policy-bundles/balanced.json"
];
const missingPackageFiles = requiredPackageFiles.filter((path) => !packedPaths.has(path));
if (missingPackageFiles.length > 0) {
  throw new Error(`Package dry-run is missing required V3 files: ${missingPackageFiles.join(", ")}`);
}

const repoOnlyPackageFiles = [
  "scripts/checkV3Readiness.ts",
  "scripts/checkStagedReadiness.ts",
  "scripts/ingestLlmsSources.ts"
];
const packagedRepoOnlyFiles = repoOnlyPackageFiles.filter((path) => packedPaths.has(path));
if (packagedRepoOnlyFiles.length > 0) {
  throw new Error(`Package dry-run includes repo-only TypeScript scripts: ${packagedRepoOnlyFiles.join(", ")}`);
}

const packedManifest = readPackedManifest();
const repoOnlyPackageScripts = [
  "typecheck",
  "test",
  "audit",
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
  "docs:get",
  "docs:build",
  "docs:preview",
  "ingest:llms",
  "dev",
  "dev:http",
  "prepack",
  "postpack"
];
const leakedPackageScripts = repoOnlyPackageScripts.filter((script) => Boolean(packedManifest.scripts?.[script]));
if (leakedPackageScripts.length > 0) {
  throw new Error(`Packed package.json includes repo-only npm scripts: ${leakedPackageScripts.join(", ")}`);
}

const maintainerLocalPathPattern = new RegExp(["", "Users", "tonycordner", ""].join("\\/"));
const trackedFiles = execFileSync("git", ["ls-files"], {
  encoding: "utf8",
  stdio: ["ignore", "pipe", "inherit"]
})
  .split("\n")
  .filter(Boolean)
  .filter((path) => !path.startsWith("node_modules/") && !path.startsWith("dist/") && !path.endsWith(".snap"));
for (const trackedFile of trackedFiles) {
  const content = readFileSync(trackedFile, "utf8");
  if (maintainerLocalPathPattern.test(content)) {
    throw new Error(`${trackedFile} contains a maintainer-local absolute path.`);
  }
}

const { createMcpReadinessReport } = await import("../dist/domain/readinessReport.js") as typeof import("../dist/domain/readinessReport.js");
const readiness = await createMcpReadinessReport();
if (!readiness.ready) {
  const failures = readiness.checks
    .filter((check) => check.status === "fail")
    .map((check) => `${check.name}: ${check.summary}`)
    .join("; ");
  throw new Error(`V3 readiness failed: ${failures}`);
}

console.log(`V3 readiness passed: ${readiness.checks.length} checks, ${packedPaths.size} package files.`);

function run(step: Step): void {
  execFileSync(step.command, step.args, {
    stdio: "inherit"
  });
}

function readPackedManifest(): { scripts?: Record<string, string> } {
  const packOutput = execFileSync("npm", ["pack", "--json"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"]
  });
  const packedArtifacts = JSON.parse(packOutput) as Array<{ filename?: string }>;
  const filename = packedArtifacts[0]?.filename;
  if (!filename) throw new Error("npm pack did not return a package filename.");

  try {
    return JSON.parse(execFileSync("tar", ["-xOf", filename, "package/package.json"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"]
    })) as { scripts?: Record<string, string> };
  } finally {
    rmSync(filename, { force: true });
  }
}
