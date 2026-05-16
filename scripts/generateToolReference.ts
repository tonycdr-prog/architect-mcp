import { readFileSync, writeFileSync } from "node:fs";
import { generateToolReferenceMarkdown } from "../src/domain/toolReferenceDocs.js";

const outputPath = "docs/tool-reference.md";
const generated = generateToolReferenceMarkdown();
const check = process.argv.includes("--check");

if (check) {
  const current = readFileSync(outputPath, "utf8");
  if (current !== generated) {
    throw new Error(`${outputPath} is stale. Run npm run docs:tool-reference.`);
  }
} else {
  writeFileSync(outputPath, generated, "utf8");
  console.log(`Wrote ${outputPath}`);
}
