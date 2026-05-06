import type { ReviewViolation } from "../domain/types.js";

export function jsonResponse(data: unknown) {
  return {
    structuredContent: data as Record<string, unknown>,
    content: [
      {
        type: "text" as const,
        text: JSON.stringify(data, null, 2)
      }
    ]
  };
}

export function errorResponse(code: string, message: string, details: unknown = undefined) {
  return {
    isError: true,
    structuredContent: {
      error: {
        code,
        message,
        details
      }
    },
    content: [
      {
        type: "text" as const,
        text: JSON.stringify({
          error: {
            code,
            message,
            details
          }
        }, null, 2)
      }
    ]
  };
}

export async function safeJsonResponse(operation: () => unknown | Promise<unknown>) {
  try {
    return jsonResponse(await operation());
  } catch (error) {
    return errorResponse(
      "ARCHITECT_TOOL_ERROR",
      error instanceof Error ? error.message : "Unexpected architect-mcp tool error.",
      undefined
    );
  }
}

export function summarizeViolations(violations: Pick<ReviewViolation, "severity">[]) {
  return {
    errors: violations.filter((violation) => violation.severity === "error").length,
    warnings: violations.filter((violation) => violation.severity === "warning").length
  };
}
