import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

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

const packagedVitePressFiles = [...packedPaths].filter((path) => path.startsWith("docs/.vitepress/"));
if (packagedVitePressFiles.length > 0) {
  throw new Error(`Package dry-run includes VitePress build/config files: ${packagedVitePressFiles.join(", ")}`);
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
  const prepareScriptPath = fileURLToPath(new URL("./preparePackageManifest.cjs", import.meta.url));
  const restoreScriptPath = fileURLToPath(new URL("./restorePackageManifest.cjs", import.meta.url));
  const manifestPath = fileURLToPath(new URL("../package.json", import.meta.url));
  let manifestPrepared = false;
  let readFailure: Error | null = null;
  try {
    try {
      execFileSync(process.execPath, [prepareScriptPath], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"]
      });
      manifestPrepared = true;
    } catch (error) {
      throw new Error(
        `Failed to prepare package manifest for V3 readiness check: ${describeExecFailure(error)}`
      );
    }
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as unknown;
    if (!manifest || typeof manifest !== "object") {
      throw new Error("Prepared package manifest is not a JSON object.");
    }
    return manifest as { scripts?: Record<string, string> };
  } catch (error) {
    readFailure = error instanceof Error ? error : new Error(String(error));
    throw readFailure;
  } finally {
    if (manifestPrepared) {
      try {
        execFileSync(process.execPath, [restoreScriptPath], {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "pipe"]
        });
      } catch (error) {
        const message = `Failed to restore package manifest after V3 readiness check: ${describeExecFailure(error)}`;
        if (readFailure) {
          console.error(message);
        } else {
          throw new Error(message);
        }
      }
    }
  }
}

function describeExecFailure(error: unknown): string {
  if (!error || typeof error !== "object") {
    return String(error);
  }
  const stderr = String(Reflect.get(error, "stderr") ?? "").trim();
  if (stderr.length > 0) {
    return stderr;
  }
  return error instanceof Error ? error.message : String(error);
}
