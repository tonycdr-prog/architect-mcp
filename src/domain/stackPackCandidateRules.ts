import type { StackPack } from "./types.js";

export function candidateRulesForText(text: string, packId: string): StackPack["fileRules"] {
  const rules: StackPack["fileRules"] = [];
  if (/component|client|ui|route|page/i.test(text)) {
    rules.push(makeRule("No god components", "Components and route views should not own fetching, mutation, validation, layout, and rendering in one file.", "src/**/*.tsx", "line-threshold"));
  }
  if (/database|query|schema|migration|sql|postgres|supabase/i.test(text)) {
    rules.push(makeRule("No UI queries", "UI files must not call database clients or own query/migration logic.", "src/**/*.tsx", "direct-db-access"));
  }
  if (/server|api|route|controller|handler/i.test(text)) {
    rules.push(makeRule("Thin route files", "Routes/controllers validate transport and call services/use-cases instead of owning workflows.", "src/**/*.{ts,tsx}", "route-thinness"));
  }
  if (rules.length === 0) {
    rules.push(makeRule("Manual architecture review", "Review generated files against the documented stack boundaries before promotion.", "src/**/*.{ts,tsx,js,jsx}", "manual-review"));
  }
  return rules.map((rule) => ({ ...rule, name: `${titleize(packId)} ${rule.name}` }));
}

export function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "custom-stack";
}

export function titleize(value: string): string {
  return value.split(/[-\s]+/).filter(Boolean).map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`).join(" ");
}

function makeRule(name: string, rule: string, path: string, triggerKind: NonNullable<StackPack["fileRules"][number]["triggerKind"]>): StackPack["fileRules"][number] {
  const trigger = triggerForKind(triggerKind, rule);
  return {
    name,
    rule,
    severity: triggerKind === "manual-review" ? "warning" : "error",
    trigger,
    recommendation: recommendationForKind(triggerKind),
    appliesToPaths: [path],
    goodExample: goodExampleForKind(triggerKind),
    badExample: badExampleForKind(triggerKind),
    triggerKind,
    detectors: [{ kind: triggerKind, description: `${trigger} ${rule}` }]
  };
}

function triggerForKind(triggerKind: NonNullable<StackPack["fileRules"][number]["triggerKind"]>, fallback: string): string {
  const triggers: Record<NonNullable<StackPack["fileRules"][number]["triggerKind"]>, string> = {
    "line-threshold": "A UI, page, route, or component file crosses the configured line threshold while also owning multiple responsibilities.",
    "import-boundary": "A lower-level shared module imports a feature-owned module or creates a cross-feature dependency.",
    "direct-db-access": "A UI or client-facing file imports a database client, schema, migration, SQL helper, or repository implementation.",
    "env-access": "A client-facing file reads a non-public environment variable.",
    "client-boundary": "A client component imports server-only code or server modules import browser-only UI concerns.",
    "route-thinness": "A route, controller, or handler owns workflow orchestration instead of delegating to a service/use-case module.",
    "migration-discipline": "Schema or migration files change without colocated repository/query ownership and behavior tests.",
    "hosted-filesystem": "Hosted execution attempts to scan arbitrary local filesystem paths instead of explicit repo snapshots.",
    "auth-boundary": "A UI/client-facing file owns auth/session/token verification or reads auth secret configuration.",
    "payment-boundary": "Payment SDK calls, checkout creation, webhook verification, or payment secrets appear outside server-owned payment modules.",
    "test-policy": "Large changed implementation files lack nearby behavior-test evidence.",
    "validation-boundary": "Reusable validation schemas are defined in UI/page files instead of validation-owned modules.",
    "ai-tool-safety": "Model calls, provider clients, or tool execution appear outside server/tool-owned modules.",
    "manual-review": fallback
  };
  return triggers[triggerKind];
}

function recommendationForKind(triggerKind: NonNullable<StackPack["fileRules"][number]["triggerKind"]>): string {
  const recommendations: Record<NonNullable<StackPack["fileRules"][number]["triggerKind"]>, string> = {
    "line-threshold": "Split rendering, state, data loading, validation, and mutation work into feature-owned components, hooks, and services before adding behavior.",
    "import-boundary": "Move shared behavior into a dependency-free shared module or invert the dependency so feature modules compose shared code.",
    "direct-db-access": "Move database access behind a server-owned service, repository, action, or API boundary and keep UI files transport-only.",
    "env-access": "Expose only intentionally public configuration through the framework's public env mechanism and keep secrets on the server side.",
    "client-boundary": "Separate client UI from server-only modules and pass serialized data through props, API responses, or server actions.",
    "route-thinness": "Keep route files responsible for transport parsing and response shaping; put business workflows in services/use-cases.",
    "migration-discipline": "Pair schema changes with migration artifacts, repository/query updates, and tests that prove the changed data behavior.",
    "hosted-filesystem": "Require an uploaded manifest, git-provider snapshot, or explicit repo archive before hosted review begins.",
    "auth-boundary": "Move auth/session/token decisions into server-owned auth modules and pass safe user state to clients.",
    "payment-boundary": "Move payment mutations and webhook verification into server-owned payment modules.",
    "test-policy": "Add or update focused behavior tests for the changed module.",
    "validation-boundary": "Move shared validation schemas to feature-owned or shared validation modules.",
    "ai-tool-safety": "Move model calls and tool execution into server-owned AI/tool modules.",
    "manual-review": "Turn the documented guidance into an executable rule before promotion."
  };
  return recommendations[triggerKind];
}

function goodExampleForKind(triggerKind: NonNullable<StackPack["fileRules"][number]["triggerKind"]>): string {
  return triggerKind === "direct-db-access"
    ? "A page calls a server action or API route that delegates to src/db/repositories/userRepository.ts."
    : "A small entry file composes feature, service, database, or shared modules with one clear responsibility.";
}

function badExampleForKind(triggerKind: NonNullable<StackPack["fileRules"][number]["triggerKind"]>): string {
  return triggerKind === "direct-db-access"
    ? "A component imports a database client and builds SQL queries directly inside render or event handlers."
    : "One file owns rendering, validation, data access, side effects, and workflow orchestration.";
}
