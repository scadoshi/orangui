//! The theme search field's states: a keyboard user sees where focus is, from a ring at least
//! 2px thick drawn in place of the browser's, and a mouse sees the field answer its hover.

const COMPONENTS: &str = include_str!("../assets/components.css");

/// The declarations of the first rule whose selector is exactly `selector`, or `""`.
fn rule(selector: &str) -> &'static str {
    let head = format!("{selector} {{");
    COMPONENTS
        .split_once(head.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_focused_search_draws_a_ring_of_its_own() {
    let focus = rule(".theme-search:focus");
    assert!(!focus.is_empty(), "a focus rule");
    assert!(
        !focus.contains("outline: none"),
        "keeps a ring, not none: {focus}"
    );
    assert!(
        focus.contains("outline: 2px solid var(--accent-primary)"),
        "a 2px ring in the accent: {focus}"
    );
}

#[test]
fn the_search_answers_a_hover() {
    let hover = rule("    .theme-search:not(:focus):hover");
    assert!(
        hover.contains("border-color"),
        "the edge answers a hover: {hover}"
    );
}
