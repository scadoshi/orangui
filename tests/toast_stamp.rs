//! A toast stamps as it lands, in one motion with its rise: it comes up from below the edge a
//! little larger than its size and shrinks to it as it rises, both on one ease-out curve, so it
//! arrives at its size and nothing moves once it is in its spot. Transform alone, and not at all
//! under reduced motion. A keyed toast's update changes it in place: no rise and no
//! stamp, its life starting over where it sits, and the part of its text that changed crossfades
//! from the old to the new.

const TOAST: &str = include_str!("../assets/toast.css");
const TOAST_STACK: &str = include_str!("../src/toast_stack.rs");
const LIB: &str = include_str!("../src/lib.rs");

/// What a toast's entrance starts from: wholly below the edge.
const BELOW_THE_EDGE: &str =
    "0 calc(100% + var(--corner-gap) + var(--keyboard-inset, 0px) + var(--space-4))";

/// How large a toast starts its rise: it shrinks to its own size as it arrives.
const STAMP_FROM: &str = "1.08";

/// The one curve a toast's rise and shrink share.
const RISE_CURVE: &str = "var(--ease-enter, ease-out)";

/// The toast at rest in its spot.
const REST: &str = "0";

/// The declarations of the first rule whose selector is exactly `selector` in `css`, or `""`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    css.split_once(opening.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

/// A stop of a toast's keyframes: its offset, and the translate, scale and timing function it
/// sets, if any.
type Stop<'css> = (
    &'css str,
    Option<&'css str>,
    Option<&'css str>,
    Option<&'css str>,
);

/// The stops of `@keyframes name` in order.
fn stops<'css>(css: &'css str, name: &str) -> Vec<Stop<'css>> {
    let Some((_, rest)) = css.split_once(&format!("@keyframes {name} {{")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut rest = rest;
    while let Some((head, after)) = rest.split_once('{') {
        // The keyframes' own closing brace ends them; a stop's comment sits before it.
        if head.contains('}') {
            break;
        }
        let stop = head.rsplit_once("*/").map_or(head, |(_, stop)| stop).trim();
        if stop.is_empty() {
            break;
        }
        let Some((body, after)) = after.split_once('}') else {
            break;
        };
        let set = |property: &str| {
            body.lines()
                .find_map(|line| line.trim().strip_prefix(property))
                .map(|v| v.trim().trim_end_matches(';'))
        };
        out.push((
            stop,
            set("translate:"),
            set("scale:"),
            set("animation-timing-function:"),
        ));
        rest = after;
    }
    out
}

#[test]
fn a_toast_comes_in_large_and_lands_at_its_size_with_nothing_after() {
    assert_eq!(
        stops(TOAST, "toast-rise-in"),
        [
            ("from", Some(BELOW_THE_EDGE), Some(STAMP_FROM), None),
            ("to", Some(REST), Some("1"), None),
        ],
        "one stretch from below and large to its spot at its size: no peak and no settle"
    );
    let toast = rule(TOAST, ".toast-container [role=\"alertdialog\"]");
    assert!(
        toast.contains(&format!(
            "animation: toast-rise-in var(--enter-duration, 280ms) {RISE_CURVE} both;"
        )),
        "the rise's length alone, on the one curve: {toast}"
    );
}

#[test]
fn a_timed_toast_comes_in_large_and_lands_at_its_size_with_nothing_after() {
    for (name, lands, held) in [
        ("toast-life-3000", "7%", "92%"),
        ("toast-life-5000", "4%", "95%"),
    ] {
        let got = stops(TOAST, name);
        assert_eq!(
            got.get(..3),
            Some(
                [
                    (
                        "0%",
                        Some(BELOW_THE_EDGE),
                        Some(STAMP_FROM),
                        Some(RISE_CURVE)
                    ),
                    (lands, Some(REST), Some("1"), None),
                    (held, Some(REST), None, None),
                ]
                .as_slice()
            ),
            "{name}: the rise and shrink end together, and it holds still until it leaves: {got:?}"
        );
    }
}

#[test]
fn no_script_stamps_a_toast_after_it_lands() {
    assert!(
        !LIB.contains("TOAST_STAMP_JS"),
        "the stamp is the rise's own keyframes"
    );
    assert!(!TOAST_STACK.contains("TOAST_STAMP_JS"));
}

#[test]
fn a_keyed_toast_s_update_holds_its_place_and_starts_its_life_over() {
    let plain = rule(
        TOAST,
        ".toast-container:not(.timed-fade) .toast-tap[data-live-update] > [role=\"alertdialog\"]",
    );
    assert!(plain.contains("animation: none;"), "no rise again: {plain}");
    let info = rule(
        TOAST,
        ".toast-container.timed-fade .toast-tap[data-live-update] > [data-type=\"info\"]",
    );
    assert!(
        info.contains("animation-name: toast-hold-3000;"),
        "its life over, from where it sits: {info}"
    );
    assert!(
        TOAST.contains("@keyframes toast-hold-3000 {\n    0% {\n        transform: translateY(0);"),
        "starts in place"
    );
}

#[test]
fn an_update_crossfades_the_part_that_changed_by_opacity() {
    let old = rule(TOAST, ".toast-live-old");
    assert!(old.contains("position: absolute;"), "{old}");
    assert!(old.contains("animation: toast-live-out"), "{old}");
    let new = rule(TOAST, ".toast-tap[data-live-update] .toast-live-new");
    assert!(new.contains("animation: toast-live-in"), "{new}");
    assert!(
        TOAST.contains("@keyframes toast-live-in {\n    from {\n        opacity: 0;"),
        "fades in by opacity alone"
    );
    assert!(
        TOAST.contains(".toast-live-old {\n        display: none;"),
        "under reduced motion the new number simply shows"
    );
}
