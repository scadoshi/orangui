//! The opening: a short overlay over the first screen while it mounts and
//! styles underneath. The backdrop rains in, the host's mark resolves over
//! it and then leaves the way it came, and the whole thing fades out and
//! unmounts. Styled by `assets/app.css`.
//!
//! The host hands in its mark, already dressed the way its home screen draws
//! it, so the two match:
//!
//! ```ignore
//! Splash { pre { class: "logo", Decode { text: MARK } } }
//! ```
//!
//! A prerendered site passes `prerendered: true`. The mark is then in the
//! page's HTML and `assets/splash.js` runs the whole opening from first paint,
//! so it never waits on the app's download; the app only waits for the fade,
//! opens the [`SplashGate`] and unmounts the overlay. The script styles
//! nothing itself: it sets `splash-running`, `splash-fading` and
//! `splash-over` on the root, and the host's sheet shows the mark under the
//! first, fades the overlay under the second and hides it under the third.

use std::fmt;

use dioxus::prelude::*;

use crate::entrance;
use crate::grid_cells::GridCells;

/// Whether the opening is over. A host provides one at its root, closed; the
/// [`Splash`] opens it as it starts to fade, and the entrances ([`crate::CountUp`],
/// [`crate::Figure`], [`crate::Decode`]) wait on it, so the first screen's
/// figures roll and its marks resolve as the overlay leaves rather than
/// underneath it. A host with no splash provides nothing and they run on mount.
#[derive(Clone, Copy)]
pub struct SplashGate(pub Signal<bool>);

impl fmt::Debug for SplashGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SplashGate").finish_non_exhaustive()
    }
}

/// Tells the splash's mark to leave. The [`Splash`] provides it, closed, to its
/// children and opens it once the mark has resolved; a [`crate::Decode`] under
/// it then runs its way out.
/// It also carries the splash's pace, so the mark's passes keep step with it,
/// and whether the page's script runs the mark instead.
#[derive(Clone, Copy)]
pub(crate) struct SplashLeave(pub(crate) Signal<bool>, pub(crate) u32, pub(crate) bool);

/// The opening on a prerendered page, run before the app is up. Inlined once;
/// it guards itself.
pub const SPLASH_JS: &str = include_str!("../assets/splash.js");

/// When the mark mounts, so it resolves as the field is landing.
const MARK_AT_MS: u32 = 400;
/// When the mark starts to leave, just after it has resolved.
const LEAVE_AT_MS: u32 = 1350;
/// When the fade starts: once the mark has left.
const OUT_AT_MS: u32 = LEAVE_AT_MS + entrance::EXIT_MS;
/// The fade itself; matches the transition in `app.css`.
const FADE_MS: u32 = 300;

/// `ms` scaled to `pace`, a percent of the full timeline.
fn paced(ms: u32, pace: u32) -> u32 {
    ms.saturating_mul(pace) / 100
}

/// The overlay. `children` is the mark; it mounts once the field is landing.
/// `pace` runs the timeline, the mark's passes included, at that percent of
/// its full length; the fade itself keeps its length. `prerendered` hands the
/// timeline to the page's script, as the module doc says.
#[component]
pub fn Splash(
    children: Element,
    #[props(default = 100)] pace: u32,
    #[props(default)] prerendered: bool,
) -> Element {
    let mut mark = use_signal(|| false);
    let mut out = use_signal(|| false);
    let mut gone = use_signal(|| false);
    // The host's gate, read before this scope provides its own open one for
    // the mark, so the mark resolves on the overlay while the screen waits.
    let host_gate = try_consume_context::<SplashGate>();
    use_context_provider(|| SplashGate(Signal::new(true)));
    let mut leave = use_context_provider(|| SplashLeave(Signal::new(false), pace, prerendered)).0;
    use_hook(move || {
        spawn(async move {
            if prerendered {
                // Settles as the script's fade starts, at once if it is over.
                let mut eval = document::eval(
                    "const s = window.__oranguiSplash; if (s) s.out.then(() => dioxus.send(true)); else dioxus.send(true);",
                );
                let _ = eval.recv::<bool>().await;
            } else {
                entrance::sleep_ms(paced(MARK_AT_MS, pace)).await;
                mark.set(true);
                entrance::sleep_ms(paced(LEAVE_AT_MS - MARK_AT_MS, pace)).await;
                leave.set(true);
                entrance::sleep_ms(paced(OUT_AT_MS - LEAVE_AT_MS, pace)).await;
                out.set(true);
            }
            if let Some(mut gate) = host_gate {
                gate.0.set(true);
            }
            entrance::sleep_ms(FADE_MS).await;
            gone.set(true);
        });
    });
    if gone() {
        return rsx! {};
    }
    if prerendered {
        let decode_ms = u32::try_from(entrance::DECODE_MS).unwrap_or(u32::MAX);
        return rsx! {
            document::Script { {SPLASH_JS} }
            div {
                class: "splash",
                aria_hidden: "true",
                "data-prerendered": "true",
                "data-mark-at": "{paced(MARK_AT_MS, pace)}",
                "data-decode": "{paced(decode_ms, pace)}",
                "data-leave-at": "{paced(LEAVE_AT_MS, pace)}",
                "data-exit": "{paced(entrance::EXIT_MS, pace)}",
                "data-out-at": "{paced(OUT_AT_MS, pace)}",
                "data-fade": "{FADE_MS}",
                "data-tick": "{entrance::DECODE_TICK_MS}",
                GridCells {}
                div { class: "splash-mark", {children} }
            }
        };
    }
    rsx! {
        div {
            class: if out() { "splash splash-out" } else { "splash" },
            aria_hidden: "true",
            GridCells {}
            if mark() {
                div { class: "splash-mark", {children} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_reads_every_timing_the_overlay_carries() {
        let source = include_str!("splash.rs");
        for (attribute, key) in [
            ("data-mark-at", "markAt"),
            ("data-decode", "decode"),
            ("data-leave-at", "leaveAt"),
            ("data-exit", "exit"),
            ("data-out-at", "outAt"),
            ("data-fade", "fade"),
            ("data-tick", "tick"),
        ] {
            assert!(source.contains(&format!("\"{attribute}\":")), "{attribute}");
            assert!(SPLASH_JS.contains(&format!("at('{key}')")), "{key}");
        }
        assert!(SPLASH_JS.contains(".splash[data-prerendered]"));
        assert!(SPLASH_JS.contains("__oranguiSplash"));
    }
}
