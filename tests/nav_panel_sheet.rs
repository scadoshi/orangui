//! A theme sheet opened from the folded menu: the sheet is the panel's descendant, and a
//! transform on the panel would make the panel the box the fixed sheet is placed in, so the panel
//! drops its closed slide while a theme sheet in it is up or leaving.

const NAV_FOLD: &str = include_str!("../assets/nav_fold.css");

/// The declarations of the first rule whose selector is exactly `selector`, or `""`.
fn rule(selector: &str) -> &'static str {
    let head = format!("{selector} {{");
    NAV_FOLD
        .split_once(head.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_panel_holding_an_open_theme_sheet_takes_no_transform() {
    let held =
        rule(".nav-panel:has(.theme-picker-sheet .bottom-sheet:is(.show, .bottom-sheet-leaving))");
    assert!(held.contains("visibility: visible;"), "the rule: {held}");
    assert!(
        held.contains("transform: none;"),
        "no containing block for the fixed sheet: {held}"
    );
}
