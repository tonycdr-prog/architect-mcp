import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { generateContract } from "../src/domain/contract.js";
import { grillMe } from "../src/domain/intake.js";
import { generateBuildPlan } from "../src/domain/buildPlan.js";
import { reviewBuildPlan } from "../src/domain/buildPlanReviewer.js";
import { discoverLlmsSources, listIngestedLlmsSources } from "../src/domain/llmsSources.js";
import { inferRepoLayoutFromFiles } from "../src/domain/repoLayout.js";
import { createReviewReport } from "../src/domain/reviewReport.js";
import { reviewFileSummaries } from "../src/domain/reviewer.js";
import { reviewProposedFilePlan } from "../src/domain/planReviewer.js";
import { analyzeStackPackConflicts, deriveStackPackFromIngestedSource, diffStackPackVersions, promoteStackPackCandidate, proposeStackPackRules, reviewStackPackCandidate, stackPackExpansionStrategy } from "../src/domain/stackPackWorkflow.js";
import { resolveStackPacks, validateStackPacks } from "../src/domain/stackPacks.js";
import { scanWorkspace } from "../src/infrastructure/scanWorkspace.js";
import {
  generatedAppBadFixture,
  generatedAppGoodFixture,
  messyReactFixture,
  nextjsGeneratedBadFixture,
  nextjsGeneratedGoodFixture,
  reactViteGeneratedBadFixture,
  reactViteGeneratedGoodFixture
} from "./fixtures/repos.js";

describe("reviewFileSummaries", () => {
  it("ignores generated lockfiles for line-count warnings", () => {
    const violations = reviewFileSummaries([
      {
        path: "package-lock.json",
        lines: 1800
      }
    ]);

    assert.equal(violations.length, 0);
  });

  it("flags oversized source files", () => {
    const violations = reviewFileSummaries([
      {
        path: "src/domain/largeModule.ts",
        lines: 450
      }
    ]);

    assert.equal(violations.length, 1);
    assert.equal(violations[0]?.severity, "warning");
    assert.equal(violations[0]?.code, "ARCH001_OVERSIZED_FILE");
    assert.equal(violations[0]?.confidence, "medium");
  });

  it("uses file-category line thresholds", () => {
    const violations = reviewFileSummaries([
      {
        path: "client/screens/DashboardScreen.tsx",
        lines: 420
      },
      {
        path: "server/routes/users.ts",
        lines: 360
      },
      {
        path: "shared/schema.ts",
        lines: 900
      }
    ]);

    assert.deepEqual(violations.map((violation) => violation.path), ["server/routes/users.ts"]);
  });

  it("flags oversized type aggregation files", () => {
    const violations = reviewFileSummaries([
      {
        path: "src/domain/types.ts",
        lines: 590
      }
    ]);

    assert.equal(violations.some((violation) => violation.code === "ARCH025_TYPE_SCHEMA_AGGREGATION"), true);
    assert.equal(violations.find((violation) => violation.code === "ARCH025_TYPE_SCHEMA_AGGREGATION")?.recommendation.includes("barrel export"), true);
  });

  it("flags oversized schema aggregation files", () => {
    const violations = reviewFileSummaries([
      {
        path: "src/tools/schemas.ts",
        lines: 466
      }
    ]);

    assert.equal(violations.some((violation) => violation.code === "ARCH025_TYPE_SCHEMA_AGGREGATION"), true);
  });

  it("flags source files dumped at the repository root", () => {
    const violations = reviewFileSummaries([
      {
        path: "server.ts",
        lines: 80
      },
      {
        path: "vite.config.ts",
        lines: 80
      }
    ]);

    assert.equal(violations.some((violation) => violation.code === "ARCH026_REPO_HYGIENE" && violation.path === "server.ts"), true);
    assert.equal(violations.some((violation) => violation.path === "vite.config.ts"), false);
  });

  it("flags shared modules that import feature modules", () => {
    const violations = reviewFileSummaries([
      {
        path: "src/shared/ui/Button.tsx",
        imports: ["@/features/billing/useBilling"]
      }
    ]);

    assert.equal(violations.length, 1);
    assert.equal(violations[0]?.severity, "error");
  });

  it("flags client components that import server modules", () => {
    const violations = reviewFileSummaries([
      {
        path: "src/features/billing/BillingClient.tsx",
        hasUseClient: true,
        imports: ["@/server/billing"]
      }
    ]);

    assert.equal(violations.length, 1);
    assert.equal(violations[0]?.severity, "error");
  });

  it("flags UI files with direct database access", () => {
    const violations = reviewFileSummaries([
      {
        path: "src/features/users/UserTable.tsx",
        hasDirectDbAccess: true
      }
    ]);

    assert.equal(violations.length, 1);
    assert.equal(violations[0]?.severity, "error");
  });

  it("allows common public client env vars in UI files", () => {
    const violations = reviewFileSummaries([
      {
        path: "client/components/AppIcon.tsx",
        envAccesses: ["NODE_ENV", "EXPO_PUBLIC_SHOW_UI_DEBUG_ERRORS"]
      }
    ]);

    assert.equal(violations.length, 0);
  });

  it("does not treat feedback in component paths as database logic", () => {
    const violations = reviewFileSummaries([
      {
        path: "docs-site/components/DocsFeedbackForm.tsx"
      }
    ]);

    assert.equal(violations.length, 0);
  });

  it("does not scan generated files for env access warnings", () => {
    const violations = reviewFileSummaries([
      {
        path: "package-lock.json",
        envAccesses: ["PORT"]
      },
      {
        path: "dist/http.js",
        envAccesses: ["HOST"]
      }
    ]);

    assert.equal(violations.length, 0);
  });

  it("ignores generated native, media, and test-output artifacts", () => {
    const violations = reviewFileSummaries([
      {
        path: "ios/Pods/SomeGeneratedFile.swift",
        lines: 4000
      },
      {
        path: "test-output/admin-dashboard.png",
        lines: 1200
      },
      {
        path: "docs-site/out/assets/screenshot.png",
        lines: 1200
      },
      {
        path: "assets/logo.png",
        lines: 1200
      }
    ]);

    assert.equal(violations.length, 0);
  });

  it("does not scan pack examples for env access warnings", () => {
    const violations = reviewFileSummaries([
      {
        path: "packs/node-api.json",
        envAccesses: ["API_KEY"]
      },
      {
        path: "src/http.ts",
        envAccesses: ["PORT"]
      }
    ]);

    assert.equal(violations.length, 0);
  });

  it("executes supported pack file rules from the architecture contract", () => {
    const contract = generateContract(
      {
        idea: "A Next.js app",
        stack: {
          frontend: "Next.js"
        }
      },
      ["nextjs"]
    );

    const violations = reviewFileSummaries(
      [
        {
          path: "src/app/dashboard/page.tsx",
          lines: 220
        }
      ],
      contract
    );

    assert.equal(violations.some((violation) => violation.message.includes("Thin route files")), true);
  });

  it("uses directory paths when checking required contract directories", () => {
    const contract = generateContract({
      idea: "A Supabase app",
      stack: {
        database: "Supabase"
      },
      repoLayout: {
        pathMap: {
          "src/db": ["supabase"],
          "src/db/supabase": ["supabase"],
          "src/domain": ["server/lib"],
          "src/infrastructure": ["server/lib"],
          "tests": ["server"]
        }
      }
    });

    const violations = reviewFileSummaries([], contract, 300, ["supabase", "server", "server/lib", "docs"]);

    assert.equal(violations.some((violation) => violation.path === "supabase"), false);
  });

  it("flags transitive UI to database imports and import cycles", () => {
    const violations = reviewFileSummaries([
      { path: "src/features/billing/BillingPage.tsx", imports: ["@/features/billing/useBilling"] },
      { path: "src/features/billing/useBilling.ts", imports: ["@/db/billingRepository"] },
      { path: "src/db/billingRepository.ts" },
      { path: "src/shared/a.ts", imports: ["@/shared/b"] },
      { path: "src/shared/b.ts", imports: ["@/shared/a"] }
    ]);

    assert.equal(violations.some((violation) => violation.code === "ARCH022_IMPORT_GRAPH_BOUNDARY" && violation.severity === "error"), true);
    assert.equal(violations.some((violation) => violation.code === "ARCH022_IMPORT_GRAPH_BOUNDARY" && violation.severity === "warning"), true);
  });
});

describe("reviewProposedFilePlan", () => {
  it("flags monolithic proposed app plans before files are written", () => {
    const violations = reviewProposedFilePlan({
      files: [
        {
          path: "src/App.tsx",
          purpose: "Own routing, fetching, auth, database writes, state, validation, and UI.",
          responsibilities: ["routing", "fetching", "auth", "database", "state", "ui"]
        }
      ]
    });

    assert.equal(violations.some((violation) => violation.code === "ARCH015_PLAN_MONOLITH_RISK"), true);
    assert.equal(violations.some((violation) => violation.code === "ARCH017_PLAN_MISSING_HARNESS"), true);
  });

  it("flags proposed database files without a server-owned data boundary", () => {
    const violations = reviewProposedFilePlan({
      files: [
        { path: "AGENTS.md", purpose: "Agent instructions." },
        { path: "docs/architecture-contract.md", purpose: "Architecture contract." },
        { path: "src/db/schema.ts", purpose: "Database schema.", responsibilities: ["define tables"] }
      ]
    });

    assert.equal(violations.some((violation) => violation.code === "ARCH016_PLAN_MISSING_BOUNDARY"), true);
  });

  it("allows a boundary-aware proposed plan", () => {
    const violations = reviewProposedFilePlan({
      files: [
        { path: "AGENTS.md", purpose: "Agent instructions." },
        { path: "docs/architecture-contract.md", purpose: "Architecture contract." },
        { path: "src/features/billing/BillingPage.tsx", purpose: "Feature view.", responsibilities: ["render billing page"] },
        { path: "src/features/billing/useBilling.ts", purpose: "Feature data hook.", responsibilities: ["load billing state"] },
        { path: "src/server/services/billingService.ts", purpose: "Billing use case.", responsibilities: ["orchestrate billing"] }
      ]
    });

    assert.equal(violations.length, 0);
  });
});

describe("reviewBuildPlan", () => {
  it("flags plans that skip harness setup or use vague checks", () => {
    const plan = generateBuildPlan(messyReactFixture.brief);
    const violations = reviewBuildPlan({
      ...plan,
      slices: plan.slices.filter((slice) => slice.id !== "agent-harness")
    });

    assert.equal(violations.some((violation) => violation.code === "ARCH018_BUILD_PLAN_ORDER"), true);
  });

  it("rejects build-plan checks outside the brief verification set", () => {
    const plan = generateBuildPlan({
      ...messyReactFixture.brief,
      verification: ["npm test"]
    });
    const violations = reviewBuildPlan({
      ...plan,
      slices: plan.slices.map((slice) => slice.id === "frontend-boundary"
        ? { ...slice, checks: ["npm run made-up-check"] }
        : slice)
    }, {
      allowedChecks: ["npm test"]
    });

    assert.equal(violations.some((violation) => violation.code === "ARCH019_BUILD_PLAN_VERIFICATION" && violation.severity === "error"), true);
  });

  it("flags implementation output that ignored the harness plan", () => {
    const violations = reviewFileSummaries([
      { path: "src/App.tsx", lines: 260, imports: ["@/db/client", "@/features/customers"] },
      { path: "src/features/customers/CustomerPage.tsx", lines: 120 },
      { path: "src/features/customers/customerState.ts", lines: 80 }
    ]);

    assert.equal(violations.some((violation) => violation.code === "ARCH020_IMPLEMENTATION_IGNORED_PLAN"), true);
  });

  it("compares generated app output against the build plan", () => {
    const buildPlan = generateBuildPlan(generatedAppBadFixture.brief);
    const badViolations = reviewFileSummaries(generatedAppBadFixture.files, undefined, 300, generatedAppBadFixture.directories, buildPlan);
    const goodViolations = reviewFileSummaries(generatedAppGoodFixture.files, undefined, 300, generatedAppGoodFixture.directories, buildPlan);

    assert.equal(badViolations.some((violation) => violation.code === "ARCH020_IMPLEMENTATION_IGNORED_PLAN"), true);
    assert.equal(goodViolations.some((violation) => violation.code === "ARCH020_IMPLEMENTATION_IGNORED_PLAN" && violation.severity === "error"), false);
  });

  it("covers Next.js generated-app stack fixtures", () => {
    const badViolations = reviewFileSummaries(nextjsGeneratedBadFixture.files, generateContract(nextjsGeneratedBadFixture.brief, ["nextjs"]), 300, nextjsGeneratedBadFixture.directories);
    const goodViolations = reviewFileSummaries(nextjsGeneratedGoodFixture.files, generateContract(nextjsGeneratedGoodFixture.brief, ["nextjs"]), 300, nextjsGeneratedGoodFixture.directories);

    assert.equal(badViolations.some((violation) => violation.severity === "error"), true);
    assert.equal(goodViolations.some((violation) => violation.severity === "error"), false);
  });

  it("covers React/Vite generated-app stack fixtures", () => {
    const badViolations = reviewFileSummaries(reactViteGeneratedBadFixture.files, generateContract(reactViteGeneratedBadFixture.brief, ["react"]), 300, reactViteGeneratedBadFixture.directories);
    const goodViolations = reviewFileSummaries(reactViteGeneratedGoodFixture.files, generateContract(reactViteGeneratedGoodFixture.brief, ["react"]), 300, reactViteGeneratedGoodFixture.directories);

    assert.equal(badViolations.some((violation) => violation.severity === "error"), true);
    assert.equal(goodViolations.some((violation) => violation.severity === "error"), false);
  });
});

describe("stack pack workflow", () => {
  it("discovers relevant upstream llms.txt sources", () => {
    const highPriority = discoverLlmsSources({ priority: "high" });
    const authSources = discoverLlmsSources({ category: "auth" });
    const ingested = listIngestedLlmsSources();

    assert.equal(highPriority.some((source) => source.id === "nextjs"), true);
    assert.equal(highPriority.some((source) => source.id === "openai"), true);
    assert.equal(authSources.some((source) => source.id === "auth0"), true);
    assert.equal(ingested.sources.length >= 39, true);
    assert.equal(ingested.sources.filter((source) => source.status === "ok").length >= 39, true);
  });

  it("proposes, reviews, promotes, and diffs stack pack candidates from local source text", async () => {
    const sourceText = await readFile("/Users/tonycordner/Documents/GitHub/architect-mcp/stack-sources/nextjs.md", "utf8");
    const strategy = stackPackExpansionStrategy();
    const candidate = proposeStackPackRules({
      stackName: "Acme Next Runtime",
      sourceText,
      sourceLabel: "Local Next.js source snapshot"
    });
    const review = reviewStackPackCandidate(candidate);
    const promoted = promoteStackPackCandidate(candidate);
    const diff = diffStackPackVersions(promoted.pack, {
      ...promoted.pack,
      fileRules: promoted.pack.fileRules.slice(0, 1)
    });

    assert.equal(strategy.priority.includes("nextjs"), true);
    assert.equal(candidate.fileRules.every((rule) => rule.triggerKind && rule.detectors?.length), true);
    assert.equal(review.valid, true);
    assert.equal(review.conflicts.length, 0);
    assert.equal(promoted.pack.id, "acme-next-runtime");
    assert.equal(diff.breaking, true);
  });

  it("derives source-backed candidates from ingested llms.txt snapshots", () => {
    const derived = deriveStackPackFromIngestedSource("hono");

    assert.equal(derived.source.id, "hono");
    assert.equal(derived.candidate.sources[0]?.note?.includes("sha256"), true);
    assert.equal(derived.matchedEvidence.some((line) => /testing|route|validation|middleware/i.test(line)), true);
    assert.equal(derived.candidate.fileRules.some((rule) => rule.triggerKind === "route-thinness"), true);
    assert.equal(derived.review.violations.length, 0);
  });

  it("blocks vague derived rules and reports overlapping stack-pack rules", () => {
    const candidate = proposeStackPackRules({
      stackName: "Vague Runtime",
      sourceText: "server api route testing validation",
      sourceLabel: "Local runtime source snapshot"
    });
    candidate.fileRules[0] = {
      ...candidate.fileRules[0],
      rule: "Use best practices",
      trigger: "Use best practices",
      recommendation: "Use best practices"
    };

    const review = reviewStackPackCandidate(candidate);
    const conflicts = analyzeStackPackConflicts([
      {
        ...candidate,
        id: "a",
        fileRules: [{
          ...candidate.fileRules[0],
          rule: "UI files must not call database clients or own query/migration logic.",
          triggerKind: "direct-db-access",
          appliesToPaths: ["src/**/*.tsx"]
        }]
      },
      {
        ...candidate,
        id: "b",
        fileRules: [{
          ...candidate.fileRules[0],
          rule: "UI files must not call database clients or own query/migration logic.",
          triggerKind: "direct-db-access",
          appliesToPaths: ["src/**/*.tsx"]
        }]
      }
    ]);

    assert.equal(review.valid, false);
    assert.equal(review.violations.some((violation) => violation.message.includes("too generic")), true);
    assert.equal(conflicts.some((conflict) => conflict.code === "duplicate-rule"), true);
    assert.equal(conflicts.some((conflict) => conflict.code === "overlapping-path-trigger"), true);
  });
});

describe("resolveStackPacks", () => {
  it("selects the MCP server pack without selecting the generic Node API pack", () => {
    const packs = resolveStackPacks({
      backend: "TypeScript MCP server",
      deployment: "Node HTTP Streamable MCP"
    });

    assert.deepEqual(packs.map((pack) => pack.id), ["mcp-server"]);
  });
});

describe("validateStackPacks", () => {
  it("validates the local pack files", () => {
    const result = validateStackPacks();

    assert.equal(result.valid, true);
    assert.equal(result.errors.length, 0);
    assert.equal(result.packCount >= 6, true);
  });
});

describe("generateContract", () => {
  it("loads stack packs from pack files", () => {
    const contract = generateContract({
      idea: "A Next.js app with Supabase",
      stack: {
        frontend: "Next.js",
        database: "Supabase"
      }
    });

    assert.deepEqual(contract.stackPacks.map((pack) => pack.id), ["nextjs", "supabase"]);
    assert.equal(contract.fileRules.some((rule) => rule.trigger), true);
  });

  it("maps canonical contract directories to an existing repo layout", () => {
    const contract = generateContract({
      idea: "A React Native app with Express and Supabase",
      stack: {
        frontend: "React",
        backend: "Node API Express",
        database: "Postgres Supabase"
      },
      repoLayout: {
        pathMap: {
          "src/features": ["client"],
          "src/shared": ["shared"],
          "src/shared/ui": ["client/components"],
          "src/server": ["server"],
          "src/server/routes": ["server/routes"],
          "src/db": ["supabase"],
          "src/db/schema": ["shared"],
          "src/db/supabase": ["supabase"]
        }
      }
    });

    const paths = contract.directories.map((directory) => directory.path);

    assert.equal(paths.includes("client"), true);
    assert.equal(paths.includes("server/routes"), true);
    assert.equal(paths.includes("supabase"), true);
    assert.equal(paths.includes("src/features"), false);
  });
});

describe("inferRepoLayoutFromFiles", () => {
  it("infers Deucalion-style top-level layout mappings", () => {
    const layout = inferRepoLayoutFromFiles([
      "client/screens/DashboardScreen.tsx",
      "admin/react/pages/AccountSettingsPage.jsx",
      "shared/schema.ts",
      "server/routes/inspectionRoutes.ts",
      "server/migrations/001-update.ts",
      "supabase/config.toml",
      "client/screens/__tests__/DashboardScreen.test.tsx"
    ]);

    assert.deepEqual(layout.pathMap["src/server/routes"], ["server/routes"]);
    assert.deepEqual(layout.pathMap["src/db/schema"], ["shared"]);
    assert.deepEqual(layout.pathMap["src/db/supabase"], ["supabase"]);
    assert.equal(layout.pathMap["src/features"]?.includes("client"), true);
  });
});

describe("createReviewReport", () => {
  it("groups and suppresses low-value migration noise", () => {
    const report = createReviewReport([
      {
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "medium",
        severity: "warning",
        path: "client/screens/A.tsx",
        message: "File has 450 lines, above the 300-line review threshold.",
        recommendation: "Split it."
      },
      {
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "medium",
        severity: "warning",
        path: "docs/audits/data.json",
        message: "File has 5000 lines, above the 300-line review threshold.",
        recommendation: "Split it."
      },
      {
        code: "ARCH009_SERVER_ENV_IN_UI",
        confidence: "high",
        severity: "error",
        path: "client/App.tsx",
        message: "UI file reads server-only environment variables.",
        recommendation: "Move secret access."
      }
    ], { mode: "migration" });

    assert.equal(report.summary.errors, 1);
    assert.equal(report.gate.status, "fail");
    assert.equal(report.summary.noiseSuppressed >= 2, true);
    assert.equal(report.priorityFindings.length, 1);
    assert.equal(report.priorityFindings[0]?.severity, "error");
  });

  it("suppresses findings included in the baseline", () => {
    const report = createReviewReport([
      {
        code: "ARCH009_SERVER_ENV_IN_UI",
        confidence: "high",
        severity: "error",
        path: "client/App.tsx",
        message: "UI file reads server-only environment variables.",
        recommendation: "Move secret access."
      },
      {
        code: "ARCH002_UI_DB_ACCESS",
        confidence: "high",
        severity: "error",
        path: "client/UserTable.tsx",
        message: "UI file has direct database access.",
        recommendation: "Move database access behind a server boundary."
      }
    ], {
      baseline: {
        findings: [
          {
            code: "ARCH009_SERVER_ENV_IN_UI",
            path: "client/App.tsx"
          }
        ]
      }
    });

    assert.equal(report.summary.errors, 1);
    assert.equal(report.summary.baselineSuppressed, 1);
    assert.equal(report.priorityFindings[0]?.path, "client/UserTable.tsx");
  });

  it("does not allow code-only baselines to suppress high-confidence errors", () => {
    const report = createReviewReport([
      {
        code: "ARCH002_UI_DB_ACCESS",
        confidence: "high",
        severity: "error",
        path: "client/UserTable.tsx",
        message: "UI file has direct database access.",
        recommendation: "Move database access behind a server boundary."
      }
    ], {
      baseline: {
        findings: [
          {
            code: "ARCH002_UI_DB_ACCESS"
          }
        ]
      }
    });

    assert.equal(report.summary.errors, 1);
    assert.equal(report.summary.baselineSuppressed, 0);
    assert.equal(report.gate.status, "fail");
    assert.equal(report.gate.lifecycle?.newHighConfidenceErrors, 1);
  });

  it("allows code-only baselines for lower-risk warnings", () => {
    const report = createReviewReport([
      {
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "medium",
        severity: "warning",
        path: "src/domain/a.ts",
        message: "File has 450 lines, above the 300-line source threshold.",
        recommendation: "Split it."
      }
    ], {
      baseline: {
        findings: [
          {
            code: "ARCH001_OVERSIZED_FILE"
          }
        ]
      }
    });

    assert.equal(report.summary.warnings, 0);
    assert.equal(report.summary.baselineSuppressed, 1);
  });

  it("returns a configurable review gate", () => {
    const report = createReviewReport([
      {
        code: "ARCH001_OVERSIZED_FILE",
        confidence: "medium",
        severity: "warning",
        path: "src/domain/a.ts",
        message: "File has 450 lines, above the 300-line source threshold.",
        recommendation: "Split it."
      }
    ], {
      mode: "ci",
      gate: {
        maxWarnings: 2,
        minScore: 50
      }
    });

    assert.equal(report.gate.status, "pass");
    assert.equal(report.gate.thresholds.maxErrors, 0);
    assert.equal(report.gate.thresholds.maxWarnings, 2);
  });
});

describe("scanWorkspace", () => {
  it("extracts imports and env access with the TypeScript parser", async () => {
    const root = await mkdtemp(join(tmpdir(), "architect-mcp-test-"));
    await writeFile(join(root, "sample.ts"), [
      "import api from './api';",
      "export { api };",
      "const literal = process.env.SECRET_KEY;",
      "const dynamic = process.env[envName];"
    ].join("\n"));

    const summaries = await scanWorkspace(root, 10);
    const sample = summaries.find((summary) => summary.path === "sample.ts");

    assert.deepEqual(sample?.imports, ["./api"]);
    assert.deepEqual(sample?.envAccesses, ["SECRET_KEY"]);
  });
});

describe("grillMe", () => {
  it("blocks implementation when the brief is too vague", () => {
    const result = grillMe({
      idea: "Build an app"
    });

    assert.equal(result.ready, false);
    assert.equal(result.phase, "intake");
    assert.equal(result.blockers.some((blocker) => blocker.includes("primary user")), true);
    assert.equal(result.contract, undefined);
  });

  it("selects stack packs and produces a contract for a ready brief", () => {
    const result = grillMe({
      idea: "A repo standards and verification MCP server.",
      users: "Solo founders who need agents to generate maintainable repos.",
      coreFlows: ["grill the brief", "generate contracts", "review repo structure"],
      stack: {
        backend: "TypeScript MCP server",
        deployment: "Node HTTP Streamable MCP"
      },
      storage: "No persistence in V1.",
      enforcement: "Advise locally; CI later.",
      repoLayout: {
        pathMap: {
          "src/domain": ["src/domain"],
          "src/infrastructure": ["src/infrastructure"],
          "src/server": ["src/server"],
          "tests": ["tests"]
        }
      },
      risk: "Agents creating monolithic files or mixing tool wiring into domain logic.",
      verification: ["npm run typecheck", "npm test", "npm run build"]
    }, {
      includeContract: true,
      includeArtifacts: true
    });

    assert.equal(result.ready, true);
    assert.equal(result.phase, "contract");
    assert.deepEqual(result.selectedStackPacks, ["mcp-server"]);
    assert.equal(result.contract?.stackPacks[0]?.id, "mcp-server");
    assert.equal(result.artifacts?.some((artifact) => artifact.path === "AGENTS.md"), true);
    assert.equal(result.scaffoldPlan?.some((item) => item.path === "docs/architecture-contract.md"), true);
  });

  it("blocks readiness for auth without an explicit trust boundary", () => {
    const result = grillMe({
      idea: "A customer portal.",
      users: "Customers managing account settings.",
      coreFlows: ["sign in", "update profile"],
      stack: {
        frontend: "React",
        auth: "Supabase Auth"
      },
      storage: "No persistence beyond auth profile.",
      enforcement: "Advise during implementation.",
      risk: "Unauthorized users seeing private profile data.",
      verification: ["npm test"]
    });

    assert.equal(result.ready, false);
    assert.equal(result.challenges.some((challenge) => challenge.field === "stack.auth" && challenge.severity === "blocker"), true);
  });

  it("blocks readiness for hosted filesystem scanning ambiguity", () => {
    const result = grillMe({
      idea: "Hosted MCP that scans local filesystem paths.",
      users: "Developers reviewing repo structure.",
      coreFlows: ["scan workspace", "review findings"],
      stack: {
        backend: "TypeScript MCP server",
        deployment: "Hosted HTTP"
      },
      storage: "No persistence.",
      enforcement: "Advise during implementation.",
      risk: "Hosted server reading arbitrary local paths.",
      verification: ["npm test"]
    });

    assert.equal(result.ready, false);
    assert.equal(result.challenges.some((challenge) => challenge.field === "stack.deployment" && challenge.severity === "blocker"), true);
  });
});
