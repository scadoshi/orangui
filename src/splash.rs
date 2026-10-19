//! The opening: a short overlay over the first screen while it mounts and
//! styles underneath. The backdrop rains in, the host's mark resolves over
//! it, and the whole thing fades out and unmounts. Styled by `assets/app.css`.
//!
//! The host hands in its mark, already dressed the way its home screen draws
//! it, so the two match:
//!
//! ```ignore
//! Splash { pre { class: "logo", Decode { text: MARK } } }
//! ```

use dioxus::prelude::*;

use crate::{entrance::sleep_ms, grid_cells::GridCells};

/// When the mark mounts, so it resolves as the field is landing.
const MARK_AT_MS: u32 = 400;
/// When the fade starts.
const OUT_AT_MS: u32 = 1300;
/// The fade itself; matches the transition in `app.css`.
const FADE_MS: u32 = 300;

/// The overlay. `children` is the mark; it mounts once the field is landing.
#[component]
pub fn Splash(children: Element) -> Element {
    let mut mark = use_signal(|| false);
    let mut out = use_signal(|| false);
    let mut gone = use_signal(|| false);
    use_hook(move || {
        spawn(async move {
            sleep_ms(MARK_AT_MS).await;
            mark.set(true);
            sleep_ms(OUT_AT_MS - MARK_AT_MS).await;
            out.set(true);
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
