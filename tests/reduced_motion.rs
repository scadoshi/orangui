//! Under `prefers-reduced-motion: reduce`, the kit's moving parts hold still: the action dock
//! rides the keyboard without easing, a banner rises in and leaves without sliding or folding,
//! the toast stack expands and collapses at once, a field's clear button shows without the field
//! giving up its width over time, and a chart tip jumps to its point. A banner's leave still
//! runs, for 1ms, because the banner leaves the page when it ends.

const COMPONENTS: &str = include_str!("../assets/components.css");
const TOAST: &str = include_str!("../assets/toast.css");

/// `sheet` without its comments.
fn uncommented(sheet: &str) -> String {
    let mut out = String::with_capacity(sheet.len());
    let mut rest = sheet;
    while let Some(start) = rest.find("/*") {
        let (before, after) = rest.split_at(start);
        out.push_str(before);
        let end = after
            .find("*/")
            .map_or(after.len(), |e| e.saturating_add(2));
        rest = after.split_at(end).1;
    }
    out.push_str(rest);
    out
}

/// The rules inside `sheet`'s reduced-motion blocks, as (selectors, declarations), whitespace
/// collapsed.
fn reduced_rules(sheet: &str) -> Vec<(String, String)> {
    let sheet = uncommented(sheet);
    let mut found = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut text = String::new();
    for ch in sheet.chars() {
        match ch {
            '{' => {
                open.push(text.split_whitespace().collect::<Vec<_>>().join(" "));
                text.clear();
            }
            '}' => {
                let prelude = open.pop().unwrap_or_default();
                if open
                    .iter()
                    .any(|p| p.contains("prefers-reduced-motion: reduce"))
                {
                    found.push((
                        prelude,
                        text.split_whitespace().collect::<Vec<_>>().join(" "),
                    ));
                }
                text.clear();
            }
            ';' if open.is_empty() => text.clear(),
            _ => text.push(ch),
        }
    }
    found
}

/// Whether a reduced-motion rule in `sheet` names `selector` and declares `declaration`.
fn stills(sheet: &str, selector: &str, declaration: &str) -> bool {
    reduced_rules(sheet).iter().any(|(selectors, body)| {
        selectors.split(',').any(|s| s.trim() == selector) && body.contains(declaration)
    })
}

#[test]
fn the_dock_rides_the_keyboard_without_easing() {
    assert!(stills(COMPONENTS, ".action-dock", "transition: none;"));
}

#[test]
fn a_banner_rises_in_and_leaves_without_sliding_or_folding() {
    assert!(stills(COMPONENTS, ".banner", "animation: none;"));
    assert!(
        stills(
            COMPONENTS,
            ".banner.banner-leaving",
            "animation-duration: 1ms;"
        ),
        "the leave still ends, so the banner leaves the page"
    );
}

#[test]
fn the_toast_stack_expands_and_collapses_at_once() {
    assert!(stills(
        TOAST,
        ".toast-container.animating li",
        "transition: none;"
    ));
    assert!(stills(
        TOAST,
        ".toast-container.animating ol",
        "transition: none;"
    ));
}

#[test]
fn a_clear_button_shows_without_the_field_giving_up_its_width_over_time() {
    assert!(stills(
        COMPONENTS,
        ".input-clear-btn",
        "transition: opacity 0.2s ease;"
    ));
}

#[test]
fn a_chart_tip_jumps_to_its_point() {
    assert!(stills(COMPONENTS, ".chart-tip", "transition: none;"));
}
