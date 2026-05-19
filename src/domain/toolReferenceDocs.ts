import {
  ADVANCED_ARCHITECTURE_TOOL_NAMES,
  CORE_ARCHITECTURE_TOOL_NAMES,
  FUTURE_ADAPTER_ARCHITECTURE_TOOL_NAMES,
  LOCAL_ONLY_ARCHITECTURE_TOOL_NAMES
} from "../tools/toolRegistry.js";

type ToolReferenceGroup = {
  title: string;
  description: string;
  tools: readonly string[];
};

const TOOL_DESCRIPTIONS: Record<string, string> = {
  grill_me: "Pressure-test the request before planning or editing.",
  create_pre_edit_contract: "Turn clarified intent into an explicit edit contract.",
  review_build_plan: "Review the implementation plan against the agreed contract.",
  review_proposed_file_plan: "Check file paths and ownership before edits start.",
  review_repo_structure: "Review repository structure from supplied file summaries.",
  review_implementation_against_contract: "Compare changed files and verification against the pre-edit contract.",
  review_agent_final_response: "Check the final response for evidence, honesty, and not-done disclosure.",
  review_agent_session: "Combine intent, contract, drift, memory, and final-response checks into one session review.",
  audit_work_gate_completeness: "Report whether direct MCP workflows supplied complete, fresh, ordered work-gate evidence.",
  list_mcp_server_catalog: "List the curated MCP server catalog.",
  recommend_mcp_servers: "Recommend MCP servers only after provider and boundary checks.",
  create_mcp_install_plan: "Create a dry-run MCP client configuration plan.",
  review_mcp_install_plan: "Review an MCP install plan for unsafe config, secrets, and unpinned packages.",
  apply_mcp_install_plan: "Local-only dry-run/apply tool for project-local MCP client config."
};

const TOOL_REFERENCE_GROUPS: ToolReferenceGroup[] = [
  {
    title: "Core Work Gate",
    description: "The default public surface for clarifying work, constraining edits, reviewing drift, and checking final evidence.",
    tools: CORE_ARCHITECTURE_TOOL_NAMES
  },
  {
    title: "Stack Packs And Standards",
    description: "Stack-pack loading, source ingestion, promotion, standards comparison, and policy set tuning.",
    tools: [
      "list_stack_packs",
      "validate_stack_packs",
      "list_foundation_packs",
      "validate_foundation_packs",
      "stack_pack_expansion_strategy",
      "discover_llms_sources",
      "fetch_llms_source",
      "ingest_llms_txt",
      "list_ingested_llms_sources",
      "derive_stack_pack_from_llms_source",
      "propose_stack_pack_rules",
      "review_stack_pack_candidate",
      "promote_stack_pack_candidate",
      "promote_stack_pack_to_files",
      "analyze_stack_pack_conflicts",
      "diff_stack_pack_versions",
      "score_stack_packs",
      "stack_pack_coverage_matrix",
      "resolve_standards_profile",
      "analyze_standards_conflicts",
      "compare_standards_profiles",
      "review_standards_refactor",
      "minimize_policy_set",
      "calibrate_rule_impact"
    ]
  },
  {
    title: "Repo Intake And Scaffolding",
    description: "Repo profile selection, intake, artifact generation, scaffold planning, and repo-readiness checks.",
    tools: [
      "list_repo_profiles",
      "get_repo_profile",
      "infer_repo_layout",
      "start_project_intake",
      "grill_project_brief",
      "continue_grill_me",
      "generate_architecture_contract",
      "generate_agent_instructions",
      "generate_agents_md",
      "generate_cursor_rules",
      "generate_repo_artifacts",
      "generate_repo_scaffold_plan",
      "review_local_workspace",
      "create_review_baseline",
      "validate_architecture_contract",
      "validate_repo_artifacts",
      "self_review_architect_mcp",
      "diff_architecture_contracts",
      "mcp_readiness_report",
      "score_repo_profile_fit"
    ]
  },
  {
    title: "Contract And Session Review",
    description: "Ambiguity classification, implementation intent, contract lifecycle review, and session continuity checks.",
    tools: [
      "interpret_implementation_intent",
      "classify_ambiguity_risk",
      "record_assumption",
      "load_triggered_stack_guidance",
      "audit_work_gate_completeness",
      "explain_review_findings",
      "simulate_policy_gate",
      "review_contract_lifecycle",
      "summarize_session_continuity",
      "cluster_review_findings",
      "generate_local_report_artifact"
    ]
  },
  {
    title: "Memory, Skills, And MCP Integrations",
    description: "Advisory memory, skill catalog review, MCP security review, MCP catalog recommendations, and guarded install plans.",
    tools: [
      "extract_harness_memory",
      "apply_harness_memory",
      "review_memory_relevance",
      "list_mcp_server_catalog",
      "recommend_mcp_servers",
      "create_mcp_install_plan",
      "review_mcp_install_plan",
      "apply_mcp_install_plan",
      "list_skill_catalog",
      "recommend_skills_for_project",
      "review_supplied_skills",
      "review_mcp_config_security",
      "scan_mcp_config_files",
      "list_client_integration_recipes",
      "audit_hosted_tool_policy"
    ]
  },
  {
    title: "Eval Harnesses And Repo Quality",
    description: "Deterministic evals, generated repo quality checks, and regression coverage analysis.",
    tools: [
      "run_v3_eval_harness",
      "score_agent_artifacts",
      "run_v5_eval_harness",
      "run_v6_eval_harness",
      "run_v7_eval_harness",
      "run_v8_eval_harness",
      "run_v9_eval_harness",
      "run_v10_eval_harness",
      "analyze_regression_coverage",
      "evaluate_scenario_acceptance",
      "build_quality_requirements_profile",
      "evaluate_repo_plan_quality",
      "audit_generated_repo_quality",
      "suggest_quality_followup_questions",
      "run_repo_quality_eval_scenarios"
    ]
  },
  {
    title: "Governance And Operating Model",
    description: "Policy bundles, governance artifacts, orchestration planning, scenario routing, and productization boundaries.",
    tools: [
      "list_policy_bundles",
      "validate_policy_bundles",
      "preview_policy_bundle",
      "review_pattern_card",
      "preview_pattern_card",
      "create_architecture_strategy_map",
      "preview_change_what_if",
      "diagnose_agent_behavior",
      "draft_rule_candidate",
      "compare_review_trends",
      "render_governance_pack",
      "select_review_playbook",
      "review_playbook_conformance",
      "check_agent_collaboration_plan",
      "run_failure_mode_drills",
      "review_documentation_intelligence",
      "select_local_orchestration_recipe",
      "normalize_mcp_result",
      "plan_context_budget",
      "route_evidence",
      "create_local_dry_run_plan",
      "review_tool_loop_quality",
      "get_v10_productization_blueprint",
      "create_v10_implementation_slice_plan",
      "plan_primer_dashboard",
      "validate_v10_productization_boundary"
    ]
  }
];

export function generateToolReferenceMarkdown(): string {
  validateToolReferenceGroups();
  const lines: string[] = [
    "# Tool Reference",
    "",
    "<!-- This file is generated by `npm run docs:tool-reference`. Do not edit by hand. -->",
    "",
    "architect-mcp exposes a small default core surface and an explicit advanced surface. The default public story is the agent work gate: clarify before edits, constrain the plan, review drift, and require verification evidence.",
    "",
    "Historical V-labels remain in tool, script, test, and compatibility file identifiers. They are internal compatibility references, not public product framing.",
    "",
    "## Surfaces",
    "",
    "- Core: the eight work-gate tools listed in the Core Work Gate section.",
    "- Advanced: the core tools plus the compatibility, maturity, governance, repo-quality, stack-pack, MCP, and eval tools listed below.",
    ""
  ];

  for (const group of TOOL_REFERENCE_GROUPS) {
    lines.push(`## ${group.title}`, "", group.description, "", "| Tool | Purpose |", "| --- | --- |");
    for (const tool of group.tools) {
      lines.push(`| \`${tool}\` | ${descriptionFor(tool)} |`);
    }
    lines.push("");
  }

  lines.push(
    "## Hosted Policy",
    "",
    "Hosted mode exposes only hosted-safe tools. Local-only tools are excluded from hosted HTTP registration even when the advanced surface is requested.",
    "",
    "Local-only tools:",
    "",
    ...LOCAL_ONLY_ARCHITECTURE_TOOL_NAMES.map((tool) => `- \`${tool}\``),
    "",
    "Future-adapter tools currently return stateless proposals. Durable storage waits for an explicit adapter:",
    "",
    ...FUTURE_ADAPTER_ARCHITECTURE_TOOL_NAMES.map((tool) => `- \`${tool}\``),
    "",
    "Unknown tools classify as `unknown` and must never be treated as hosted-safe.",
    ""
  );

  return `${lines.join("\n").trimEnd()}\n`;
}

export function validateToolReferenceGroups(): void {
  const grouped = new Map<string, string>();
  for (const group of TOOL_REFERENCE_GROUPS) {
    for (const tool of group.tools) {
      const existingGroup = grouped.get(tool);
      if (existingGroup) throw new Error(`Tool ${tool} appears in both ${existingGroup} and ${group.title}.`);
      grouped.set(tool, group.title);
    }
  }

  const missing = ADVANCED_ARCHITECTURE_TOOL_NAMES.filter((tool) => !grouped.has(tool));
  if (missing.length > 0) throw new Error(`Tool reference groups missing registered tools: ${missing.join(", ")}`);

  const unknown = [...grouped.keys()].filter((tool) => !ADVANCED_ARCHITECTURE_TOOL_NAMES.includes(tool as typeof ADVANCED_ARCHITECTURE_TOOL_NAMES[number]));
  if (unknown.length > 0) throw new Error(`Tool reference groups include unknown tools: ${unknown.join(", ")}`);
}

function descriptionFor(tool: string): string {
  return TOOL_DESCRIPTIONS[tool] ?? `${titleCase(tool)}.`;
}

function titleCase(tool: string): string {
  return tool
    .split("_")
    .map((part) => part.toUpperCase() === part ? part : `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}
