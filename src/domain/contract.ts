import type { ArchitectureContract, ProjectBrief } from "./types.js";
import { applyRepoLayoutToContract } from "./repoLayout.js";
import { resolveStackPacks } from "./stackPacks.js";
import { validateArchitectureContract } from "./contractValidation.js";
import { inferAppArchetype } from "./archetypes.js";
import { listFoundationPacks } from "./foundationPacks.js";

const CONTRACT_VERSION = "0.2.0";
const TOOL_VERSION = "0.1.0";

export function generateContract(brief: ProjectBrief, stackPackIds: string[] = []): ArchitectureContract {
  const stack = brief.stack ?? {};
  const stackPacks = resolveStackPacks(stack, stackPackIds);
  const stackPackIdSet = new Set(stackPacks.map((pack) => pack.id));
  const isMcpServer = stackPackIdSet.has("mcp-server");
  const hasFrontend = Boolean(stack.frontend) && !isMcpServer;
  const packDirectories = stackPacks.flatMap((pack) => pack.directories);
  const packFileRules = stackPacks.flatMap((pack) => pack.fileRules);
  const packBoundaries = stackPacks.flatMap((pack) => pack.moduleBoundaries);
  const packTestingExpectations = stackPacks.flatMap((pack) => pack.testingExpectations);
  const packAgentInstructions = stackPacks.flatMap((pack) => pack.agentInstructions);

  const contract: ArchitectureContract = {
    contractVersion: CONTRACT_VERSION,
    generatedBy: {
      tool: "architect-mcp",
      version: TOOL_VERSION,
      generatedAt: new Date().toISOString()
    },
    name: "Anti-Monolith Architecture Contract",
    purpose: brief.idea,
    stack,
    stackPacks,
    foundationPacks: listFoundationPacks(),
    archetype: inferAppArchetype(brief),
    directories: [
      ...(hasFrontend
        ? [
            { path: "src/app", purpose: "Application entrypoints, routes, and composition only.", required: true },
            { path: "src/features", purpose: "Vertical feature modules with UI, logic, and tests scoped by feature.", required: true },
            { path: "src/shared", purpose: "Reusable primitives that are genuinely shared across features.", required: true }
          ]
        : [
            { path: "src/domain", purpose: "Pure business rules and deterministic project logic.", required: true },
            { path: "src/infrastructure", purpose: "Filesystem, network, runtime, and host-specific adapters.", required: true }
          ]),
      { path: "src/server", purpose: "Backend/API handlers, services, and infrastructure adapters.", required: Boolean(stack.backend) },
      { path: "src/db", purpose: "Database schema, migrations, repositories, and seed scripts.", required: Boolean(stack.database) },
      { path: "tests", purpose: "Integration and end-to-end tests that cross feature boundaries.", required: true },
      { path: "docs", purpose: "Architecture decisions, contracts, and implementation notes.", required: true },
      { path: ".cursor/rules", purpose: "Optional editor/agent rules generated from the same contract.", required: false },
      ...packDirectories
    ],
    fileRules: [
      {
        name: "No giant implementation files",
        rule: "Application source files should stay under 300 lines unless explicitly justified.",
        severity: "warning"
      },
      {
        name: "No mixed frontend/backend concerns",
        rule: "UI components must not contain database queries, migration logic, or direct secret access.",
        severity: "error"
      },
      {
        name: "No god components",
        rule: "A component should own one view or interaction surface, not an entire application workflow.",
        severity: "warning"
      },
      {
        name: "No unbounded shared folder",
        rule: "Code only belongs in shared after at least two features need it.",
        severity: "warning"
      },
      {
        name: "No coding before contract",
        rule: "Agents should not create implementation files until the brief has passed /grill-me and the architecture contract exists.",
        severity: "error"
      },
      ...packFileRules
    ],
    moduleBoundaries: [
      "Feature modules may depend on shared primitives, but shared primitives must not import feature modules.",
      "Database access should go through repositories or services, not UI components.",
      "Route/entrypoint files should compose modules and avoid containing business logic.",
      "Tests should live near the behavior they verify unless they cross module boundaries.",
      "Frontend workflows should be split into feature-owned screens/routes, state/data hooks, and presentational components.",
      "Backend routes/controllers should validate and map transport concerns; services/use-cases own workflow orchestration.",
      "Database schema, migrations, seeds, and repository/query modules should have explicit ownership before implementation.",
      ...packBoundaries
    ],
    testingExpectations: [
      "Every feature should include at least one behavior-level test.",
      "Architecture review should run before large implementation tasks are accepted.",
      "Generated contracts should be committed under docs/architecture-contract.md.",
      "Verification commands should be named in the brief and rerun after generated scaffolding.",
      ...(brief.verification ?? []),
      ...packTestingExpectations
    ],
    agentInstructions: [
      "Before implementation, call /grill-me until there are no blocker questions.",
      "Generate or update the architecture contract before creating files.",
      "Generate or update AGENTS.md and editor rules from the same contract so future agents inherit the guardrails.",
      "Prefer small, named modules over large app-level files.",
      "Implement the scaffold plan first; then add features one workflow at a time.",
      "Run repo review after scaffolding and after major feature implementation.",
      ...packAgentInstructions
    ]
  };

  const mappedContract = applyRepoLayoutToContract(contract, brief.repoLayout);
  const validation = validateArchitectureContract(mappedContract);
  if (!validation.valid) {
    throw new Error(`Generated architecture contract failed validation: ${validation.errors.join("; ")}`);
  }
  return mappedContract;
}

export function renderContractMarkdown(contract: ArchitectureContract): string {
  const metadata = [
    `- Contract version: ${contract.contractVersion}`,
    `- Generated by: ${contract.generatedBy.tool} ${contract.generatedBy.version}`,
    `- Generated at: ${contract.generatedBy.generatedAt}`
  ].join("\n");
  const directories = contract.directories
    .map((directory) => `- \`${directory.path}\` ${directory.required ? "(required)" : "(optional)"}: ${directory.purpose}`)
    .join("\n");
  const fileRules = contract.fileRules
    .map((rule) => `- ${rule.severity.toUpperCase()}: ${rule.name}: ${rule.rule}`)
    .join("\n");
  const stackPacks = contract.stackPacks.length > 0
    ? contract.stackPacks.map((pack) => `- ${pack.name} (${pack.id}@${pack.version}): ${pack.rationale}`).join("\n")
    : "- None";
  const foundationPacks = contract.foundationPacks
    .map((pack) => `- ${pack.name} (${pack.id}): ${pack.rules.join(" ")}`)
    .join("\n");
  const repoLayout = renderRepoLayout(contract);
  const boundaries = contract.moduleBoundaries.map((boundary) => `- ${boundary}`).join("\n");
  const tests = contract.testingExpectations.map((expectation) => `- ${expectation}`).join("\n");
  const agentInstructions = contract.agentInstructions.map((instruction) => `- ${instruction}`).join("\n");
  const preCodingChecklist = [
    "- /grill-me has no blocker questions.",
    "- The selected stack packs match the requested frontend, backend, database, auth, and deployment stack.",
    "- Repo layout is mapped before applying canonical src/** paths to an existing repo.",
    "- AGENTS.md and docs/architecture-contract.md are generated or updated from this contract.",
    "- Verification commands are known before implementation starts."
  ].join("\n");
  const appGenerationGuardrails = [
    "- Frontend: split workflows into feature-owned screens/routes, data hooks, state, validation, and presentational components.",
    "- Backend: keep routes/controllers thin; move workflows into services/use-cases and integrations into adapters.",
    "- Database: keep schema, migrations, seeds, and repositories/query modules explicit and server-owned.",
    "- Repository: add directories by responsibility rather than growing one app-level file.",
    "- Agent harness: commit instructions and review gates so future agents do not rely on chat-only context."
  ].join("\n");
  const harnessSetup = [
    "- Generate AGENTS.md for repo-level coding-agent instructions.",
    "- Commit docs/architecture-contract.md as the human-readable contract.",
    "- Generate .cursor/rules/architecture.mdc when Cursor rules are useful.",
    "- Run architecture review after scaffolding and after substantial generated changes.",
    "- Treat intentional violations as accepted findings with reasons, not silent suppressions."
  ].join("\n");

  return `# ${contract.name}

## Metadata
${metadata}

## Purpose
${contract.purpose}

## Directories
${directories}

## Stack Packs
${stackPacks}

## Foundation Packs
${foundationPacks}

## App Archetype
- ${contract.archetype ?? "custom-app"}

## Repo Layout
${repoLayout}

## File Rules
${fileRules}

## Module Boundaries
${boundaries}

## Testing Expectations
${tests}

## Agent Instructions
${agentInstructions}

## Pre-Coding Checklist
${preCodingChecklist}

## App Generation Guardrails
${appGenerationGuardrails}

## Agent Harness Setup
${harnessSetup}

## Review Gate
- Run architecture review after scaffolding and large feature changes.
- Default CI gate: zero error findings, zero warning findings, minimum score 90.
- During migration, use an explicit baseline and reviewed gate thresholds instead of silently ignoring findings.

## Baseline Lifecycle
- New findings must be fixed or deliberately accepted with a reason.
- Baseline findings are known debt and should not hide new findings.
- Accepted findings need an owner-facing reason.
- Resolved findings should be removed from the baseline.
`;
}

function renderRepoLayout(contract: ArchitectureContract): string {
  const mappedInstructions = contract.agentInstructions.filter((instruction) => instruction.includes("repo layout mapping"));
  if (mappedInstructions.length === 0) return "- Use the contract directory paths as written.";
  return "- This contract has been adapted to the target repo layout. Interpret stack-pack paths through the mapped directories above.";
}
