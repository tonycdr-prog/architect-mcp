#!/usr/bin/env node
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import express from "express";
import type { ErrorRequestHandler, NextFunction, Request, Response } from "express";
import { fileURLToPath } from "node:url";
import { createArchitectServer } from "./server/createArchitectServer.js";

const port = parsePort(process.env.PORT ?? "3000");
const host = process.env.HOST ?? "0.0.0.0";
const jsonBodyLimit = process.env.JSON_BODY_LIMIT ?? "10mb";
const configuredAllowedHosts = parseCsvSet(process.env.ALLOWED_HOSTS);
const configuredAllowedOrigins = parseCsvSet(process.env.ALLOWED_ORIGINS);

type HttpMcpServer = {
  connect(transport: unknown): Promise<void>;
  close(): Promise<void>;
};

type HttpTransport = {
  handleRequest(req: Request, res: Response, body: unknown): Promise<void>;
  close(): Promise<void>;
};

export type HttpAppOptions = {
  allowedHosts?: Set<string>;
  allowedOrigins?: Set<string>;
  jsonBodyLimit?: string;
  serverFactory?: () => HttpMcpServer;
  transportFactory?: () => HttpTransport;
};

export function createHttpApp(options: HttpAppOptions = {}) {
  const allowedHosts = options.allowedHosts ?? configuredAllowedHosts;
  const allowedOrigins = options.allowedOrigins ?? configuredAllowedOrigins;
  const app = express();
  app.use(validateHostHeader(allowedHosts));
  app.use(validateOriginHeader(allowedHosts, allowedOrigins));
  app.use(express.json({ limit: options.jsonBodyLimit ?? jsonBodyLimit }));
  app.use(jsonParserErrorHandler);

  app.get("/health", (_req: Request, res: Response) => {
    res.status(200).json({
      ok: true,
      name: "architect-mcp",
      mode: "stateless-http"
    });
  });

  app.post("/mcp", async (req: Request, res: Response) => {
    const server = options.serverFactory?.() ?? createArchitectServer({
      enableLocalWorkspaceTool: false
    });

    const transport = options.transportFactory?.() ?? new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined
    });

    let closed = false;
    const cleanup = () => {
      if (closed) return;
      closed = true;
      void transport.close();
      void server.close();
    };
    res.once("close", cleanup);

    try {
      await server.connect(transport);
      await transport.handleRequest(req, res, req.body);
    } catch (error) {
      cleanup();
      console.error("Error handling MCP request:", error);
      if (!res.headersSent) {
        res.status(500).json(jsonRpcError(-32603, "Internal server error"));
      }
    }
  });

  app.get("/mcp", methodNotAllowed);
  app.delete("/mcp", methodNotAllowed);

  return app;
}

export function startHttpServer(options: HttpAppOptions = {}) {
  const app = createHttpApp(options);
  const server = app.listen(port, host, () => {
    console.log(`architect-mcp HTTP server listening on http://${host}:${port}/mcp`);
  });
  server.on("error", (error) => {
    console.error("Failed to start architect-mcp HTTP server:", error);
    process.exit(1);
  });
  return server;
}

export function parsePort(value: string): number {
  if (!/^\d+$/.test(value)) {
    throw new Error(`Invalid PORT value "${value}". Expected an integer from 0 to 65535.`);
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isInteger(parsed) || parsed < 0 || parsed > 65535) {
    throw new Error(`Invalid PORT value "${value}". Expected an integer from 0 to 65535.`);
  }
  return parsed;
}

function validateHostHeader(allowedHosts: Set<string>) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const hostHeader = req.headers.host;
    const hostName = hostFromHeader(hostHeader);
    if (isAllowedHost(hostName, allowedHosts)) {
      next();
      return;
    }

    res.status(403).json({
      error: "Forbidden host header"
    });
  };
}

function validateOriginHeader(allowedHosts: Set<string>, allowedOrigins: Set<string>) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const origin = req.headers.origin;
    if (!origin) {
      next();
      return;
    }

    try {
      const parsed = new URL(origin);
      const normalizedOrigin = parsed.origin.toLowerCase();
      const originAllowed = allowedOrigins.size > 0
        ? allowedOrigins.has(origin.toLowerCase()) || allowedOrigins.has(normalizedOrigin)
        : isAllowedHost(parsed.hostname, allowedHosts);
      if (originAllowed) {
        next();
        return;
      }
    } catch {
      // Fall through to the JSON error below.
    }

    res.status(403).json({
      error: "Forbidden origin header"
    });
  };
}

const jsonParserErrorHandler: ErrorRequestHandler = (error, _req, res, next) => {
  if (!isBodyParserError(error)) {
    next(error);
    return;
  }

  res.status(error.statusCode ?? error.status ?? 400).json(jsonRpcError(-32700, "Parse error"));
};

function methodNotAllowed(_req: Request, res: Response): void {
  res.status(405).json(jsonRpcError(-32000, "Method not allowed."));
}

function isAllowedHost(hostHeader: string | undefined, allowedHosts: Set<string>): boolean {
  const hostName = hostHeader?.toLowerCase();
  if (!hostName) return false;
  if (allowedHosts.size > 0) return allowedHosts.has(hostName);
  return hostName === "localhost" || hostName === "127.0.0.1" || hostName === "::1";
}

function hostFromHeader(hostHeader: string | undefined): string | undefined {
  if (!hostHeader) return undefined;
  if (hostHeader.startsWith("[")) return hostHeader.match(/^\[([^\]]+)\]/)?.[1]?.toLowerCase();
  return hostHeader.split(":")[0]?.toLowerCase();
}

function parseCsvSet(value: string | undefined): Set<string> {
  return new Set(value?.split(",")
    .map((entry) => entry.trim().toLowerCase())
    .filter(Boolean) ?? []);
}

function jsonRpcError(code: number, message: string) {
  return {
    jsonrpc: "2.0",
    error: {
      code,
      message
    },
    id: null
  };
}

function isBodyParserError(error: unknown): error is { status?: number; statusCode?: number; type?: string } {
  return typeof error === "object" &&
    error !== null &&
    ("status" in error || "statusCode" in error) &&
    "type" in error;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  startHttpServer();
}
