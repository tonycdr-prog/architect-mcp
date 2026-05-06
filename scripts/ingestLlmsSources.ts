import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { KNOWN_LLMS_SOURCES, fetchLlmsSource } from "../src/domain/llmsSources.js";
import type { IngestedLlmsSource } from "../src/domain/types.js";

const outputDirectory = "stack-sources/ingested";

await mkdir(outputDirectory, { recursive: true });

const sources: IngestedLlmsSource[] = [];

for (const source of KNOWN_LLMS_SOURCES) {
  try {
    const snapshot = await fetchLlmsSource(source.id, { maxBytes: 500_000 });
    const path = join(outputDirectory, `${source.id}.txt`);
    await writeFile(path, snapshot.content, "utf8");
    sources.push({
      id: source.id,
      stack: source.stack,
      category: source.category,
      url: snapshot.source.url,
      fetchedAt: snapshot.fetchedAt,
      sha256: snapshot.sha256,
      bytes: snapshot.bytes,
      contentType: snapshot.contentType,
      path,
      status: "ok"
    });
  } catch (error) {
    sources.push({
      id: source.id,
      stack: source.stack,
      category: source.category,
      url: source.url,
      fetchedAt: new Date().toISOString(),
      status: "error",
      error: error instanceof Error ? error.message : String(error)
    });
  }
}

const index = {
  generatedAt: new Date().toISOString(),
  sources
};

await writeFile(join(outputDirectory, "index.json"), `${JSON.stringify(index, null, 2)}\n`, "utf8");

const successes = sources.filter((source) => source.status === "ok").length;
const failures = sources.length - successes;
console.log(JSON.stringify({ total: sources.length, successes, failures }, null, 2));
