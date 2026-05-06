import type { RepoFixture } from "./repos.js";

export const expoGeneratedBadFixture: RepoFixture = {
  name: "expo-generated-bad",
  brief: {
    idea: "An Expo field inspection mobile app.",
    users: "Field engineers capturing inspection notes and photos.",
    coreFlows: ["open assigned inspection", "capture notes", "sync inspection"],
    stack: {
      frontend: "Expo React Native"
    },
    risk: "Agent puts navigation, permissions, data fetching, local state, and rendering into one large screen.",
    verification: ["npm run typecheck", "npm test"]
  },
  stackPackIds: ["expo"],
  directories: ["src/screens"],
  files: [
    { path: "src/screens/InspectionScreen.tsx", lines: 620 }
  ]
};

export const expoGeneratedGoodFixture: RepoFixture = {
  name: "expo-generated-good",
  brief: expoGeneratedBadFixture.brief,
  stackPackIds: ["expo"],
  directories: ["src/app", "src/screens", "src/features", "src/shared", "src/shared/ui", "tests", "docs"],
  files: [
    { path: "AGENTS.md", lines: 80 },
    { path: "docs/architecture-contract.md", lines: 100 },
    { path: "src/app/App.tsx", lines: 50, imports: ["@/screens/InspectionScreen"] },
    { path: "src/screens/InspectionScreen.tsx", lines: 140, imports: ["@/features/inspection/useInspection", "@/features/inspection/InspectionForm"] },
    { path: "src/features/inspection/useInspection.ts", lines: 100 },
    { path: "src/features/inspection/InspectionForm.tsx", lines: 130 },
    { path: "src/shared/ui/Button.tsx", lines: 70 },
    { path: "tests/inspection.test.ts", lines: 90 }
  ]
};

export const paidAuthSupabaseBadFixture: RepoFixture = {
  name: "paid-auth-supabase-bad",
  brief: {
    idea: "A paid support dashboard with Auth0, Stripe, and Supabase.",
    users: "Support managers reviewing escalations and billing state.",
    coreFlows: ["review escalations", "authorize manager access", "start checkout"],
    dataEntities: ["tickets", "billing plans", "manager roles"],
    stack: {
      frontend: "React",
      backend: "Hono with Stripe and AI SDK",
      auth: "Auth0",
      database: "Supabase Postgres"
    },
    storage: "Persist tickets, roles, and billing state.",
    enforcement: "Fail CI on architecture errors.",
    risk: "UI imports auth/session helpers, Supabase service role clients, Stripe server helpers, and AI/model libraries.",
    verification: ["npm run typecheck", "npm test", "npm run build"]
  },
  stackPackIds: ["react", "hono", "auth0", "supabase", "stripe", "ai-sdk", "vitest"],
  directories: ["src/app", "src/features", "src/db/supabase"],
  files: [
    {
      path: "src/features/admin/AdminDashboard.tsx",
      lines: 360,
      hasUseClient: true,
      imports: ["@/server/auth/session", "@/db/supabase/server", "@/server/payments/createCheckout", "openai"],
      envAccesses: ["AUTH0_SECRET"]
    },
    { path: "src/db/supabase/server.ts", lines: 90 }
  ]
};

export const paidAuthSupabaseGoodFixture: RepoFixture = {
  name: "paid-auth-supabase-good",
  brief: paidAuthSupabaseBadFixture.brief,
  stackPackIds: paidAuthSupabaseBadFixture.stackPackIds,
  directories: [
    "src/app",
    "src/features",
    "src/server/routes",
    "src/server/services",
    "src/server/adapters",
    "src/server/auth",
    "src/server/payments",
    "src/server/ai",
    "src/db",
    "src/db/schema",
    "src/db/migrations",
    "src/db/repositories",
    "src/db/supabase",
    "src/shared",
    "src/shared/ui",
    "tests",
    "docs"
  ],
  files: [
    { path: "AGENTS.md", lines: 90 },
    { path: "docs/architecture-contract.md", lines: 120 },
    { path: "src/app/AdminApp.tsx", lines: 50, imports: ["@/features/admin/AdminDashboard"] },
    { path: "src/features/admin/AdminDashboard.tsx", lines: 180 },
    { path: "src/server/routes/adminRoutes.ts", lines: 90, imports: ["@/server/services/adminService"] },
    { path: "src/server/services/adminService.ts", lines: 130, imports: ["@/db/repositories/ticketRepository", "@/server/auth/requireManager"] },
    { path: "src/server/auth/requireManager.ts", lines: 80 },
    { path: "src/server/payments/createCheckout.ts", lines: 120, imports: ["stripe", "@/server/config/env"] },
    { path: "src/server/ai/summarizeTicket.ts", lines: 110, imports: ["ai"] },
    { path: "src/server/config/env.ts", lines: 80, envAccesses: ["AUTH0_SECRET", "STRIPE_SECRET_KEY"] },
    { path: "src/db/supabase/browser.ts", lines: 70 },
    { path: "src/db/supabase/server.ts", lines: 70 },
    { path: "src/db/repositories/ticketRepository.ts", lines: 110 },
    { path: "tests/admin.test.ts", lines: 110 }
  ]
};
