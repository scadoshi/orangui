//! A text field's edge: the only thing that shows a field on a panel of the same fill, so it
//! needs 3:1 against what it sits on (WCAG 2.2 SC 1.4.11). The field borders in
//! `--ink-text-muted`, which the ink gate (`themes_ink.rs`) holds at 4.5:1 or better on
//! `--bg-primary` and `--bg-sink` in every theme; focus keeps its accent border.

const APP: &str = include_str!("../assets/app.css");

/// The body of the rule for `selector`.
fn rule(selector: &str) -> &'static str {
    let Some(at) = APP.find(&format!("\n{selector} {{")) else {
        return "";
    };
    APP.get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_field_borders_in_the_muted_ink() {
    let input = rule(".input");
    assert!(
        input.contains("border: 1px solid var(--ink-text-muted, var(--text-muted));"),
        "{input:?}"
    );
}

#[test]
fn focus_keeps_its_accent_border() {
    let focus = rule(".input:focus");
    assert!(
        focus.contains("border-color: var(--accent-primary);"),
        "{focus:?}"
    );
}
