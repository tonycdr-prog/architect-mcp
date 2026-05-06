import { execFileSync } from "node:child_process";

const stage = process.argv[2] ?? "v9";
if (!["v5", "v6", "v7", "v8", "v9", "v10"].includes(stage)) {
  throw new Error(`Unknown staged readiness target: ${stage}`);
}

execFileSync("npm", ["run", "check:v3"], { stdio: "inherit" });

if (stage === "v10") {
  const { runV10EvalHarness } = await import("../dist/domain/v10Productization.js") as typeof import("../dist/domain/v10Productization.js");
  const report = runV10EvalHarness();
  if (report.status !== "pass") {
    const failures = report.results
      .filter((result) => !result.passed)
      .map((result) => `${result.name}`)
      .join("; ");
    throw new Error(`V10 readiness failed: ${failures}`);
  }
  console.log(`V10 readiness passed: ${report.summary.passed}/${report.summary.total} staged evals.`);
  process.exit(0);
}

const { runV5V9EvalHarness } = await import("../dist/domain/v5V9EvalHarness.js") as typeof import("../dist/domain/v5V9EvalHarness.js");
const report = runV5V9EvalHarness(stage as "v5" | "v6" | "v7" | "v8" | "v9");
if (report.status !== "pass") {
  const failures = report.results
    .filter((result) => !result.passed)
    .map((result) => `${result.stage}/${result.name}: ${result.detail}`)
    .join("; ");
  throw new Error(`${stage.toUpperCase()} readiness failed: ${failures}`);
}

console.log(`${stage.toUpperCase()} readiness passed: ${report.summary.passed}/${report.summary.total} staged evals.`);
