import { inferAppArchetype } from "./archetypes.js";
import type { BuildPlan, ProjectBrief } from "./types.js";

export function generateBuildPlan(brief: ProjectBrief): BuildPlan {
  const archetype = inferAppArchetype(brief);
  const checks = brief.verification?.length ? brief.verification : ["typecheck", "test", "architecture review"];
  const coreFlows = brief.coreFlows?.slice(0, 3) ?? ["first core workflow"];
  const hasFrontend = Boolean(brief.stack?.frontend);
  const hasBackend = Boolean(brief.stack?.backend);
  const hasDatabase = Boolean(brief.stack?.database);

  return {
    archetype,
    slices: [
      {
        id: "agent-harness",
        title: "Agent Harness And Contract",
        order: 1,
        goal: "Commit the instructions future agents must follow before writing app code.",
        inputs: ["project brief", "/grill-me result", "selected stack packs"],
        outputs: ["AGENTS.md", "docs/architecture-contract.md", ".cursor/rules/architecture.mdc"],
        allowedDirectories: ["docs", ".cursor/rules"],
        forbiddenFiles: ["src/App.tsx", "src/app/page.tsx", "src/server.ts", "src/index.ts"],
        files: ["AGENTS.md", "docs/architecture-contract.md", ".cursor/rules/architecture.mdc"],
        checks: ["architecture contract validates"],
        stopAfter: "Stop if /grill-me still has blocker questions."
      },
      {
        id: "repo-scaffold",
        title: "Responsibility-Based Scaffold",
        order: 2,
        goal: "Create only the directories needed for selected stack packs and core workflows.",
        inputs: ["architecture contract", "repo layout mapping"],
        outputs: ["responsibility-owned directories", "empty feature/server/db/test boundaries"],
        allowedDirectories: ["src/app", "src/features", "src/shared", "src/server", "src/db", "tests", "docs"],
        forbiddenFiles: ["src/App.tsx", "src/server.ts", "src/index.ts"],
        files: ["src/app or mapped app entry", "src/features or mapped feature folders", "src/server/services", "src/db"],
        checks: ["architecture review"],
        stopAfter: "Stop if scaffold review finds missing boundaries."
      },
      ...(hasDatabase
        ? [{
            id: "database-boundary",
            title: "Database Boundary",
            order: 3,
            goal: "Define schema, migrations, seed strategy, and repository/query ownership before UI uses data.",
            inputs: ["data entities", "database stack", "migration strategy"],
            outputs: ["schema ownership", "migration ownership", "repository/query modules"],
            allowedDirectories: ["src/db", "src/server/repositories", "src/server/services", "tests"],
            forbiddenFiles: ["src/App.tsx", "src/app/page.tsx", "src/features/**/page.tsx"],
            files: ["src/db/schema", "src/db/migrations", "src/db/repositories"],
            checks,
            stopAfter: "Stop if UI files import database clients."
          }]
        : []),
      ...(hasBackend
        ? [{
            id: "backend-boundary",
            title: "Backend Services And Routes",
            order: 4,
            goal: "Create thin routes/controllers backed by services/use-cases and adapters.",
            inputs: ["core workflows", "auth/trust boundary", "database repositories if present"],
            outputs: ["routes/controllers", "services/use-cases", "adapters"],
            allowedDirectories: ["src/server/routes", "src/server/services", "src/server/adapters", "tests"],
            forbiddenFiles: ["src/server.ts", "src/index.ts", "src/app/api/route.ts"],
            files: ["src/server/routes", "src/server/services", "src/server/adapters"],
            checks,
            stopAfter: "Stop if routes own business workflows."
          }]
        : []),
      ...(hasFrontend
        ? [{
            id: "frontend-boundary",
            title: "Frontend Feature Slices",
            order: 5,
            goal: `Build feature-owned UI for ${coreFlows.join(", ")} without god components.`,
            inputs: ["core workflows", "server/API contracts", "feature ownership map"],
            outputs: ["feature screens/routes", "data hooks", "presentational components", "behavior tests"],
            allowedDirectories: ["src/app", "src/features", "src/shared/ui", "tests"],
            forbiddenFiles: ["src/App.tsx", "src/app/page.tsx", "src/pages/index.tsx"],
            files: coreFlows.map((flow) => `src/features/${slugify(flow)}`),
            checks,
            stopAfter: "Stop after each workflow and run review before adding the next."
          }]
        : []),
      {
        id: "verification",
        title: "Verification And Review Gate",
        order: 6,
        goal: "Run named checks and architecture review before accepting generated work.",
        inputs: ["implemented slice", "review baseline if any", "verification commands"],
        outputs: ["test results", "architecture review report", "accepted/resolved baseline notes"],
        allowedDirectories: ["tests", "docs"],
        forbiddenFiles: ["baseline entries without path or reason"],
        files: ["tests", "review baseline only if adopting in a mature repo"],
        checks,
        stopAfter: "Stop if review gate fails or new high-confidence errors appear."
      }
    ].map((slice, index) => ({ ...slice, order: index + 1 }))
  };
}

function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "workflow";
}
