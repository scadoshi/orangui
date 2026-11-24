//! Toasts and banners sit in the bottom-left corner as the action dock's [+] sits in the
//! bottom-right one: as far from the screen's side edge as from its bottom edge, measured from
//! the physical edges, on a phone with a home indicator and on the web alike. One gap,
//! `--corner-gap`, places all three, so they sit on the same 45 degree line from their corners.
//! A toast is a container, so it rounds at the panel radius with the panel padding inside.
//!
//! A toast rises in from below the bottom edge and drops back below it as it leaves, moving by
//! translate only. Stacked, the older toasts line up behind the newest, which stays in front.

#![expect(
    clippy::float_arithmetic,
    reason = "a test: works out CSS lengths in px"
)]

const TOAST: &str = include_str!("../assets/toast.css");
const COMPONENTS: &str = include_str!("../assets/components.css");
const TOAST_STACK: &str = include_str!("../src/toast_stack.rs");

/// What a toast's entrance starts from and its timed exit ends on: its own height, the corner
/// gap and the keyboard below its spot, and its shadow's reach, so it is wholly below the edge.
const BELOW_THE_EDGE: &str =
    "0 calc(100% + var(--corner-gap) + var(--keyboard-inset, 0px) + var(--space-4))";

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    let (_, rest) = css
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selector}"));
    body
}

/// The value `property` takes in `declarations`, if it is declared there.
fn value<'css>(declarations: &'css str, property: &str) -> Option<&'css str> {
    let prefix = format!("{property}:");
    declarations
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix.as_str()))
        .map(|v| v.trim().trim_end_matches(';'))
}

/// The body of the keyframe stop `stop` (`from`, `to`, `0%`, ...) in `@keyframes name`.
fn keyframe<'css>(css: &'css str, name: &str, stop: &str) -> &'css str {
    let (_, rest) = css
        .split_once(&format!("@keyframes {name} {{"))
        .unwrap_or_else(|| panic!("keyframes {name}"));
    let (_, rest) = rest
        .split_once(&format!("{stop} {{"))
        .unwrap_or_else(|| panic!("{name}'s {stop}"));
    rest.split_once('}').map_or("", |(body, _)| body)
}

/// A length in px: `expr` written with `var(--corner-gap)`, `var(--space-2)`,
/// `var(--keyboard-inset, 0px)`, `env(safe-area-inset-*, 0px)`, px, rem, `+`, `/` and `calc()`,
/// on a device whose home indicator's safe area is `bottom_inset` px and whose side safe area is
/// none.
fn px(expr: &str, bottom_inset: f64) -> f64 {
    let gap = value(rule(COMPONENTS, ":root"), "--corner-gap").unwrap_or("0px");
    let resolved = expr
        .replace("var(--corner-gap)", gap)
        .replace("var(--keyboard-inset, 0px)", "0px")
        .replace("var(--space-2)", "0.5rem")
        .replace(
            "env(safe-area-inset-bottom, 0px)",
            &format!("{bottom_inset}px"),
        )
        .replace("env(safe-area-inset-left, 0px)", "0px")
        .replace("calc", "");
    eval(&resolved)
}

/// Sums `expr`'s `+`-separated terms, each a length or a length `/` a number, brackets ignored.
fn eval(expr: &str) -> f64 {
    let flat: String = expr.chars().filter(|c| !matches!(c, '(' | ')')).collect();
    flat.split('+')
        .map(|term| {
            let (length, divisor) = term
                .split_once('/')
                .map_or((term, 1.0), |(l, d)| (l, d.trim().parse().unwrap_or(1.0)));
            let length = length.trim();
            let number = |unit: &str| {
                length
                    .strip_suffix(unit)
                    .and_then(|n| n.parse::<f64>().ok())
            };
            number("rem").map_or_else(|| number("px").unwrap_or(f64::NAN), |rem| rem * 16.0)
                / divisor
        })
        .sum()
}

#[test]
fn one_corner_gap_from_the_home_indicator_places_the_dock_the_toasts_and_the_banners() {
    assert_eq!(
        value(rule(COMPONENTS, ":root"), "--corner-gap"),
        Some("calc(env(safe-area-inset-bottom, 0px) / 2 + var(--space-2))"),
        "half the home indicator's strip plus a margin, clear of the rounded corner"
    );
    assert_eq!(
        value(rule(COMPONENTS, ".action-dock"), "--dock-gap"),
        Some("var(--corner-gap)")
    );
    for (css, selector) in [(TOAST, ".toast-container"), (COMPONENTS, ".banner-stack")] {
        let stack = rule(css, selector);
        assert_eq!(
            value(stack, "left"),
            Some("var(--corner-gap)"),
            "{selector}"
        );
        assert_eq!(
            value(stack, "bottom"),
            Some("calc(var(--corner-gap) + var(--keyboard-inset, 0px))"),
            "{selector}: rides the keyboard, as the dock does"
        );
        assert_eq!(
            value(stack, "top"),
            None,
            "nothing anchors {selector} to the top"
        );
    }
}

#[test]
fn the_toast_s_side_and_bottom_gaps_match_with_and_without_a_home_indicator() {
    let stack = rule(TOAST, ".toast-container");
    let left = value(stack, "left").unwrap_or_default();
    let bottom = value(stack, "bottom").unwrap_or_default();
    // An iPhone in portrait: a 34px home indicator strip and no side safe area. Then the web.
    for (bottom_inset, device) in [(34.0, "iPhone"), (0.0, "web")] {
        let (l, b) = (px(left, bottom_inset), px(bottom, bottom_inset));
        assert!((l - b).abs() < 0.01, "{device}: left {l}px, bottom {b}px");
    }
    assert!(
        (px(left, 34.0) - 25.0).abs() < 0.01,
        "17px of the strip plus 8px"
    );
    let dock = value(rule(COMPONENTS, ".action-dock"), "--dock-gap").unwrap_or_default();
    assert!(
        (px(dock, 34.0) - px(left, 34.0)).abs() < 0.01,
        "the toast and the [+] sit on the same line from their corners"
    );
}

#[test]
fn the_toast_takes_the_panel_corner_and_the_close_button_nests_inside_it() {
    let toast = rule(TOAST, ".toast-container [role=\"alertdialog\"]");
    assert_eq!(value(toast, "border-radius"), Some("var(--panel-radius)"));
    assert_eq!(value(toast, "padding"), Some("var(--panel-pad)"));
    // The × sits var(--space-2) in from the toast's edge: its padding less its negative margin.
    assert_eq!(
        value(
            rule(TOAST, ".toast-container [role=\"alertdialog\"] button"),
            "border-radius"
        ),
        Some("max(0.15rem, calc(var(--panel-radius) - 1px - var(--space-2)))")
    );
}

#[test]
fn a_toast_rises_from_below_the_bottom_edge_by_translate_alone() {
    let from = keyframe(TOAST, "toast-rise-in", "from");
    assert_eq!(value(from, "translate"), Some(BELOW_THE_EDGE));
    assert_eq!(value(from, "opacity"), None, "it moves; it does not fade");
    assert_eq!(
        value(keyframe(TOAST, "toast-rise-in", "to"), "translate"),
        Some("0")
    );
}

/// Where a leaving toast drifts to as it fades: a little way up from its spot.
const DRIFT: &str = "translateY(-0.75rem)";

/// The same drift as the timed keyframes write it.
const DRIFT_TRANSLATE: &str = "0 -0.75rem";

#[test]
fn a_timed_toast_rises_from_below_and_leaves_drifting_up_as_it_fades() {
    for (name, held) in [("toast-life-3000", "92%"), ("toast-life-5000", "95%")] {
        assert_eq!(
            value(keyframe(TOAST, name, "0%"), "translate"),
            Some(BELOW_THE_EDGE),
            "{name} rises from below"
        );
        let up = keyframe(TOAST, name, held);
        assert_eq!(value(up, "translate"), Some("0"), "{name} {held}");
        assert_eq!(value(up, "opacity"), Some("1"), "{name} {held}");
        let gone = keyframe(TOAST, name, "100%");
        assert_eq!(
            value(gone, "translate"),
            Some(DRIFT_TRANSLATE),
            "{name} drifts up"
        );
        assert_eq!(value(gone, "opacity"), Some("0"), "{name} fades out");
    }
}

#[test]
fn under_reduced_motion_a_timed_toast_only_fades_out() {
    for (name, held) in [("toast-still-3000", "92%"), ("toast-still-5000", "95%")] {
        assert_eq!(
            value(keyframe(TOAST, name, held), "opacity"),
            Some("1"),
            "{name}"
        );
        let gone = keyframe(TOAST, name, "100%");
        assert_eq!(value(gone, "opacity"), Some("0"), "{name}");
        assert_eq!(value(gone, "transform"), None, "{name} never moves");
        assert_eq!(value(gone, "translate"), None, "{name} never moves");
    }
}

#[test]
fn a_toast_that_leaves_drifts_up_and_fades_out() {
    let js = orangui::TOAST_LEAVE_JS;
    assert!(js.contains("new MutationObserver"), "{js}");
    assert!(
        js.contains(&format!("{{ transform: '{DRIFT}', opacity: 0 }}")),
        "it drifts up a little as it fades: {js}"
    );
    assert!(
        js.contains("{ opacity: 0 }"),
        "under reduced motion it only fades: {js}"
    );
    assert!(
        !js.contains("innerHeight - box.top"),
        "it no longer falls below the edge: {js}"
    );
    assert!(
        js.contains("animationName.startsWith('toast-life')"),
        "a timed toast that already faded by its own keyframes is not faded again: {js}"
    );
    assert!(
        TOAST_STACK.contains("document::eval(crate::TOAST_LEAVE_JS)"),
        "ToastStack installs it"
    );
}

#[test]
fn stacked_toasts_line_up_behind_the_newest_which_stays_in_front() {
    // Collapsed, an older toast adds nothing to the stack's width and takes the newest's.
    let older = rule(
        TOAST,
        ".toast-container:not(.expanded) li:not(:first-child)",
    );
    assert_eq!(value(older, "width"), Some("0"));
    assert_eq!(value(older, "min-width"), Some("100%"));
    let stretched = rule(
        TOAST,
        ".toast-container:not(.expanded) li:not(:first-child) [role=\"alertdialog\"]",
    );
    assert_eq!(value(stretched, "width"), Some("100%"));
    // The library renders the newest first, and the first paints over the rest.
    let z = |n: u8| {
        value(
            rule(TOAST, &format!(".toast-container li:nth-child({n})")),
            "z-index",
        )
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0)
    };
    assert!(z(1) > z(2) && z(2) > z(3), "the newest is in front");
    assert!(
        orangui::TOAST_LEAVE_JS.contains("front ? '10000' : '9998'"),
        "a leaving newest fades in front of the rest, an older one behind them"
    );
}

#[test]
fn a_width_change_eases_and_older_toasts_keep_to_one_line() {
    let js = orangui::TOAST_WIDTH_JS;
    assert!(js.contains("toast.animate([{ width:"), "{js}");
    assert!(
        js.contains("prefers-reduced-motion: reduce"),
        "it jumps under reduced motion: {js}"
    );
    assert!(
        js.contains("offsetWidth"),
        "it reads layout widths, so the arrival's scale is no change: {js}"
    );
    assert!(
        TOAST_STACK.contains("document::eval(crate::TOAST_WIDTH_JS)"),
        "ToastStack installs it"
    );
    let older = TOAST
        .split(".toast-container:not(.expanded) li:not(:first-child) [id$=\"-label\"],")
        .nth(1)
        .expect("older toasts' label rule");
    let rule = older.split('}').next().expect("rule ends");
    assert!(rule.contains("white-space: nowrap"), "{rule}");
    assert!(rule.contains("text-overflow: ellipsis"), "{rule}");
}
