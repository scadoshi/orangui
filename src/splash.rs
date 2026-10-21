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

use dioxus::prelude::*;

use crate::{entrance::sleep_ms, grid_cells::GridCells};

/// Whether the opening is over. A host provides one at its root, closed; the
/// [`Splash`] opens it as it starts to fade, and the entrances ([`crate::CountUp`],
/// [`crate::Figure`], [`crate::Decode`]) wait on it, so the first screen's
/// figures roll and its marks resolve as the overlay leaves rather than
/// underneath it. A host with no splash provides nothing and they run on mount.
#[derive(Clone, Copy)]
pub struct SplashGate(pub Signal<bool>);

/// Tells the splash's mark to leave. The [`Splash`] provides it, closed, to its
/// children and opens it once the mark has resolved; a [`crate::Decode`] under
/// it then runs its way out.
/// It also carries the splash's pace, so the mark's passes keep step with it.
#[derive(Clone, Copy)]
pub(crate) struct SplashLeave(pub(crate) Signal<bool>, pub(crate) u32);

/// When the mark mounts, so it resolves as the field is landing.
const MARK_AT_MS: u32 = 400;
/// When the mark starts to leave, just after it has resolved.
const LEAVE_AT_MS: u32 = 1350;
/// When the fade starts: once the mark has left.
const OUT_AT_MS: u32 = LEAVE_AT_MS + crate::entrance::EXIT_MS;
/// The fade itself; matches the transition in `app.css`.
const FADE_MS: u32 = 300;

/// `ms` scaled to `pace`, a percent of the full timeline.
fn paced(ms: u32, pace: u32) -> u32 {
    ms * pace / 100
}

/// The overlay. `children` is the mark; it mounts once the field is landing.
/// `pace` runs the timeline, the mark's passes included, at that percent of
/// its full length; the fade itself keeps its length.
#[component]
pub fn Splash(children: Element, #[props(default = 100)] pace: u32) -> Element {
    let mut mark = use_signal(|| false);
    let mut out = use_signal(|| false);
    let mut gone = use_signal(|| false);
    // The host's gate, read before this scope provides its own open one for
    // the mark, so the mark resolves on the overlay while the screen waits.
    let host_gate = try_consume_context::<SplashGate>();
    use_context_provider(|| SplashGate(Signal::new(true)));
    let mut leave = use_context_provider(|| SplashLeave(Signal::new(false), pace)).0;
    use_hook(move || {
        spawn(async move {
            sleep_ms(paced(MARK_AT_MS, pace)).await;
            mark.set(true);
            sleep_ms(paced(LEAVE_AT_MS - MARK_AT_MS, pace)).await;
            leave.set(true);
            sleep_ms(paced(OUT_AT_MS - LEAVE_AT_MS, pace)).await;
            out.set(true);
            if let Some(mut gate) = host_gate {
                gate.0.set(true);
            }
            sleep_ms(FADE_MS).await;
            gone.set(true);
        });
    });
    if gone() {
        return rsx! {};
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
