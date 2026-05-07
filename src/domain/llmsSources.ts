import { createHash } from "node:crypto";
import { lookup } from "node:dns/promises";
import { existsSync, readFileSync } from "node:fs";
import { isIP } from "node:net";
import { dirname, isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { IngestedLlmsSource, LlmsSource, LlmsSourceSnapshot } from "./types.js";

const DEFAULT_FETCH_TIMEOUT_MS = 10_000;
const DEFAULT_MAX_BYTES = 300_000;

export const KNOWN_LLMS_SOURCES: LlmsSource[] = [
  { id: "nextjs", stack: "Next.js", category: "frontend", url: "https://nextjs.org/docs/llms.txt", fullUrl: "https://nextjs.org/docs/llms-full.txt", priority: "high", notes: "Primary source for Next.js App Router and full-stack React guidance." },
  { id: "react", stack: "React", category: "frontend", url: "https://react.dev/llms.txt", priority: "high", notes: "Official React docs index with component, hooks, purity, and server component sections." },
  { id: "vite", stack: "Vite", category: "frontend", url: "https://vite.dev/llms.txt", priority: "high", notes: "Official Vite docs index for React/Vite and build-tool guidance." },
  { id: "expo", stack: "Expo React Native", category: "frontend", url: "https://docs.expo.dev/llms.txt", priority: "high", notes: "Official Expo docs with current guidance on CNG, development builds, routing, databases, auth, and native capabilities." },
  { id: "supabase", stack: "Supabase", category: "database", url: "https://supabase.com/llms.txt", fullUrl: "https://supabase.com/llms-full.txt", priority: "high", notes: "Official Supabase docs index with guides, JS reference, CLI reference, auth, storage, realtime, and database docs." },
  { id: "vercel", stack: "Vercel", category: "deployment", url: "https://vercel.com/llms.txt", fullUrl: "https://vercel.com/docs/llms-full.txt", priority: "high", notes: "Official Vercel platform docs for deployment, supported frameworks, AI SDK, protection, and production readiness." },
  { id: "auth0", stack: "Auth0", category: "auth", url: "https://auth0.com/llms.txt", fullUrl: "https://auth0.com/docs/llms-full.txt", priority: "high", notes: "Official Auth0 docs index for authentication, authorization, SDKs, agent skills, and secure actions." },
  { id: "stripe", stack: "Stripe", category: "payments", url: "https://docs.stripe.com/llms.txt", priority: "high", notes: "Official Stripe docs index and plain-text workflow for payments, billing, and integration docs." },
  { id: "openai", stack: "OpenAI Platform", category: "ai", url: "https://platform.openai.com/docs/llms.txt", fullUrl: "https://developers.openai.com/api/docs/llms-full.txt", priority: "high", notes: "Official OpenAI developer docs for agents, tools, API usage, evals, safety, and production concerns." },
  { id: "ai-sdk", stack: "Vercel AI SDK", category: "ai", url: "https://ai-sdk.dev/llms.txt", priority: "high", notes: "AI SDK docs for TypeScript AI apps across React, Next.js, Vue, Svelte, and Node." },
  { id: "prisma", stack: "Prisma", category: "orm", url: "https://www.prisma.io/llms.txt", priority: "medium", notes: "Prisma ORM docs index for schema, migrations, query client, and database access patterns." },
  { id: "drizzle", stack: "Drizzle ORM", category: "orm", url: "https://orm.drizzle.team/llms.txt", priority: "medium", notes: "Drizzle ORM docs index for SQL-like schema/query/migration patterns." },
  { id: "railway", stack: "Railway", category: "deployment", url: "https://docs.railway.com/llms.txt", fullUrl: "https://docs.railway.com/llms-full.txt", priority: "medium", notes: "Railway deployment, production readiness, networking, observability, and database platform docs." },
  { id: "cloudflare", stack: "Cloudflare", category: "deployment", url: "https://developers.cloudflare.com/llms.txt", priority: "medium", notes: "Cloudflare developer docs for Workers, Pages, storage, and edge deployment patterns." },
  { id: "anthropic", stack: "Anthropic", category: "ai", url: "https://docs.anthropic.com/llms.txt", priority: "medium", notes: "Anthropic developer docs for Claude APIs and AI app integration." },
  { id: "tanstack", stack: "TanStack", category: "frontend", url: "https://tanstack.com/llms.txt", priority: "medium", notes: "TanStack ecosystem docs for router, query, table, forms, and app architecture primitives." },
  { id: "shadcn-ui", stack: "shadcn/ui", category: "frontend", url: "https://ui.shadcn.com/llms.txt", priority: "medium", notes: "shadcn/ui component registry docs for UI composition and component installation." }
  ,
  { id: "astro", stack: "Astro", category: "frontend", url: "https://docs.astro.build/llms.txt", fullUrl: "https://docs.astro.build/llms-full.txt", priority: "high", notes: "Astro docs for island architecture, content collections, server-first design, integrations, and deployment." },
  { id: "svelte", stack: "Svelte/SvelteKit", category: "frontend", url: "https://svelte.dev/llms.txt", fullUrl: "https://svelte.dev/llms-full.txt", priority: "high", notes: "Svelte and SvelteKit docs for compiler-driven UI, app routing, server load, and deployment." },
  { id: "hono", stack: "Hono", category: "backend", url: "https://hono.dev/llms.txt", fullUrl: "https://hono.dev/llms-full.txt", priority: "high", notes: "Hono docs for web-standard HTTP APIs across edge, serverless, and Node runtimes." },
  { id: "langchain", stack: "LangChain", category: "ai", url: "https://docs.langchain.com/llms.txt", priority: "high", notes: "LangChain and LangSmith docs for agents, deployment, observability, evaluation, and MCP-enabled workflows." },
  { id: "vitest", stack: "Vitest", category: "frontend", url: "https://vitest.dev/llms.txt", priority: "medium", notes: "Vitest docs for unit testing, browser mode, coverage, mocks, and test configuration." },
  { id: "zod", stack: "Zod", category: "backend", url: "https://zod.dev/llms.txt", priority: "medium", notes: "Zod docs for schema validation, parsing, JSON schema, codecs, and typed data boundaries." },
  { id: "trpc", stack: "tRPC", category: "backend", url: "https://trpc.io/llms.txt", priority: "medium", notes: "tRPC docs for type-safe API routers, procedures, clients, and full-stack TypeScript boundaries." },
  { id: "zustand", stack: "Zustand", category: "frontend", url: "https://zustand.docs.pmnd.rs/llms.txt", priority: "medium", notes: "Zustand docs for small state stores and React state management boundaries." },
  { id: "better-auth", stack: "Better Auth", category: "auth", url: "https://better-auth.com/llms.txt", priority: "medium", notes: "Better Auth docs for authentication flows, sessions, plugins, and server/client auth integration." },
  { id: "convex", stack: "Convex", category: "database", url: "https://www.convex.dev/llms.txt", priority: "medium", notes: "Convex docs for reactive backend, database functions, auth, scheduling, and generated app architecture." },
  { id: "posthog", stack: "PostHog", category: "platform", url: "https://posthog.com/llms.txt", priority: "medium", notes: "PostHog docs for product analytics, feature flags, experiments, session replay, and observability." },
  { id: "launchdarkly", stack: "LaunchDarkly", category: "platform", url: "https://launchdarkly.com/docs/llms.txt", priority: "medium", notes: "LaunchDarkly docs for feature flagging, environments, SDKs, targeting, and rollout safety." },
  { id: "render", stack: "Render", category: "deployment", url: "https://render.com/llms.txt", priority: "medium", notes: "Render docs for deploying web services, background workers, cron jobs, databases, and production configuration." },
  { id: "appwrite", stack: "Appwrite", category: "database", url: "https://appwrite.io/llms.txt", priority: "medium", notes: "Appwrite docs for backend-as-a-service auth, databases, storage, functions, and realtime." },
  { id: "llamaindex", stack: "LlamaIndex", category: "ai", url: "https://developers.llamaindex.ai/llms.txt", priority: "medium", notes: "LlamaIndex docs for RAG, agents, document parsing, indexing, retrieval, and MCP search." },
  { id: "mistral", stack: "Mistral AI", category: "ai", url: "https://docs.mistral.ai/llms.txt", priority: "medium", notes: "Mistral AI docs for model APIs, agents, embeddings, fine-tuning, and deployment workflows." },
  { id: "replicate", stack: "Replicate", category: "ai", url: "https://replicate.com/llms.txt", priority: "medium", notes: "Replicate docs for running models via API, predictions, deployments, and model packaging." },
  { id: "cohere", stack: "Cohere", category: "ai", url: "https://docs.cohere.com/llms.txt", priority: "medium", notes: "Cohere docs for embeddings, reranking, generation, and enterprise AI APIs." },
  { id: "weaviate", stack: "Weaviate", category: "database", url: "https://weaviate.io/llms.txt", priority: "medium", notes: "Weaviate docs for vector database architecture, schema, retrieval, and RAG integrations." },
  { id: "pinecone", stack: "Pinecone", category: "database", url: "https://docs.pinecone.io/llms.txt", priority: "medium", notes: "Pinecone docs for vector indexes, namespaces, metadata filtering, embeddings, and RAG data boundaries." },
  { id: "huggingface-hub", stack: "Hugging Face Hub", category: "ai", url: "https://huggingface.co/docs/hub/llms.txt", priority: "medium", notes: "Hugging Face Hub docs for model, dataset, and space hosting workflows." },
  { id: "huggingface-transformers", stack: "Hugging Face Transformers", category: "ai", url: "https://huggingface.co/docs/transformers/llms.txt", priority: "medium", notes: "Transformers docs for model usage, pipelines, training, tokenizers, and inference patterns." },
  { id: "mongodb", stack: "MongoDB", category: "database", url: "https://www.mongodb.com/docs/llms.txt", priority: "medium", notes: "MongoDB docs for Atlas, schema modeling, querying, indexing, vector search, and application data boundaries." }
];

export function discoverLlmsSources(query?: { stack?: string; category?: LlmsSource["category"]; priority?: LlmsSource["priority"] }): LlmsSource[] {
  return KNOWN_LLMS_SOURCES.filter((source) =>
    (!query?.stack || source.stack.toLowerCase().includes(query.stack.toLowerCase()) || source.id.includes(query.stack.toLowerCase())) &&
    (!query?.category || source.category === query.category) &&
    (!query?.priority || source.priority === query.priority)
  );
}

export function listIngestedLlmsSources(): { generatedAt?: string; sources: IngestedLlmsSource[] } {
  try {
    const index = JSON.parse(readFileSync(resolveIngestedIndexPath(), "utf8")) as { generatedAt?: string; sources?: IngestedLlmsSource[] };
    return {
      generatedAt: index.generatedAt,
      sources: index.sources ?? []
    };
  } catch {
    return {
      sources: []
    };
  }
}

export function readIngestedLlmsSnapshot(relativeOrAbsolutePath: string): string {
  const candidates = isAbsolute(relativeOrAbsolutePath)
    ? [relativeOrAbsolutePath]
    : [
      resolve(process.cwd(), relativeOrAbsolutePath),
      resolve(dirname(fileURLToPath(import.meta.url)), "../../", relativeOrAbsolutePath),
      resolve(dirname(fileURLToPath(import.meta.url)), "../../../", relativeOrAbsolutePath)
    ];
  const existing = candidates.find((candidate) => existsSync(candidate));
  if (!existing) {
    throw new Error(`Could not resolve ingested llms.txt snapshot ${relativeOrAbsolutePath}. Tried: ${candidates.join(", ")}`);
  }
  return readFileSync(existing, "utf8");
}

export async function fetchLlmsSource(sourceIdOrUrl: string, options: { preferFull?: boolean; maxBytes?: number; timeoutMs?: number } = {}): Promise<LlmsSourceSnapshot> {
  const source = resolveLlmsSource(sourceIdOrUrl, options.preferFull);
  const maxBytes = options.maxBytes ?? DEFAULT_MAX_BYTES;
  const timeoutMs = options.timeoutMs ?? DEFAULT_FETCH_TIMEOUT_MS;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetchWithValidatedRedirects(new URL(source.url), controller.signal);
    if (!response.ok) {
      throw new Error(`Could not fetch ${source.url}: ${response.status} ${response.statusText}`);
    }
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.includes("text/html")) {
      throw new Error(`Fetched ${source.url} but received HTML instead of llms.txt content.`);
    }
    const contentLength = response.headers.get("content-length");
    if (contentLength && Number.parseInt(contentLength, 10) > maxBytes) {
      throw new Error(`Fetched ${source.url} exceeds maxBytes (${maxBytes}).`);
    }

    const content = await readResponseTextWithLimit(response, maxBytes, source.url, controller.signal);
    return {
      source,
      fetchedAt: new Date().toISOString(),
      sha256: createHash("sha256").update(content).digest("hex"),
      bytes: Buffer.byteLength(content),
      contentType: contentType || undefined,
      content
    };
  } catch (error) {
    if (controller.signal.aborted) {
      throw new Error(`Timed out fetching ${source.url} after ${timeoutMs}ms.`);
    }
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

function resolveLlmsSource(sourceIdOrUrl: string, preferFull?: boolean): LlmsSource {
  const known = KNOWN_LLMS_SOURCES.find((source) => source.id === sourceIdOrUrl);
  if (known) {
    return {
      ...known,
      url: preferFull && known.fullUrl ? known.fullUrl : known.url
    };
  }

  const url = validateCallerProvidedLlmsUrl(sourceIdOrUrl);
  return {
    id: slugify(url.hostname),
    stack: url.hostname,
    category: "platform",
    url: url.toString(),
    priority: "low",
    notes: "Caller-provided llms.txt URL."
  };
}

function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "llms-source";
}

function resolveIngestedIndexPath(): string {
  const candidates = [
    resolve(process.cwd(), "stack-sources/ingested/index.json"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../stack-sources/ingested/index.json"),
    resolve(dirname(fileURLToPath(import.meta.url)), "../../../stack-sources/ingested/index.json")
  ];

  const existing = candidates.find((candidate) => existsSync(candidate));
  return existing ?? candidates[0];
}

async function readResponseTextWithLimit(response: Response, maxBytes: number, sourceUrl: string, signal: AbortSignal): Promise<string> {
  const reader = response.body?.getReader();
  if (!reader) {
    const text = await withAbort(response.text(), signal);
    const bytes = Buffer.byteLength(text);
    if (bytes > maxBytes) throw new Error(`Fetched ${sourceUrl} exceeds maxBytes (${maxBytes}).`);
    return text;
  }

  const chunks: Uint8Array[] = [];
  let totalBytes = 0;

  while (true) {
    const { done, value } = await withAbort(reader.read(), signal);
    if (done) break;
    if (!value) continue;
    totalBytes += value.byteLength;
    if (totalBytes > maxBytes) {
      await reader.cancel();
      throw new Error(`Fetched ${sourceUrl} exceeds maxBytes (${maxBytes}).`);
    }
    chunks.push(value);
  }

  return Buffer.concat(chunks).toString("utf8");
}

function withAbort<T>(operation: Promise<T>, signal: AbortSignal): Promise<T> {
  if (signal.aborted) return Promise.reject(new Error("Operation aborted."));
  return new Promise((resolve, reject) => {
    const abort = () => reject(new Error("Operation aborted."));
    signal.addEventListener("abort", abort, { once: true });
    operation.then(resolve, reject).finally(() => {
      signal.removeEventListener("abort", abort);
    });
  });
}

async function fetchWithValidatedRedirects(url: URL, signal: AbortSignal, redirectsRemaining = 5): Promise<Response> {
  await assertSafeFetchUrl(url);
  const response = await fetch(url, {
    signal,
    redirect: "manual"
  });

  if (isRedirect(response.status)) {
    if (redirectsRemaining <= 0) {
      throw new Error(`Too many redirects while fetching ${url.toString()}.`);
    }
    const location = response.headers.get("location");
    if (!location) {
      throw new Error(`Redirect from ${url.toString()} did not include a Location header.`);
    }
    return fetchWithValidatedRedirects(new URL(location, url), signal, redirectsRemaining - 1);
  }

  return response;
}

function isRedirect(status: number): boolean {
  return status >= 300 && status < 400;
}

function validateCallerProvidedLlmsUrl(value: string): URL {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    throw new Error("Caller-provided llms.txt source must be a valid URL or a known source id.");
  }

  assertLlmsPath(url);
  assertSafeUrlShape(url);

  return url;
}

async function assertSafeFetchUrl(url: URL): Promise<void> {
  assertSafeUrlShape(url);
  assertLlmsPath(url);
  const hostname = normalizeHostname(url.hostname);
  if (isIP(hostname)) {
    if (isUnsafeIpAddress(hostname)) {
      throw new Error("Caller-provided llms.txt source must not target localhost, private, or link-local hosts.");
    }
    return;
  }

  const addresses = await lookup(hostname, {
    all: true,
    verbatim: true
  });
  if (addresses.some((address) => isUnsafeIpAddress(address.address))) {
    throw new Error("Caller-provided llms.txt source resolved to a localhost, private, or link-local address.");
  }
}

function assertLlmsPath(url: URL): void {
  if (!/\/llms(?:-full)?\.txt$/i.test(url.pathname)) {
    throw new Error("Caller-provided llms.txt source path must end with /llms.txt or /llms-full.txt.");
  }
}

function assertSafeUrlShape(url: URL): void {
  if (url.protocol !== "https:") {
    throw new Error("Caller-provided llms.txt source must use https.");
  }
  if (url.username || url.password) {
    throw new Error("Caller-provided llms.txt source must not include credentials.");
  }
  if (url.port) {
    throw new Error("Caller-provided llms.txt source must not include an explicit port.");
  }
  if (isUnsafeHostname(url.hostname)) {
    throw new Error("Caller-provided llms.txt source must not target localhost, private, or link-local hosts.");
  }
}

function normalizeHostname(hostname: string): string {
  return hostname.toLowerCase().replace(/^\[|\]$/g, "").replace(/\.$/, "");
}

function isUnsafeHostname(hostname: string): boolean {
  const normalized = normalizeHostname(hostname);
  if (normalized === "localhost" || normalized.endsWith(".localhost")) return true;
  if (isIP(normalized)) return isUnsafeIpAddress(normalized);
  return false;
}

function isUnsafeIpAddress(address: string): boolean {
  const normalized = normalizeHostname(address);
  const mappedIpv4 = normalized.match(/^::ffff:(\d+\.\d+\.\d+\.\d+)$/i);
  if (mappedIpv4) return isUnsafeIpAddress(mappedIpv4[1]);
  if (/^::ffff:/i.test(normalized)) return true;
  if (normalized === "0.0.0.0" || normalized === "::" || normalized === "::1" || normalized === "0:0:0:0:0:0:0:1") return true;
  if (/^127\./.test(normalized) || /^10\./.test(normalized) || /^169\.254\./.test(normalized)) return true;
  if (/^192\.168\./.test(normalized)) return true;
  const private172 = normalized.match(/^172\.(\d+)\./);
  if (private172 && Number(private172[1]) >= 16 && Number(private172[1]) <= 31) return true;
  if (/^(fc|fd)[0-9a-f]{2}:/i.test(normalized) || /^fe80:/i.test(normalized)) return true;
  return false;
}
