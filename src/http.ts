#!/usr/bin/env node
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import express from "express";
import type { Request, Response } from "express";
import { createArchitectServer } from "./server/createArchitectServer.js";

const port = Number.parseInt(process.env.PORT ?? "3000", 10);
const host = process.env.HOST ?? "0.0.0.0";
const jsonBodyLimit = process.env.JSON_BODY_LIMIT ?? "10mb";
const allowedHosts = new Set(process.env.ALLOWED_HOSTS?.split(",")
  .map((entry) => entry.trim())
  .filter(Boolean) ?? []);

const app = express();
app.use(validateHostHeader);
app.use(express.json({ limit: jsonBodyLimit }));

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

  try {
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);

    res.on("close", () => {
      void transport.close();
      void server.close();
    });
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

app.listen(port, host, (error?: Error) => {
  if (error) {
    console.error("Failed to start architect-mcp HTTP server:", error);
    process.exit(1);
  }

  console.log(`architect-mcp HTTP server listening on http://${host}:${port}/mcp`);
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
