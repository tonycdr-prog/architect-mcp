import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { generateContract } from "../src/domain/contract.js";
import { reviewFileSummaries } from "../src/domain/reviewer.js";

describe("promoted stack-pack executable detectors", () => {
  it("flags auth, payment, validation, AI, test, and route detector signals", () => {
    const contract = generateContract({
      idea: "A full-stack app",
      stack: {
        backend: "Hono with Auth0 Stripe AI SDK Zod Vitest"
      },
      verification: ["npm test"]
    }, ["hono", "auth0", "stripe", "ai-sdk", "zod", "vitest"]);

    const findings = reviewFileSummaries([
      { path: "src/server/routes/users.ts", lines: 260 },
      { path: "src/app/ProfilePage.tsx", lines: 120, imports: ["../auth/session"], hasUseClient: true },
      { path: "src/components/Checkout.tsx", lines: 90, imports: ["stripe"] },
      { path: "src/app/FormPage.tsx", lines: 80, imports: ["zod"] },
      { path: "src/features/chat/client.ts", lines: 120, imports: ["openai"] },
      { path: "src/domain/largeFeature.ts", lines: 260 }
    ], contract);

    const messages = findings.map((finding) => finding.message).join("\n");
    assert.match(messages, /Hono Thin Route Files/);
    assert.match(messages, /Auth Server Boundary/);
    assert.match(messages, /Stripe Payment Server Boundary/);
    assert.match(messages, /Zod Validation Boundary/);
    assert.match(messages, /AI Tool Safety Boundary/);
    assert.match(messages, /Vitest Test Policy/);
  });

  it("allows Stripe secret access inside server-owned payment modules", () => {
    const contract = generateContract({
      idea: "A paid app",
      stack: {
        backend: "Stripe"
      },
      verification: ["npm test"]
    }, ["stripe"]);

    const findings = reviewFileSummaries([
      { path: "src/server/payments/createCheckout.ts", lines: 120, imports: ["stripe"], envAccesses: ["STRIPE_SECRET_KEY"] }
    ], contract);

    assert.equal(findings.some((finding) => finding.message.includes("Stripe Payment Server Boundary")), false);
  });

  it("allows env access in typed config modules for env-spread pack rules", () => {
    const contract = generateContract({
      idea: "A Node API",
      stack: {
        backend: "Node API"
      },
      verification: ["npm test"]
    }, ["node-api"]);

    const findings = reviewFileSummaries([
      { path: "src/server/config/env.ts", lines: 60, envAccesses: ["PORT"] }
    ], contract);

    assert.equal(findings.some((finding) => finding.code === "ARCH008_ENV_SCATTER"), false);
  });
});
