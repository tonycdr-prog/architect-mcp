import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

type Step = {
  name: string;
  command: string;
  args: string[];
};

const steps: Step[] = [
  { name: "typecheck", command: "npm", args: ["run", "typecheck"] },
  { name: "test", command: "npm", args: ["test"] },
  { name: "build", command: "npm", args: ["run", "build"] },
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
  "LICENSE",
  "SECURITY.md",
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

for (const docsPath of ["README.md", "AGENTS.md"]) {
  const content = readFileSync(docsPath, "utf8");
  if (/\/Users\/tonycordner\//.test(content)) {
    throw new Error(`${docsPath} contains a maintainer-local /Users/tonycordner path.`);
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
