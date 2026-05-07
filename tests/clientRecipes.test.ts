import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { listClientIntegrationRecipes } from "../src/domain/clientRecipes.js";

describe("client integration recipes", () => {
  it("lists executable recipes for the V3 client workflows", () => {
    const result = listClientIntegrationRecipes();
    const ids = result.recipes.map((recipe) => recipe.id);

    assert.equal(ids.includes("guided-yolo-pre-edit"), true);
    assert.equal(ids.includes("repo-review-ci"), true);
    assert.equal(ids.includes("stack-pack-promotion"), true);
    assert.equal(ids.includes("memory-pr-review"), true);
    assert.equal(result.recipes.every((recipe) => recipe.calls.length > 0), true);
    assert.deepEqual(
      result.recipes.find((recipe) => recipe.id === "guided-yolo-pre-edit")?.calls[0]?.arguments,
      { input: { request: "fix this with best practices", mode: "guided-yolo" } }
    );
    assert.deepEqual(
      result.recipes.find((recipe) => recipe.id === "guided-yolo-pre-edit")?.calls[1]?.arguments,
      { intent: "interpret_implementation_intent result" }
    );
  });

  it("filters to a requested recipe", () => {
    const result = listClientIntegrationRecipes({ recipe: "stack-pack-promotion" });

    assert.deepEqual(result.recipes.map((recipe) => recipe.id), ["stack-pack-promotion"]);
    assert.equal(result.recipes[0]?.calls.some((call) => call.tool === "promote_stack_pack_to_files"), true);
  });
});
