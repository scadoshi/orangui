//! The folded menu's edges: a full-bleed rule under the header row as it opens, and its footer
//! band padded below as much as above, so the buttons in it never sit on the panel's bottom rule.

const NAV_FOLD: &str = include_str!("../assets/nav_fold.css");

/// The declarations of the first rule whose selector is exactly `selector`, or `""`.
fn rule(selector: &str) -> &'static str {
    let head = format!("\n{selector} {{");
    NAV_FOLD
        .split_once(head.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_open_menu_has_a_rule_under_the_header() {
    let closed = rule(".nav-panel");
    assert!(
        closed.contains("border-top: 1px solid transparent;"),
        "{closed}"
    );
    let open = rule(".nav-panel-open");
    assert!(
        open.contains("border-top-color: var(--border-primary);"),
        "{open}"
    );
}

#[test]
fn the_menu_footer_is_padded_below_as_above() {
    let band = rule(".nav-panel .nav-trailing");
    assert!(
        band.contains("padding: var(--space-3) var(--nav-pad);"),
        "{band}"
    );
    let row = rule(".nav-panel .nav-trailing-row");
    assert!(
        row.contains("padding-bottom: 0;"),
        "the band's padding is the room, not the row's own: {row}"
    );
}
