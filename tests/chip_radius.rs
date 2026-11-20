//! Chip corners: a chip that only shows something (a tag, a stat chip, a status pill, the
//! no-results chip) takes the read-only radius, and a chip that does something (the toggle chip,
//! a tag or stat chip that is a link or a button) takes the control radius, so the corner says
//! which is which.

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
fn the_two_radii_are_tokens() {
    let root = rule(COMPONENTS, ":root");
    assert!(root.contains("--radius-readonly: 0.4rem;"), "{root:?}");
    assert!(root.contains("--radius-control: 0.6rem;"), "{root:?}");
}

#[test]
fn read_only_chips_take_the_read_only_radius() {
    for selector in [
        ":where(.tag)",
        ":where(.stat-chip)",
        ".status-tag",
        ".chip-unselected",
    ] {
        let body = rule(COMPONENTS, selector);
        assert!(
            body.contains("border-radius: var(--radius-readonly);"),
            "{selector}: {body:?}"
        );
    }
}

#[test]
fn interactive_chips_take_the_control_radius() {
    for selector in [
        ".chip",
        ":where(a.tag, button.tag, a.stat-chip, button.stat-chip)",
    ] {
        let body = rule(COMPONENTS, selector);
        assert!(
            body.contains("border-radius: var(--radius-control);"),
            "{selector}: {body:?}"
        );
    }
}

#[test]
fn a_linked_chip_outranks_the_read_only_default() {
    let read_only = COMPONENTS.find("\n:where(.stat-chip) {");
    let control = COMPONENTS.find("\n:where(a.tag, button.tag, a.stat-chip, button.stat-chip) {");
    assert!(
        matches!((read_only, control), (Some(r), Some(c)) if c > r),
        "both rules have zero specificity, so the control rule comes later"
    );
}
