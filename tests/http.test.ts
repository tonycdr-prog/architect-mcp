import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { request } from "node:http";
import type { AddressInfo } from "node:net";
import { createHttpApp } from "../src/http.js";

describe("HTTP transport entrypoint", () => {
  it("returns JSON-RPC parse errors for malformed MCP JSON", async () => {
    const server = await listen();
    try {
      const response = await httpRequest(server, {
        method: "POST",
        path: "/mcp",
        headers: {
          "content-type": "application/json",
          host: "127.0.0.1"
        },
        body: "{not-json"
      });

      assert.equal(response.status, 400);
      assert.equal(JSON.parse(response.body).error.code, -32700);
    } finally {
      await close(server);
    }
  });

  it("rejects disallowed Host and Origin headers", async () => {
    const server = await listen({
      allowedHosts: new Set(["127.0.0.1"]),
      allowedOrigins: new Set(["http://127.0.0.1"])
    });
    try {
      const badHost = await httpRequest(server, {
        method: "GET",
        path: "/health",
        headers: {
          host: "evil.test"
        }
      });
      const badOrigin = await httpRequest(server, {
        method: "GET",
        path: "/health",
        headers: {
          host: "127.0.0.1",
          origin: "https://evil.test"
        }
      });
      const good = await httpRequest(server, {
        method: "GET",
        path: "/health",
        headers: {
          host: "127.0.0.1",
          origin: "http://127.0.0.1"
        }
      });

      assert.equal(badHost.status, 403);
      assert.equal(badOrigin.status, 403);
      assert.equal(good.status, 200);
    } finally {
      await close(server);
    }
  });
});

async function listen(options: Parameters<typeof createHttpApp>[0] = {}) {
  const app = createHttpApp(options);
  return await new Promise<ReturnType<typeof app.listen>>((resolve) => {
    const server = app.listen(0, "127.0.0.1", () => resolve(server));
  });
}

async function close(server: Awaited<ReturnType<typeof listen>>): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    server.close((error) => error ? reject(error) : resolve());
  });
}

async function httpRequest(server: Awaited<ReturnType<typeof listen>>, input: {
  method: string;
  path: string;
  headers?: Record<string, string>;
  body?: string;
}): Promise<{ status: number; body: string }> {
  const address = server.address() as AddressInfo;
  return await new Promise((resolve, reject) => {
    const req = request({
      host: "127.0.0.1",
      port: address.port,
      method: input.method,
      path: input.path,
      headers: input.headers
    }, (res) => {
      let body = "";
      res.setEncoding("utf8");
      res.on("data", (chunk) => {
        body += chunk;
      });
      res.on("end", () => {
        resolve({
          status: res.statusCode ?? 0,
          body
        });
      });
    });
    req.on("error", reject);
    if (input.body) req.write(input.body);
    req.end();
  });
}
