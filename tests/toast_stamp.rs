//! A toast stamps as it lands, in one motion with its rise: it comes up from below the edge, carries
//! a little past its spot as it grows, as a stack badge pops, and settles, by transform alone and
//! not at all under reduced motion. A keyed toast's update changes it in place: no rise and no
//! stamp, its life starting over where it sits, and the part of its text that changed crossfades
//! from the old to the new.

const TOAST: &str = include_str!("../assets/toast.css");
const TOAST_STACK: &str = include_str!("../src/toast_stack.rs");
const LIB: &str = include_str!("../src/lib.rs");

/// What a toast's entrance starts from: wholly below the edge.
const BELOW_THE_EDGE: &str =
    "0 calc(100% + var(--corner-gap) + var(--keyboard-inset, 0px) + var(--space-4))";

/// The stamp's peak: a little above the toast's spot.
const PEAK: &str = "0 -2px";

/// The toast at rest in its spot.
const REST: &str = "0";

/// The declarations of the first rule whose selector is exactly `selector` in `css`, or `""`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    css.split_once(opening.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

/// A stop of a toast's keyframes: its offset, and the translate and scale it sets, if any.
type Stop<'css> = (&'css str, Option<&'css str>, Option<&'css str>);

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
        out.push((stop, set("translate:"), set("scale:")));
        rest = after;
    }
    out
}

#[test]
fn a_toast_rises_into_its_stamp_and_settles_in_one_motion() {
    assert_eq!(
        stops(TOAST, "toast-rise-in"),
        [
            ("from", Some(BELOW_THE_EDGE), Some("1")),
            ("55%", None, Some("1")),
            ("70%", Some(PEAK), Some("1.04")),
            ("to", Some(REST), Some("1")),
        ],
        "no stop at rest between the rise and the stamp, and it grows only as it lands"
    );
    let toast = rule(TOAST, ".toast-container [role=\"alertdialog\"]");
    assert!(
        toast.contains(
            "animation: toast-rise-in calc(var(--enter-duration, 280ms) + 120ms) var(--ease-enter, ease-out) both;"
        ),
        "the rise's length plus the settle: {toast}"
    );
}

#[test]
fn a_timed_toast_rises_into_its_stamp_and_settles_in_one_motion() {
    for (name, grows, peak, settled, held) in [
        ("toast-life-3000", "5%", "7%", "11%", "92%"),
        ("toast-life-5000", "3%", "4%", "6.5%", "95%"),
    ] {
        let got = stops(TOAST, name);
        assert_eq!(
            got.get(..5),
            Some(
                [
                    ("0%", Some(BELOW_THE_EDGE), Some("1")),
                    (grows, None, Some("1")),
                    (peak, Some(PEAK), Some("1.04")),
                    (settled, Some(REST), Some("1")),
                    (held, Some(REST), None),
                ]
                .as_slice()
            ),
            "{name}: {got:?}"
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
