#!/usr/bin/env node
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import express from "express";
import type { NextFunction, Request, Response } from "express";
import { createArchitectServer } from "./server/createArchitectServer.js";

const port = Number.parseInt(process.env.PORT ?? "3000", 10);
const host = process.env.HOST ?? "0.0.0.0";
const jsonBodyLimit = process.env.JSON_BODY_LIMIT ?? "10mb";
const allowedHosts = new Set(process.env.ALLOWED_HOSTS?.split(",")
  .map((entry) => entry.trim())
  .filter(Boolean) ?? []);
const allowedOrigins = new Set(process.env.ALLOWED_ORIGINS?.split(",")
  .map((entry) => entry.trim())
  .filter(Boolean) ?? []);

const app = express();
app.use(validateHostHeader);
app.use(validateOriginHeader);
app.use(express.json({ limit: jsonBodyLimit }));
app.use(handleJsonParseError);

app.get("/health", (_req: Request, res: Response) => {
  res.status(200).json({
    ok: true,
    name: "architect-mcp",
    mode: "stateless-http"
  });
});

app.post("/mcp", async (req: Request, res: Response) => {
  const server = createArchitectServer({
    enableLocalWorkspaceTool: false
  });

  const transport = new StreamableHTTPServerTransport({
    sessionIdGenerator: undefined
  });
  let closed = false;
  const close = () => {
    if (closed) return;
    closed = true;
    void transport.close();
    void server.close();
  };
  res.once("close", close);

  try {
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);
  } catch (error) {
    console.error("Error handling MCP request:", error);
    if (!res.headersSent) {
      res.status(500).json({
        jsonrpc: "2.0",
        error: {
          code: -32603,
          message: "Internal server error"
        },
        id: null
      });
    }
    close();
  }
});

app.get("/mcp", (_req: Request, res: Response) => {
  res.status(405).json({
    jsonrpc: "2.0",
    error: {
      code: -32000,
      message: "Method not allowed."
    },
    id: null
  });
});

app.delete("/mcp", (_req: Request, res: Response) => {
  res.status(405).json({
    jsonrpc: "2.0",
    error: {
      code: -32000,
      message: "Method not allowed."
    },
    id: null
  });
});

const httpServer = app.listen(port, host, () => {
  console.log(`architect-mcp HTTP server listening on http://${host}:${port}/mcp`);
});
httpServer.on("error", (error) => {
  console.error("Failed to start architect-mcp HTTP server:", error);
  process.exit(1);
});

function validateHostHeader(req: Request, res: Response, next: () => void): void {
  if (allowedHosts.size === 0) {
    next();
    return;
  }

  const hostHeader = req.headers.host?.split(":")[0];
  if (hostHeader && allowedHosts.has(hostHeader)) {
    next();
    return;
  }

  res.status(403).json({
    error: "Forbidden host header"
  });
}

function validateOriginHeader(req: Request, res: Response, next: () => void): void {
  const origin = req.headers.origin;
  if (!origin) {
    next();
    return;
  }

  if (allowedOrigins.has(origin)) {
    next();
    return;
  }

  try {
    const parsed = new URL(origin);
    if (allowedHosts.has(parsed.hostname) || (allowedHosts.size === 0 && isLoopbackHost(parsed.hostname))) {
      next();
      return;
    }
  } catch {
    // Reject malformed Origin values below.
  }

  res.status(403).json({
    error: "Forbidden origin header"
  });
}

function isLoopbackHost(value: string): boolean {
  return value === "localhost" || value === "127.0.0.1" || value === "::1";
}

function handleJsonParseError(error: unknown, _req: Request, res: Response, next: NextFunction): void {
  if (error instanceof SyntaxError && "body" in error) {
    res.status(400).json({
      jsonrpc: "2.0",
      error: {
        code: -32700,
        message: "Parse error"
      },
      id: null
    });
    return;
  }

  if (isPayloadTooLargeError(error)) {
    res.status(413).json({
      jsonrpc: "2.0",
      error: {
        code: -32000,
        message: "Request payload too large"
      },
      id: null
    });
    return;
  }

  next(error);
}

function isPayloadTooLargeError(error: unknown): boolean {
  return Boolean(error && typeof error === "object" && "type" in error && (error as { type?: string }).type === "entity.too.large");
}
