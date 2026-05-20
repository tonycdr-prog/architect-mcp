export type ToolSurface = "core" | "advanced";
export type ToolPolicy = "hosted-safe" | "local-only" | "future-adapter" | "unknown";

export const CORE_ARCHITECTURE_TOOL_NAMES = [
  "grill_me",
  "create_pre_edit_contract",
  "review_build_plan",
  "review_proposed_file_plan",
  "review_repo_structure",
  "review_implementation_against_contract",
  "review_agent_final_response",
  "review_agent_session"
] as const;

export const LOCAL_ONLY_ARCHITECTURE_TOOL_NAMES = [
  "promote_stack_pack_to_files",
  "review_local_workspace",
  "derive_local_repo_constitution",
  "scan_mcp_config_files",
  "apply_mcp_install_plan"
] as const;

export const FUTURE_ADAPTER_ARCHITECTURE_TOOL_NAMES = [
  "extract_harness_memory",
  "apply_harness_memory",
  "review_memory_relevance"
] as const;

export const ADVANCED_ARCHITECTURE_TOOL_NAMES = [
  "list_stack_packs", "validate_stack_packs", "list_foundation_packs", "validate_foundation_packs",
  "stack_pack_expansion_strategy", "discover_llms_sources", "fetch_llms_source", "ingest_llms_txt",
  "list_ingested_llms_sources", "derive_stack_pack_from_llms_source", "propose_stack_pack_rules",
  "review_stack_pack_candidate", "promote_stack_pack_candidate", "promote_stack_pack_to_files",
  "analyze_stack_pack_conflicts", "diff_stack_pack_versions", "list_repo_profiles", "get_repo_profile",
  "infer_repo_layout", "start_project_intake", "grill_project_brief", "grill_me", "continue_grill_me",
  "generate_architecture_contract", "generate_agent_instructions", "generate_agents_md", "generate_cursor_rules",
  "generate_repo_artifacts", "generate_repo_scaffold_plan", "review_build_plan", "review_proposed_file_plan",
  "review_repo_structure", "create_review_baseline", "validate_architecture_contract", "validate_repo_artifacts",
  "self_review_architect_mcp", "diff_architecture_contracts", "mcp_readiness_report",
  "interpret_implementation_intent", "classify_ambiguity_risk", "create_pre_edit_contract",
  "review_implementation_against_contract", "record_assumption", "load_triggered_stack_guidance",
  "extract_harness_memory", "apply_harness_memory", "review_memory_relevance",
  "list_mcp_server_catalog", "recommend_mcp_servers", "create_mcp_install_plan",
  "review_mcp_install_plan", "apply_mcp_install_plan", "list_skill_catalog",
  "recommend_skills_for_project", "review_supplied_skills", "review_mcp_config_security",
  "run_v3_eval_harness", "score_agent_artifacts", "list_client_integration_recipes",
  "review_agent_final_response", "review_agent_session", "audit_work_gate_completeness", "create_work_gate_sequence_receipt", "audit_hosted_tool_policy", "score_stack_packs",
  "stack_pack_coverage_matrix",
  "resolve_standards_profile", "explain_review_findings", "simulate_policy_gate",
  "analyze_standards_conflicts", "score_repo_profile_fit", "review_contract_lifecycle",
  "run_v5_eval_harness", "list_policy_bundles", "validate_policy_bundles", "preview_policy_bundle",
  "summarize_session_continuity", "cluster_review_findings", "generate_local_report_artifact",
  "analyze_regression_coverage", "review_pattern_card", "preview_pattern_card", "run_v6_eval_harness",
  "create_architecture_strategy_map", "compare_standards_profiles", "preview_change_what_if",
  "diagnose_agent_behavior", "draft_rule_candidate", "compare_review_trends", "render_governance_pack",
  "run_v7_eval_harness", "review_standards_refactor", "minimize_policy_set", "select_review_playbook",
  "review_playbook_conformance", "check_agent_collaboration_plan", "run_failure_mode_drills",
  "calibrate_rule_impact", "review_documentation_intelligence", "run_v8_eval_harness",
  "select_local_orchestration_recipe", "evaluate_scenario_acceptance", "normalize_mcp_result",
  "plan_context_budget", "route_evidence", "create_local_dry_run_plan", "review_tool_loop_quality",
  "run_v9_eval_harness",
  "get_v10_productization_blueprint", "create_v10_implementation_slice_plan", "plan_primer_dashboard",
  "validate_v10_productization_boundary", "run_v10_eval_harness",
  "build_quality_requirements_profile", "evaluate_repo_plan_quality", "audit_generated_repo_quality",
  "suggest_quality_followup_questions", "run_repo_quality_eval_scenarios",
  "forge_foundry_previews", "route_foundry_decisions", "score_foundry_actionability", "normalize_foundry_evidence", "derive_repo_constitution", "derive_local_repo_constitution",
  "review_local_workspace", "scan_mcp_config_files"
] as const;

const LOCAL_ONLY_TOOLS = new Set<string>(LOCAL_ONLY_ARCHITECTURE_TOOL_NAMES);
const ADVANCED_TOOL_NAMES = new Set<string>(ADVANCED_ARCHITECTURE_TOOL_NAMES);
const CORE_TOOL_NAMES = new Set<string>(CORE_ARCHITECTURE_TOOL_NAMES);

export function registeredArchitectureToolNames(includeLocal: boolean, surface: ToolSurface = "advanced"): string[] {
  const names = surface === "core" ? [...CORE_ARCHITECTURE_TOOL_NAMES] : [...ADVANCED_ARCHITECTURE_TOOL_NAMES];
  return names.filter((name) => includeLocal || !LOCAL_ONLY_TOOLS.has(name));
}

export function isArchitectureToolRegistered(name: string): boolean {
  return ADVANCED_TOOL_NAMES.has(name);
}

export function isArchitectureToolLocalOnly(name: string): boolean {
  return LOCAL_ONLY_TOOLS.has(name);
}

export function isArchitectureToolEnabled(name: string, options: { includeLocal: boolean; surface: ToolSurface }): boolean {
  if (isArchitectureToolLocalOnly(name) && !options.includeLocal) return false;
  return options.surface === "core" ? CORE_TOOL_NAMES.has(name) : ADVANCED_TOOL_NAMES.has(name);
}
