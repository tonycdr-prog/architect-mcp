import { createFinding } from "./findingMetadata.js";
import { isGeneratedFile, isSourceCodeFile, matchesPathPattern } from "./pathRules.js";
import type { BuildPlan, FileSummary, ReviewViolation } from "./types.js";

export function reviewImplementationPlanDrift(files: Array<FileSummary & { path: string }>, filePaths: string[], buildPlan?: BuildPlan): ReviewViolation[] {
  const violations: ReviewViolation[] = [];
  const hasMonolithProneImplementation = files.some((file) => /(^|\/)(App|app|page|index|server|main)\.(tsx|ts|jsx|js)$/.test(file.path) || /^src\/features\//.test(file.path));
  const hasFeatureImplementation = files.some((file) => /^src\/features\//.test(file.path));
  const hasTests = filePaths.some((path) => /(^|\/)(tests|__tests__)\/|\.test\.(ts|tsx|js|jsx)$/.test(path));
  const hasHarness = filePaths.includes("AGENTS.md") && filePaths.includes("docs/architecture-contract.md");
  const allowedDirectories = new Set(buildPlan?.slices.flatMap((slice) => slice.allowedDirectories).map(stripGlob));
  const forbiddenFiles = buildPlan?.slices.flatMap((slice) => slice.forbiddenFiles) ?? [];
  const plannedOutputs = buildPlan?.slices.flatMap((slice) => slice.outputs) ?? [];

  if (hasMonolithProneImplementation && !hasHarness) {
    violations.push(createFinding({
      code: "ARCH020_IMPLEMENTATION_IGNORED_PLAN",
      severity: "error",
      message: "Implementation files exist before required agent harness artifacts.",
      recommendation: "Generate AGENTS.md and docs/architecture-contract.md before continuing implementation."
    }));
  }
  if (hasFeatureImplementation && !hasTests) {
    violations.push(createFinding({
      code: "ARCH020_IMPLEMENTATION_IGNORED_PLAN",
      severity: "warning",
      message: "Feature implementation exists without any visible tests.",
      recommendation: "Add behavior-level tests for the generated feature slice before adding another slice."
    }));
  }

  for (const file of files) {
    if (!/(^|\/)(App|app|index|server|main)\.(tsx|ts|jsx|js)$/.test(file.path)) continue;
    if (/^src\/app\/App\.(tsx|ts|jsx|js)$/.test(file.path)) continue;
    if ((file.lines ?? 0) > 220 || (file.imports ?? []).some((specifier) => /db|database|repository|service|auth|features/.test(specifier))) {
      violations.push(createFinding({
        code: "ARCH020_IMPLEMENTATION_IGNORED_PLAN",
        severity: "error",
        path: file.path,
        message: "App entry file appears to own feature, service, auth, or database implementation.",
        recommendation: "Keep entry files for composition and move workflow logic into feature/service/database-owned modules."
      }));
    }
  }

  if (!buildPlan) return violations;
  for (const file of files.filter((candidate) => isSourceCodeFile(candidate.path) && !isGeneratedFile(candidate.path))) {
    if (allowedDirectories.size > 0 && ![...allowedDirectories].some((directory) => file.path === directory || file.path.startsWith(`${directory}/`))) {
      violations.push(createFinding({
        code: "ARCH020_IMPLEMENTATION_IGNORED_PLAN",
        severity: "error",
        path: file.path,
        message: "File is outside the directories allowed by the build plan.",
        recommendation: "Move the file into an allowed slice directory or update and re-review the build plan before implementation."
      }));
    }
    if (forbiddenFiles.some((pattern) => matchesPlanPattern(file.path, pattern))) {
      violations.push(createFinding({
        code: "ARCH020_IMPLEMENTATION_IGNORED_PLAN",
        severity: "error",
        path: file.path,
        message: "File matches a forbidden path in the build plan.",
        recommendation: "Keep app entry files for composition and place implementation in feature/service-owned modules."
      }));
    }
  }

  for (const output of plannedOutputs.filter((candidate) => looksLikePath(candidate))) {
    if (!filePaths.some((path) => path === output || path.startsWith(`${output}/`))) {
      violations.push(createFinding({
        code: "ARCH020_IMPLEMENTATION_IGNORED_PLAN",
        severity: "warning",
        path: output,
        message: "Planned build output is missing from the reviewed file tree.",
        recommendation: "Complete the current build-plan slice before starting later implementation work."
      }));
    }
  }
  return violations;
}

function stripGlob(value: string): string {
  return value.replace(/\/?\*\*.*$/, "").replace(/\/?\*.*$/, "");
}

function matchesPlanPattern(path: string, pattern: string): boolean {
  return matchesPathPattern(path, pattern);
}

function looksLikePath(value: string): boolean {
  return /^(src|tests|docs|server|client|shared|supabase)\//.test(value) || /^\.[\w/-]+/.test(value) || /^[A-Z]+\.md$/.test(value);
}
