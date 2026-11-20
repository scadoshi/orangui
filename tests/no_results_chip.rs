//! A no-results state reads as one muted chip, the way zwipe's Quick add says "No results" and
//! Send to says "No deck matches": the kit owns that chip, `.chip-unselected`, and the theme
//! search's "No themes match" wears it.

const COMPONENTS: &str = include_str!("../assets/components.css");
const THEME_SEARCH: &str = include_str!("../src/theme_search.rs");

#[test]
fn the_kit_owns_the_no_results_chip() {
    let rule = COMPONENTS
        .split_once("\n.chip-unselected {")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body);
    for declaration in [
        "display: inline-block;",
        "padding: var(--space-1) var(--space-2);",
        "background-color: var(--bg-primary);",
        "border: 1px solid var(--border-muted);",
        "border-radius: var(--radius-readonly);",
        "font-size: 0.75rem;",
        "color: var(--ink-text-muted, var(--text-muted));",
        "box-shadow: var(--shadow-sm);",
    ] {
        assert!(rule.contains(declaration), "{declaration} in {rule:?}");
    }
}

#[test]
fn no_themes_match_reads_as_the_no_results_chip() {
    assert!(
        THEME_SEARCH.contains("div { class: \"chip-unselected\", \"No themes match\" }"),
        "the theme search's empty line is the chip"
    );
    assert!(
        !THEME_SEARCH.contains("p { class: \"theme-search-empty\", \"No themes match\" }"),
        "not plain text"
    );
}
