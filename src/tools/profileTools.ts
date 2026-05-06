import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { getRepoProfile, listRepoProfiles } from "../domain/repoProfiles.js";
import { safeJsonResponse } from "./responses.js";
import { genericObjectOutputSchema } from "./schemas.js";

export function registerProfileTools(server: McpServer): void {
  server.registerTool(
    "list_repo_profiles",
    {
      title: "List Repo Profiles",
      description: "List reusable repo profiles for common app/repo layouts.",
      inputSchema: {},
      outputSchema: genericObjectOutputSchema
    },
    async () => safeJsonResponse(() => ({ profiles: listRepoProfiles() }))
  );

  server.registerTool(
    "get_repo_profile",
    {
      title: "Get Repo Profile",
      description: "Get a reusable repo profile by id.",
      inputSchema: {
        profileId: z.string()
      },
      outputSchema: genericObjectOutputSchema
    },
    async ({ profileId }) => safeJsonResponse(() => ({
      profile: getRepoProfile(profileId)
    }))
  );
}
