import type { FileSummary, ProjectBrief } from "../../src/domain/types.js";

export type RepoFixture = {
  name: string;
  brief: ProjectBrief;
  files: FileSummary[];
  directories: string[];
  stackPackIds?: string[];
};

export const cleanMcpServerFixture: RepoFixture = {
  name: "clean-mcp-server",
  brief: {
    idea: "A local-first MCP server that generates architecture contracts and reviews repo structure.",
    users: "Builders using coding agents who need repo guardrails before implementation.",
    coreFlows: ["grill project brief", "generate contracts", "review repo structure"],
    stack: {
      backend: "TypeScript MCP server",
      deployment: "Node HTTP Streamable MCP"
    },
    storage: "No persistence in V1.",
    enforcement: "Advise locally and expose review gates.",
    repoLayout: {
      pathMap: {
        "src/domain": ["src/domain"],
        "src/tools": ["src/tools"],
        "src/server": ["src/server"],
        "src/infrastructure": ["src/infrastructure"],
        "tests": ["tests"]
      }
    },
    risk: "Agents mixing MCP transport wiring into domain logic.",
    verification: ["npm run typecheck", "npm test", "npm run build"]
  },
  stackPackIds: ["mcp-server"],
  directories: ["src/domain", "src/tools", "src/server", "src/infrastructure", "tests", "docs"],
  files: [
    { path: "src/domain/intake.ts", lines: 140 },
    { path: "src/domain/reviewer.ts", lines: 240 },
    { path: "src/tools/intakeTools.ts", lines: 70 },
    { path: "src/server/createArchitectServer.ts", lines: 40 },
    { path: "src/infrastructure/scanWorkspace.ts", lines: 110 },
    { path: "tests/reviewer.test.ts", lines: 260 }
  ]
};

export const messyReactFixture: RepoFixture = {
  name: "messy-react-app",
  brief: {
    idea: "A React app with a Postgres-backed admin dashboard.",
    users: "Ops admins who need to manage customer records.",
    coreFlows: ["view dashboard", "edit customers", "export reports"],
    stack: {
      frontend: "React",
      database: "Postgres"
    },
    storage: "Persist customer and report data.",
    enforcement: "Fail CI on architecture errors.",
    risk: "UI components calling the database and reading server-only secrets.",
    verification: ["npm test"]
  },
  directories: ["src/app", "src/features", "src/shared", "src/db", "tests"],
  files: [
    {
      path: "src/features/customers/CustomerDashboard.tsx",
      lines: 950,
      hasDirectDbAccess: true,
      envAccesses: ["DATABASE_URL"]
    },
    {
      path: "src/shared/ui/Button.tsx",
      lines: 80,
      imports: ["@/features/customers/customerState"]
    },
    {
      path: "src/app/page.tsx",
      lines: 280
    }
  ]
};

export const existingLayoutFixture: RepoFixture = {
  name: "existing-layout",
  brief: {
    idea: "A React Native field app with an Express API.",
    users: "Field engineers collecting inspection data.",
    coreFlows: ["capture inspection", "sync records", "admin review"],
    stack: {
      frontend: "React Native",
      backend: "Express Node API",
      database: "Postgres"
    },
    storage: "Persist inspections and assets.",
    enforcement: "Advise in local reviews.",
    repoLayout: {
      pathMap: {
        "src/features": ["client"],
        "src/shared": ["shared"],
        "src/shared/ui": ["client/components"],
        "src/server": ["server"],
        "src/server/routes": ["server/routes"],
        "src/db": ["supabase"],
        "src/db/schema": ["shared"],
        "src/db/migrations": ["server/migrations"],
        "tests": ["client", "server"]
      }
    },
    risk: "Forcing a src folder onto an established client/server/shared layout.",
    verification: ["npm test", "npm run check:types"]
  },
  directories: ["client", "client/components", "server", "server/routes", "server/migrations", "shared", "supabase", "docs"],
  files: [
    { path: "client/screens/InspectionScreen.tsx", lines: 320 },
    { path: "client/components/InspectionCard.tsx", lines: 140 },
    { path: "server/routes/inspectionRoutes.ts", lines: 120 },
    { path: "shared/schema.ts", lines: 900 },
    { path: "server/migrations/001-init.ts", lines: 80 }
  ]
};

export const migrationBaselineFixture: RepoFixture = {
  name: "migration-baseline",
  brief: {
    idea: "A mature app adopting architecture review after existing debt.",
    users: "Maintainers who want to prevent new architecture regressions.",
    coreFlows: ["review current repo", "create baseline", "fail new findings"],
    stack: {
      frontend: "React",
      backend: "Node API",
      database: "Postgres"
    },
    storage: "Persist app data, but not review history yet.",
    enforcement: "Use baseline to suppress existing warnings.",
    risk: "Blocking all historical warnings would make adoption impossible.",
    verification: ["npm test", "architecture review"]
  },
  directories: ["src/app", "src/features", "src/shared", "src/server", "src/db", "tests", "docs"],
  files: [
    { path: "src/features/legacy/LegacyDashboard.tsx", lines: 980 },
    { path: "src/server/routes/legacyRoutes.ts", lines: 460 },
    { path: "src/shared/schema.ts", lines: 1300 },
    { path: "src/features/new/NewPanel.tsx", lines: 160 }
  ]
};

export const generatedAppBadFixture: RepoFixture = {
  name: "generated-app-bad",
  brief: {
    idea: "A React admin app for managing customer records.",
    users: "Ops admins who manage customer records.",
    coreFlows: ["view customers", "edit customers", "export customers"],
    stack: {
      frontend: "React",
      backend: "Node API",
      database: "Postgres"
    },
    storage: "Persist customers and audit events.",
    enforcement: "Fail CI on new architecture errors.",
    risk: "Agent creates one giant App.tsx with database calls and no tests.",
    verification: ["npm run typecheck", "npm test"]
  },
  directories: ["src", "src/db"],
  files: [
    {
      path: "src/App.tsx",
      lines: 620,
      imports: ["@/db/client", "@/server/services/customerService"],
      hasDirectDbAccess: true,
      envAccesses: ["DATABASE_URL"]
    },
    { path: "src/db/schema.ts", lines: 120 },
    { path: "src/index.tsx", lines: 80, imports: ["./App"] }
  ]
};

export const generatedAppGoodFixture: RepoFixture = {
  name: "generated-app-good",
  brief: generatedAppBadFixture.brief,
  directories: ["src/app", "src/features/customers", "src/server/routes", "src/server/services", "src/db/schema", "tests", "docs", ".cursor/rules"],
  files: [
    { path: "AGENTS.md", lines: 80 },
    { path: "docs/architecture-contract.md", lines: 120 },
    { path: "docs/build-plan.md", lines: 90 },
    { path: ".cursor/rules/architecture.mdc", lines: 60 },
    { path: "src/app/AppShell.tsx", lines: 90 },
    { path: "src/features/customers/CustomerListPage.tsx", lines: 180 },
    { path: "src/features/customers/useCustomers.ts", lines: 90, imports: ["@/server/routes/customerRoutes"] },
    { path: "src/server/routes/customerRoutes.ts", lines: 90, imports: ["@/server/services/customerService"] },
    { path: "src/server/services/customerService.ts", lines: 160, imports: ["@/db/repositories/customerRepository"] },
    { path: "src/db/schema/customerSchema.ts", lines: 140 },
    { path: "tests/customers.test.ts", lines: 120 }
  ]
};

export const nextjsGeneratedBadFixture: RepoFixture = {
  name: "nextjs-generated-bad",
  brief: {
    idea: "A Next.js dashboard for billing operations.",
    users: "Finance admins reviewing billing status.",
    coreFlows: ["view billing", "edit plan"],
    stack: {
      frontend: "Next.js",
      backend: "Node API",
      database: "Postgres"
    },
    risk: "Agent puts all logic in page.tsx and leaks database access into client code.",
    verification: ["npm run typecheck", "npm test"]
  },
  directories: ["src/app/billing", "src/db"],
  files: [
    { path: "src/app/billing/page.tsx", lines: 520, hasUseClient: true, imports: ["@/db/billingRepository"], hasDirectDbAccess: true },
    { path: "src/db/billingRepository.ts", lines: 120 }
  ]
};

export const nextjsGeneratedGoodFixture: RepoFixture = {
  name: "nextjs-generated-good",
  brief: nextjsGeneratedBadFixture.brief,
  directories: ["src/app/billing", "src/features/billing", "src/server/services", "src/db", "tests", "docs"],
  files: [
    { path: "AGENTS.md", lines: 80 },
    { path: "docs/architecture-contract.md", lines: 100 },
    { path: "src/app/billing/page.tsx", lines: 30, imports: ["@/features/billing/BillingPage"] },
    { path: "src/features/billing/BillingPage.tsx", lines: 160, imports: ["@/features/billing/useBilling"] },
    { path: "src/features/billing/useBilling.ts", lines: 80 },
    { path: "src/server/services/billingService.ts", lines: 110, imports: ["@/db/billingRepository"] },
    { path: "src/db/billingRepository.ts", lines: 90 },
    { path: "tests/billing.test.ts", lines: 80 }
  ]
};

export const reactViteGeneratedBadFixture: RepoFixture = {
  name: "react-vite-generated-bad",
  brief: {
    idea: "A Vite React customer admin app.",
    users: "Support admins editing customers.",
    coreFlows: ["search customers", "edit customers"],
    stack: {
      frontend: "React Vite",
      backend: "Node API",
      database: "Postgres"
    },
    risk: "Agent creates a single App.tsx with DB calls and no feature boundaries.",
    verification: ["npm run typecheck", "npm test"]
  },
  directories: ["src"],
  files: [
    { path: "src/App.tsx", lines: 640, imports: ["@/db/customerRepository"], hasDirectDbAccess: true },
    { path: "src/db/customerRepository.ts", lines: 90 }
  ]
};

export const reactViteGeneratedGoodFixture: RepoFixture = {
  name: "react-vite-generated-good",
  brief: reactViteGeneratedBadFixture.brief,
  directories: ["src/app", "src/features/customers", "src/server/routes", "src/server/services", "src/server/adapters", "src/db", "src/db/schema", "src/db/migrations", "src/db/repositories", "src/shared/ui", "tests", "docs"],
  files: [
    { path: "AGENTS.md", lines: 80 },
    { path: "docs/architecture-contract.md", lines: 100 },
    { path: "src/app/App.tsx", lines: 50, imports: ["@/features/customers/CustomerAdminPage"] },
    { path: "src/features/customers/CustomerAdminPage.tsx", lines: 180, imports: ["@/features/customers/useCustomers"] },
    { path: "src/features/customers/useCustomers.ts", lines: 80 },
    { path: "src/server/services/customerService.ts", lines: 120, imports: ["@/db/customerRepository"] },
    { path: "src/db/customerRepository.ts", lines: 90 },
    { path: "tests/customers.test.ts", lines: 80 }
  ]
};
