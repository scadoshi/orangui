//! Nested corners: a rounded box that sits near another's corner takes the outer radius minus
//! the border and the inset between them (never under 0.15rem), so the gap between the two
//! curves stays even around the corner. The outer radius and the inset are tokens, so a host
//! that changes one moves the inner corner with it.

const COMPONENTS: &str = include_str!("../assets/components.css");

/// The body of the rule for `selector` in `sheet`.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let Some(at) = sheet.find(&format!("\n{selector} {{")) else {
        return "";
    };
    sheet
        .get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn dock_buttons_follow_the_dock_corner() {
    let dock = rule(COMPONENTS, ":where(.action-dock)");
    assert!(
        dock.contains("--dock-radius: var(--corner-radius);"),
        "the corner floats share one radius: {dock:?}"
    );
    assert!(dock.contains("--dock-inset: var(--space-1);"), "{dock:?}");
    assert!(
        dock.contains("border-radius: var(--dock-radius);"),
        "{dock:?}"
    );
    let button = rule(COMPONENTS, ":where(.action-dock-btn)");
    assert!(
        button.contains("margin: var(--dock-inset) 0;"),
        "{button:?}"
    );
    assert!(
        button.contains(
            "border-radius: max(0.15rem, calc(var(--dock-radius) - 1px - var(--dock-inset)));"
        ),
        "{button:?}"
    );
}
