import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { generateContract } from "../src/domain/contract.js";
import { grillMe } from "../src/domain/intake.js";
import { reviewProposedFilePlan } from "../src/domain/planReviewer.js";
import { reviewFileSummaries } from "../src/domain/reviewer.js";
import { createReviewReport } from "../src/domain/reviewReport.js";
import { reviewMcpConfigSecurity } from "../src/domain/mcpSecurity.js";
import { reviewAgentFinalResponse } from "../src/domain/finalResponseReview.js";

describe("realistic full repo loop", () => {
  it("runs a synthetic app through grill, contract, plan, review, MCP config, and final response", () => {
    const brief = {
      idea: "A paid AI support dashboard.",
      users: "Support managers handling customer escalations.",
      coreFlows: ["review tickets", "summarize ticket", "start paid plan checkout"],
      dataEntities: ["tickets owned by ticketRepository", "billing plans owned by paymentRepository"],
      stack: { frontend: "React", backend: "Hono with Stripe and AI SDK", auth: "Auth0", database: "Postgres" },
      storage: "Persist tickets and billing state.",
      enforcement: "Fail CI on architecture errors.",
      repoLayout: {
        pathMap: {
          "src/features": ["src/features"],
          "src/server/routes": ["src/server/routes"],
          "src/server/services": ["src/server/services"],
          "src/server/payments": ["src/server/payments"],
          "src/server/ai": ["src/server/ai"],
          "src/server/auth": ["src/server/auth"],
          "src/db": ["src/db"],
          "tests": ["tests"]
        }
      },
      risk: "Payment, Auth0 sessions/roles are validated in src/server/auth, AI calls stay in src/server/ai, schema/migrations/repositories own data access, and AGENTS.md plus architecture contract are generated before coding.",
      verification: ["npm run typecheck", "npm test", "npm run build"]
    };
    const grilled = grillMe(brief, { includeContract: true, stackPackIds: ["react", "hono", "stripe", "auth0", "ai-sdk", "vitest"] });
    const contract = grilled.contract ?? generateContract(brief, ["react", "hono", "stripe", "auth0", "ai-sdk", "vitest"]);
    const planFindings = reviewProposedFilePlan({
      files: [
        { path: "AGENTS.md", purpose: "Agent instructions." },
        { path: "docs/architecture-contract.md", purpose: "Contract." },
        { path: "src/features/tickets/TicketDashboard.tsx", purpose: "Ticket UI only." },
        { path: "src/server/payments/createCheckout.ts", purpose: "Stripe checkout." },
        { path: "src/server/ai/summarizeTicket.ts", purpose: "AI summary service." }
      ]
    });
    const repoFindings = reviewFileSummaries([
      { path: "AGENTS.md", lines: 80 },
      { path: "docs/architecture-contract.md", lines: 100 },
      { path: "src/features/tickets/TicketDashboard.tsx", lines: 180 },
      { path: "src/server/routes/ticketRoutes.ts", lines: 120 },
      { path: "src/server/config/env.ts", lines: 70, envAccesses: ["STRIPE_SECRET_KEY"] },
      { path: "src/server/payments/createCheckout.ts", lines: 120, imports: ["stripe", "../config/env"] },
      { path: "src/server/ai/summarizeTicket.ts", lines: 130, imports: ["ai"] },
      { path: "tests/tickets.test.ts", lines: 90 }
    ], contract, 300, ["src/app", "src/features", "src/server/routes", "src/server/payments", "src/server/ai", "src/server/auth", "src/server/services", "src/server/adapters", "src/db", "src/db/schema", "src/db/migrations", "src/db/repositories", "src/shared", "src/shared/ui", "tests", "docs"]);
    const configReview = reviewMcpConfigSecurity({ config: { mcpServers: { safe: { command: "npx", args: ["-y", "safe-mcp@1.2.3"] } } } });
    const finalReview = reviewAgentFinalResponse({
      response: "Changed the synthetic app plan. Verified with npm run typecheck, npm test, and npm run build. Assumptions: no new assumptions. Not done: no remaining requested work.",
      requiredChecks: brief.verification
    });

    assert.equal(grilled.ready, true);
    assert.equal(planFindings.length, 0);
    assert.equal(createReviewReport(repoFindings, { mode: "ci" }).gate.status, "pass");
    assert.equal(configReview.status, "pass");
    assert.equal(finalReview.status, "pass");
  });
});
