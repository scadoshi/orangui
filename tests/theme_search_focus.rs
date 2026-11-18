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

#[test]
fn the_search_stays_in_view_as_the_themes_scroll() {
    let bar = rule(".theme-search-bar");
    assert!(bar.contains("position: sticky;"), "{bar}");
    assert!(
        bar.contains("top: calc(-1 * var(--space-4));"),
        "pinned at the sheet body's padding edge, past the content edge sticky uses: {bar}"
    );
    assert!(
        bar.contains("background-color: var(--bg-sink);"),
        "opaque in the sheet's own fill, so rows pass under it: {bar}"
    );
}

#[test]
fn the_search_is_a_theme_rows_size() {
    let field = rule(".theme-search");
    for want in [
        "padding: var(--space-2) var(--space-4);",
        "font-size: 0.85rem;",
        "border-radius: 0.4rem;",
    ] {
        assert!(field.contains(want), "{want} as a row has it: {field}");
    }
    let wrap = rule(".theme-search-bar .input-clear-wrap");
    assert!(
        wrap.contains("margin-bottom: 0;"),
        "the bar's padding is the gap under it: {wrap}"
    );
}

#[test]
fn a_phone_s_theme_sheet_holds_its_height_while_searching() {
    let held = rule(
        "    .bottom-sheet:has(.theme-search:focus, .input-clear-wrap.has-text > .theme-search)",
    );
    assert!(held.contains("height: 85vh;"), "full height: {held}");
    let at = COMPONENTS
        .find(".bottom-sheet:has(.theme-search:focus")
        .unwrap_or(0);
    let query = COMPONENTS
        .get(..at)
        .and_then(|head| head.rfind("@media"))
        .and_then(|q| COMPONENTS.get(q..at))
        .and_then(|line| line.split('{').next())
        .unwrap_or("");
    assert_eq!(
        query.trim(),
        "@media (max-width: 59.99rem)",
        "below the drawer"
    );
}
