export function listClientIntegrationRecipes(input: { recipe?: string } = {}) {
  const recipes = [
    {
      id: "guided-yolo-pre-edit",
      title: "Guided YOLO pre-edit gate",
      purpose: "Interpret vague implementation requests before an agent edits code.",
      calls: [
        { tool: "interpret_implementation_intent", arguments: { input: { request: "fix this with best practices", mode: "guided-yolo" } } },
        { tool: "create_pre_edit_contract", arguments: { input: { intent: "interpret_implementation_intent result" } }, when: "decision is confirm_before_edit or blastRadius is high" },
        { tool: "review_implementation_against_contract", arguments: { input: { contract: "pre-edit contract", changedFiles: "changed file summaries", verification: "checks run" } }, when: "after files change" }
      ],
      clientPolicy: "Proceed on green, log assumptions on yellow, ask one plain-English question on red or risky yellow."
    },
    {
      id: "repo-review-ci",
      title: "Repo review CI gate",
      purpose: "Fail only on new architecture findings after a baseline is established.",
      calls: [
        { tool: "scan_mcp_config_files", when: "local client wants MCP config security included" },
        { tool: "review_repo_structure", arguments: { mode: "ci", files: "workspace file summaries", contract: "architecture contract" } },
        { tool: "create_review_baseline", when: "maintainer accepts mature-repo findings" },
        { tool: "review_agent_final_response", when: "before returning final agent output" }
      ],
      clientPolicy: "Persist baselines in the repo and require reasons for accepted findings."
    },
    {
      id: "stack-pack-promotion",
      title: "Source-backed stack-pack promotion",
      purpose: "Promote reviewed llms.txt guidance into versioned pack JSON files.",
      calls: [
        { tool: "derive_stack_pack_from_llms_source", arguments: { request: { sourceId: "react" } } },
        { tool: "review_stack_pack_candidate" },
        { tool: "analyze_stack_pack_conflicts" },
        { tool: "promote_stack_pack_to_files", arguments: { writeFiles: false } },
        { tool: "validate_stack_packs", when: "after writing files" }
      ],
      clientPolicy: "Use dry-run output for review, then write only under packs/ and update packs/manifest.json."
    },
    {
      id: "memory-pr-review",
      title: "GitHub-style memory proposal review",
      purpose: "Batch useful memory without silently writing durable state.",
      calls: [
        { tool: "extract_harness_memory" },
        { tool: "review_memory_relevance" },
        { tool: "apply_harness_memory", when: "memory is relevant and within token budget" }
      ],
      clientPolicy: "Treat red memory as confirm/discard, batch yellow memory for review, and disclose every applied memory."
    }
  ];

  return {
    recipes: input.recipe ? recipes.filter((recipe) => recipe.id === input.recipe) : recipes
  };
}
