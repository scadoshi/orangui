//! A toast stamps as it lands: once it has risen into its spot it jumps a little and grows, then
//! settles, as a stack badge pops, by transform alone and not at all under reduced motion. A keyed
//! toast's update changes it in place: no rise and no stamp, its life starting over where it sits,
//! and the part of its text that changed crossfades from the old to the new.

const TOAST: &str = include_str!("../assets/toast.css");
const STAMP: &str = include_str!("../assets/toast-stamp.js");

/// The declarations of the first rule whose selector is exactly `selector` in `css`, or `""`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    css.split_once(opening.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_toast_stamps_once_it_has_landed_as_a_badge_pops() {
    for want in [
        "[{ transform: 'none' }, { transform: 'translateY(-2px) scale(1.04)', offset: 0.4 }, { transform: 'none' }]",
        "duration: STAMP_MS, delay: rise(li), easing: ease(), composite: 'add'",
        "const STAMP_MS = 200;",
        "matchMedia('(prefers-reduced-motion: reduce)').matches",
        "li.querySelector('[data-live-update]')",
        "closest('.toast-ghost')",
    ] {
        assert!(STAMP.contains(want), "{want} in {STAMP}");
    }
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
