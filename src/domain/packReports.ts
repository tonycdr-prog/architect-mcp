import { listIngestedLlmsSources } from "./llmsSources.js";
import { listStackPacks } from "./stackPacks.js";

export function scoreStackPacks() {
  const packs = listStackPacks();
  return {
    packs: packs.map((pack) => {
      const sourceScore = pack.sources.some((source) => source.url && /sha256|snapshot/i.test(source.note ?? "")) ? 25 : pack.sources.length ? 15 : 0;
      const detectorScore = pack.fileRules.some((rule) => rule.triggerKind && rule.triggerKind !== "manual-review") ? 25 : 0;
      const exampleScore = pack.fileRules.every((rule) => rule.goodExample && rule.badExample) ? 20 : 0;
      const testScore = pack.testingExpectations.length > 0 ? 15 : 0;
      const boundaryScore = pack.moduleBoundaries.length > 0 && pack.directories.length > 0 ? 15 : 0;
      const score = sourceScore + detectorScore + exampleScore + testScore + boundaryScore;
      return {
        id: pack.id,
        score,
        status: score >= 85 ? "pass" : score >= 65 ? "warn" : "fail",
        dimensions: { sourceScore, detectorScore, exampleScore, testScore, boundaryScore },
        detectorKinds: [...new Set(pack.fileRules.map((rule) => rule.triggerKind ?? "manual-review"))]
      };
    })
  };
}

export function createStackPackCoverageMatrix() {
  const packs = listStackPacks();
  const ingested = listIngestedLlmsSources().sources;
  const ingestedById = new Map(ingested.map((source) => [source.id, source]));

  return {
    rows: packs.map((pack) => {
      const source = ingestedById.get(pack.id);
      return {
        id: pack.id,
        hasPack: true,
        hasIngestedLlms: Boolean(source),
        ingestedStatus: source?.status,
        detectorKinds: [...new Set(pack.fileRules.map((rule) => rule.triggerKind ?? "manual-review"))],
        hasExecutableDetector: pack.fileRules.some((rule) => rule.triggerKind && rule.triggerKind !== "manual-review"),
        hasTests: pack.testingExpectations.length > 0,
        hasDocs: pack.agentInstructions.length > 0 && pack.sources.length > 0
      };
    })
  };
}
