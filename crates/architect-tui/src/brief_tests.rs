use serde_json::json;

use crate::brief::{apply_brief_answer, brief_from_prompt};

#[test]
fn answer_parser_shapes_grill_me_fields() {
    let mut brief = json!({ "idea": "build controlled TUI" });
    apply_brief_answer(&mut brief, "coreFlows", "intake; adapter review; promote");
    apply_brief_answer(
        &mut brief,
        "verification",
        "cargo test --workspace; npm run release:check",
    );
    apply_brief_answer(
        &mut brief,
        "stack",
        "frontend=Rust Ratatui; backend=TypeScript MCP",
    );
    apply_brief_answer(
        &mut brief,
        "repoLayout",
        "tui=crates/architect-tui/src; docs=docs",
    );

    assert_eq!(brief["coreFlows"].as_array().expect("flows").len(), 3);
    assert_eq!(brief["verification"].as_array().expect("checks").len(), 2);
    assert_eq!(brief["stack"]["frontend"], "Rust Ratatui");
    assert_eq!(
        brief["repoLayout"]["pathMap"]["tui"][0],
        "crates/architect-tui/src"
    );
}

#[test]
fn prompt_parser_extracts_labeled_brief_segments() {
    let brief = brief_from_prompt(
        "Users: maintainers. Job: control agents. Core flows: grill; review; promote. Stack: frontend=Rust; backend=MCP. Risk: auto mutation. Verification: cargo test",
    );

    assert!(
        brief["users"]
            .as_str()
            .expect("users")
            .contains("control agents")
    );
    assert_eq!(brief["coreFlows"].as_array().expect("flows").len(), 3);
    assert_eq!(brief["stack"]["backend"], "MCP");
    assert_eq!(brief["risk"], "auto mutation.");
    assert_eq!(brief["verification"][0], "cargo test");
}

#[test]
fn prompt_parser_trims_sentence_periods_from_list_items_but_not_text() {
    let brief = brief_from_prompt(
        "Risks: missing proof. Verification: cargo test --workspace; npm run release:check.",
    );

    assert_eq!(brief["risk"], "missing proof.");
    assert_eq!(brief["verification"][0], "cargo test --workspace");
    assert_eq!(brief["verification"][1], "npm run release:check");
}

#[test]
fn prompt_parser_uses_original_unicode_offsets() {
    let brief = brief_from_prompt(
        "İstanbul maintainers need safety. Users: terminal operators. Core flows: grill; approve.",
    );

    assert_eq!(brief["users"], "terminal operators.");
    assert_eq!(brief["coreFlows"][0], "grill");
    assert_eq!(brief["coreFlows"][1], "approve");
}

#[test]
fn answer_parser_ignores_unknown_fields_that_would_break_strict_schema() {
    let mut brief = json!({ "idea": "build controlled TUI" });

    apply_brief_answer(&mut brief, "stack.mobile", "Flutter");
    apply_brief_answer(&mut brief, "unknown", "poison");
    apply_brief_answer(&mut brief, "stack.backend", "TypeScript MCP.");

    assert!(brief.get("stack.mobile").is_none());
    assert!(brief.get("unknown").is_none());
    assert_eq!(brief["stack"]["backend"], "TypeScript MCP");
    assert!(brief["stack"].get("mobile").is_none());
}
