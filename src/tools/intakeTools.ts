import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { getNextIntakeQuestion, grillMe, grillProjectBrief } from "../domain/intake.js";
import type { ProjectBrief } from "../domain/types.js";
import { safeJsonResponse } from "./responses.js";
import { genericObjectOutputSchema, grillMeOutputSchema, intakeAnswerSchema, projectBriefSchema } from "./schemas.js";

export function registerIntakeTools(server: McpServer): void {
  server.registerTool(
    "start_project_intake",
    {
      title: "Start Project Intake",
      description: "Ask the next most important question before an agent starts building.",
      inputSchema: {
        brief: projectBriefSchema
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ brief }) => safeJsonResponse(() => {
      const nextQuestion = getNextIntakeQuestion(brief as ProjectBrief);
      return { nextQuestion };
    })
  );

  server.registerTool(
    "grill_project_brief",
    {
      title: "Grill Project Brief",
      description: "Stress-test a project brief before implementation. Kept as a compatibility alias for grill_me.",
      inputSchema: {
        brief: projectBriefSchema
      },
      outputSchema: grillMeOutputSchema
    },
    async ({ brief }) => safeJsonResponse(() => grillProjectBrief(brief as ProjectBrief))
  );

  server.registerTool(
    "grill_me",
    {
      title: "/grill-me",
      description: "Run the full local-first architecture intake loop: pressure-test the brief, select stack packs, and optionally generate contract artifacts.",
      inputSchema: {
        brief: projectBriefSchema,
        stackPackIds: z.array(z.string()).optional(),
        includeContract: z.boolean().default(true),
        includeArtifacts: z.boolean().default(false),
        answer: intakeAnswerSchema.optional()
      },
      outputSchema: grillMeOutputSchema
    },
    async ({ brief, stackPackIds, includeContract, includeArtifacts, answer }) => safeJsonResponse(() => grillMe(brief as ProjectBrief, {
      stackPackIds,
      includeContract,
      includeArtifacts,
      answer: answer as never
    }))
  );

  server.registerTool(
    "continue_grill_me",
    {
      title: "Continue /grill-me",
      description: "Apply one intake answer to a brief and rerun /grill-me.",
      inputSchema: {
        brief: projectBriefSchema,
        answer: intakeAnswerSchema,
        stackPackIds: z.array(z.string()).optional(),
        includeContract: z.boolean().default(true),
        includeArtifacts: z.boolean().default(false)
      },
      outputSchema: grillMeOutputSchema
    },
    async ({ brief, answer, stackPackIds, includeContract, includeArtifacts }) => safeJsonResponse(() => grillMe(brief as ProjectBrief, {
      stackPackIds,
      includeContract,
      includeArtifacts,
      answer: answer as never
    }))
  );
}
