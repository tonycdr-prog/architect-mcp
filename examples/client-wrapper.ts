type McpCall = (tool: string, args: Record<string, unknown>) => Promise<unknown>;

export async function guidedYoloEditLoop(callTool: McpCall, request: string, changedFiles: unknown[]) {
  const intent = await callTool("interpret_implementation_intent", {
    input: {
      request,
      mode: "guided-yolo"
    }
  });

  const decision = (intent as { decision?: string }).decision;
  if (decision === "confirm_before_edit" || decision === "block_until_clarified") {
    return {
      status: "needs-confirmation",
      intent
    };
  }

  const contractResult = await callTool("create_pre_edit_contract", {
    intent,
    likelyFiles: changedFiles
  });
  const contract = (contractResult as { contract?: unknown }).contract;

  const drift = await callTool("review_implementation_against_contract", {
    input: {
      contract,
      changedFiles,
      verification: [{ check: "npm test", status: "not_run" }]
    }
  });

  return {
    status: "reviewed",
    intent,
    contract,
    drift
  };
}

export async function reviewFinalMessage(callTool: McpCall, response: string) {
  return callTool("review_agent_final_response", {
    request: {
      response,
      requiredChecks: ["npm run typecheck", "npm test", "npm run build"]
    }
  });
}
