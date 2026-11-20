//! A toast stamps as it lands, in one motion with its rise: it comes up from below the edge well
//! over its size, lands in its spot at its size, squashes a touch under it, rebounds a touch over
//! it and settles, one set of keyframes with a curve per stretch. Transform and shadow alone, and
//! not at all under reduced motion. A keyed toast's update changes it in place: no rise and no
//! stamp, its life starting over where it sits, and the part of its text that changed crossfades
//! from the old to the new.

const TOAST: &str = include_str!("../assets/toast.css");
const TOAST_STACK: &str = include_str!("../src/toast_stack.rs");
const LIB: &str = include_str!("../src/lib.rs");

/// What a toast's entrance starts from: wholly below the edge.
const BELOW_THE_EDGE: &str =
    "0 calc(100% + var(--corner-gap) + var(--keyboard-inset, 0px) + var(--space-4))";

/// How large a toast starts its rise: it is at its own size when it lands.
const STAMP_FROM: &str = "1.28";

/// The squash just after it lands, and the rebound after that.
const SQUASH: &str = "0.96";
const REBOUND: &str = "1.02";

/// The drop into its spot: near linear, a fast attack that still lands with speed.
const DROP: &str = "cubic-bezier(0.35, 0.2, 0.85, 0.7)";

/// Out of the impact and the rebound: an ease-out.
const OUT: &str = "cubic-bezier(0.2, 0.6, 0.4, 1)";

/// From the squash into the rebound: an ease-in-out.
const SWAY: &str = "cubic-bezier(0.4, 0, 0.6, 1)";

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
fn a_toast_stamps_lands_squashes_rebounds_and_settles_in_one_motion() {
    assert_eq!(
        stops(TOAST, "toast-rise-in"),
        [
            ("from", Some(BELOW_THE_EDGE), Some(STAMP_FROM), Some(DROP)),
            ("40%", Some(REST), Some("1"), Some(OUT)),
            ("58%", None, Some(SQUASH), Some(SWAY)),
            ("78%", None, Some(REBOUND), Some(OUT)),
            ("to", Some(REST), Some("1"), None),
        ],
        "lands at its size at 160ms, squashes, rebounds and settles by 400ms"
    );
    let toast = rule(TOAST, ".toast-container [role=\"alertdialog\"]");
    assert!(
        toast.contains("animation: toast-rise-in 400ms both;"),
        "the stops carry their own curves: {toast}"
    );
}

#[test]
fn a_timed_toast_stamps_in_its_first_400ms_then_holds_still() {
    for (name, lands, squash, rebound, settled, held) in [
        (
            "toast-life-3000",
            "5.33%",
            "7.73%",
            "10.4%",
            "13.33%",
            "92%",
        ),
        ("toast-life-5000", "3.2%", "4.64%", "6.24%", "8%", "95%"),
    ] {
        let got = stops(TOAST, name);
        assert_eq!(
            got.get(..6),
            Some(
                [
                    ("0%", Some(BELOW_THE_EDGE), Some(STAMP_FROM), Some(DROP)),
                    (lands, Some(REST), Some("1"), Some(OUT)),
                    (squash, None, Some(SQUASH), Some(SWAY)),
                    (rebound, None, Some(REBOUND), Some(OUT)),
                    (settled, Some(REST), Some("1"), None),
                    (held, Some(REST), None, None),
                ]
                .as_slice()
            ),
            "{name}: {got:?}"
        );
    }
}

#[test]
fn a_toast_lands_with_its_shadow_pressed_short() {
    for name in ["toast-rise-in", "toast-life-3000", "toast-life-5000"] {
        let frames = TOAST
            .split_once(&format!("@keyframes {name} {{"))
            .and_then(|(_, rest)| rest.split_once("\n}"))
            .map_or("", |(body, _)| body);
        let long = frames.find("box-shadow: var(--shadow-lg);");
        let short = frames.find("box-shadow: var(--shadow-sm);");
        let rest = frames.find("box-shadow: var(--shadow-md);");
        assert!(
            long < short && short < rest && long.is_some(),
            "{name}: long in the air, short on impact, resting after"
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
