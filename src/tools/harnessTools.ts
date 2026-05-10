import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { classifyAmbiguityRisk, createPreEditContract, interpretImplementationIntent, recordAssumption, reviewImplementationAgainstContract } from "../domain/harness.js";
import { loadTriggeredStackGuidance } from "../domain/harnessGuidance.js";
import type { AssumptionLedgerEntry, HarnessIntentInput, HarnessIntentResult, PreEditContract } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { assumptionInputSchema, genericObjectOutputSchema, harnessIntentInputSchema, harnessIntentResultSchema, implementationReviewSchema, preEditContractSchema, triggeredStackGuidanceInputSchema } from "./schemas.js";

export function registerHarnessTools(server: McpServer): void {
  server.registerTool(
    "interpret_implementation_intent",
    {
      title: "Interpret Implementation Intent",
      description: "Interpret vague novice implementation requests and return a guided-yolo decision before code edits.",
      inputSchema: {
        input: harnessIntentInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => interpretImplementationIntent(input as HarnessIntentInput))
  );

  server.registerTool(
    "classify_ambiguity_risk",
    {
      title: "Classify Ambiguity Risk",
      description: "Classify whether an agent can proceed, should log assumptions, must confirm, or must block.",
      inputSchema: {
        input: harnessIntentInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => classifyAmbiguityRisk(input as HarnessIntentInput))
  );

  server.registerTool(
    "create_pre_edit_contract",
    {
      title: "Create Pre-Edit Contract",
      description: "Create a small implementation-intent contract before risky edits, either from a prior intent result or raw intent input.",
      inputSchema: {
        intent: harnessIntentResultSchema.optional(),
        input: harnessIntentInputSchema.optional(),
        likelyFiles: preEditContractSchema.shape.likelyFiles.optional(),
        verificationChecks: preEditContractSchema.shape.verificationChecks.optional()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ intent, input, likelyFiles, verificationChecks }) => safeJsonResponse(() => {
      const resolvedIntent = intent ?? (input ? interpretImplementationIntent(input as HarnessIntentInput) : undefined);
      if (!resolvedIntent) {
        throw new Error("create_pre_edit_contract requires either intent or input.");
      }
      return {
        intent: resolvedIntent,
        contract: createPreEditContract({
          intent: resolvedIntent as HarnessIntentResult,
          likelyFiles,
          verificationChecks
        })
      };
    })
  );

  server.registerTool(
    "review_implementation_against_contract",
    {
      title: "Review Implementation Against Contract",
      description: "Review changed files or a proposed plan against a pre-edit implementation contract.",
      inputSchema: {
        input: implementationReviewSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => reviewImplementationAgainstContract(input as {
      contract: PreEditContract;
      changedFiles?: Parameters<typeof reviewImplementationAgainstContract>[0]["changedFiles"];
      proposedPlan?: Parameters<typeof reviewImplementationAgainstContract>[0]["proposedPlan"];
      verification?: Parameters<typeof reviewImplementationAgainstContract>[0]["verification"];
    }))
  );

  server.registerTool(
    "record_assumption",
    {
      title: "Record Assumption",
      description: "Return a stateless assumption ledger entry for guided-yolo agent runs.",
      inputSchema: {
        assumption: assumptionInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ assumption }) => safeJsonResponse(() => recordAssumption(assumption as AssumptionLedgerEntry))
  );

  server.registerTool(
    "load_triggered_stack_guidance",
    {
      title: "Load Triggered Stack Guidance",
      description: "Load local ingested llms.txt guidance relevant to a vague request and stack hints.",
      inputSchema: {
        input: triggeredStackGuidanceInputSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ input }) => safeJsonResponse(() => loadTriggeredStackGuidance(input))
  );
}
