//! The entrance: how a page's headline pieces arrive.
//!
//! A number counts up from zero ([`CountUp`]), a formatted figure rolls its
//! digits into place ([`Figure`]), block-glyph art resolves from noise left to
//! right ([`Decode`]), and all of them run again whenever the page's
//! [`Replay`] counter is bumped, which the nav logo does on a click. A page
//! that provides no counter gets each piece once, on mount.
//!
//! Everything is prerendered settled, so the page reads right before any
//! script runs, and the motion only ever starts from what is already there.
//! A viewer who asks for less motion gets the settled value at once.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use orangui_core::{count_up, decode, number};

use crate::splash::{SplashGate, SplashLeave};

/// How many times the page has been asked to run its entrance again. The nav
/// logo bumps it; every `CountUp` and `Decode` under it starts over.
#[derive(Clone, Copy)]
pub struct Replay(pub Signal<u32>);

impl fmt::Debug for Replay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Replay").finish_non_exhaustive()
    }
}

/// The page's replay counter, or a private one that never moves when the page
/// provides none.
fn use_replay() -> Signal<u32> {
    let own = use_signal(|| 0u32);
    try_consume_context::<Replay>().map_or(own, |replay| replay.0)
}

/// The host's [`SplashGate`], if it has one. Read inside an entrance's effect
/// so the flip to open runs it; with no gate the entrance runs on mount.
fn use_gate() -> Option<SplashGate> {
    try_consume_context::<SplashGate>()
}

fn open(gate: Option<SplashGate>) -> bool {
    gate.is_none_or(|gate| gate.0())
}

/// Whether the viewer has asked for less motion. Asked of the page, so it
/// holds in the app's WebView too, through the `dioxus.send` channel the app
/// reads its other measurements over; `false` when nothing answers.
async fn reduced_motion() -> bool {
    let mut eval = document::eval(
        "dioxus.send(window.matchMedia('(prefers-reduced-motion: reduce)').matches);",
    );
    eval.recv::<bool>().await.unwrap_or(false)
}

/// Browser `setTimeout` as a future on the web, tokio's timer elsewhere.
pub(crate) async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(u64::from(ms))).await;
}

/// A number that rolls to its value: random digits inside a window that
/// narrows to nothing, printed with thousands separators. It never shows a
/// blank or a climb from zero.
///
/// `None` is a figure that has not arrived. It rolls on, for as long as that
/// takes, and lands when the value turns up; a value that changes mid-roll
/// becomes the new target the same way.
#[component]
pub fn CountUp(
    value: Option<u64>,
    /// Print `28.0k` and `1.2m` rather than every digit, for a figure in a
    /// box too narrow for all of them. Below ten thousand it prints in full.
    #[props(default)]
    compact: bool,
) -> Element {
    let replay = use_replay();
    // Its own seed, so three figures waiting side by side do not roll the
    // same digits in step.
    let seed = use_hook(|| count_up::seed_for(SEEDS.fetch_add(1, Ordering::Relaxed)));
    let mut target = use_signal(|| value);
    let mut shown = use_signal(|| value);
    // Set once the viewer turns out to want less motion: from then on the
    // number shows each value as it arrives, with no roll.
    let mut still = use_signal(|| false);
    let gate = use_gate();
    use_effect(use_reactive!(|value| target.set(value)));
    use_effect(move || {
        let value = target();
        if still() {
            shown.set(value);
        }
    });
    use_effect(move || {
        let run = replay();
        if !open(gate) {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                still.set(true);
                return;
            }
            // Nothing to land on yet: roll at a fixed width until there is.
            let mut tick = 0u64;
            while target.peek().is_none() {
                sleep_ms(count_up::TICK_MS).await;
                if replay.peek().ne(&run) {
                    return;
                }
                tick = tick.wrapping_add(1);
                shown.set(Some(count_up::waiting(seed, tick)));
            }
            for frame in 1..=count_up::FRAMES {
                sleep_ms(count_up::TICK_MS).await;
                // A later replay owns the number now.
                if replay.peek().ne(&run) {
                    return;
                }
                let Some(value) = *target.peek() else {
                    // It went back to waiting; the next effect run picks it up.
                    return;
                };
                shown.set(Some(count_up::count_frame(value, frame)));
            }
            shown.set(*target.peek());
        });
    });
    let print = if compact {
        number::short
    } else {
        number::with_separators
    };
    rsx! {
        match shown() {
            Some(number) => rsx! { "{print(number)}" },
            // Before any frame runs there is nothing honest to print.
            None => rsx! { "{count_up::PLACEHOLDER}" },
        }
    }
}

/// Hands each `CountUp` its own starting seed.
static SEEDS: AtomicU64 = AtomicU64::new(1);

/// A figure the caller has already formatted (`12.3k`, `1,234.6`, `87%`,
/// `+606`) whose digits roll into place: every digit flickers through random
/// ones and settles over a second, while the separators, the sign, the unit
/// and anything else stay put, so the width never moves. Rolls on mount, on
/// every replay, and again whenever `text` changes, which is what makes a tap
/// on a counter visibly move its figure.
///
/// `start` holds the roll until it reads true: a card below the fold passes
/// the signal its reveal flips, so its numbers roll as it scrolls into view
/// rather than unseen at mount. A viewer who asks for less motion gets the
/// settled text at once.
#[component]
pub fn Figure(text: String, #[props(default)] start: Option<Signal<bool>>) -> Element {
    let replay = use_replay();
    let seed = use_hook(|| count_up::seed_for(SEEDS.fetch_add(1, Ordering::Relaxed)));
    let mut target = use_signal(|| text.clone());
    let mut shown = use_signal(|| text.clone());
    let gate = use_gate();
    use_effect(use_reactive!(|text| target.set(text)));
    use_effect(move || {
        let run = replay();
        let goal = target();
        // Reading them here subscribes the effect, so a flip to true runs it.
        let go = start.is_none_or(|start| start()) && open(gate);
        if !go {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                shown.set(goal);
                return;
            }
            for frame in 1..=count_up::FRAMES {
                sleep_ms(count_up::TICK_MS).await;
                if replay.peek().ne(&run) || target.peek().ne(&goal) {
                    // A newer roll owns the figure now.
                    return;
                }
                shown.set(count_up::figure_frame(&goal, frame, seed));
            }
            shown.set(goal);
        });
    });
    rsx! { "{shown()}" }
}

/// Block-glyph art that builds left to right in two waves: a fast wave of
/// noise draws the shape out of blank, and a slower decode wave follows it,
/// turning the noise into the art. The blank columns are spaces, so the art
/// keeps its size while it builds.
///
/// `hover` is a signal the caller sets from the element around this one.
/// While it is true the art keeps resolving, pass after pass, and settles on
/// the pass that finishes after it goes false.
///
/// Inside a [`crate::Splash`] the art also leaves: when the splash says so it
/// runs the same two waves in reverse, the art turning to noise and the noise
/// clearing, left to right. Inside a prerendered one the page's script runs
/// both passes instead (`assets/splash.js`).
#[component]
pub fn Decode(text: &'static str, #[props(default)] hover: Option<Signal<bool>>) -> Element {
    let replay = use_replay();
    let gate = use_gate();
    let leave = try_consume_context::<SplashLeave>();
    // Under a prerendered splash the page's script runs the art, so this
    // renders it whole into the HTML for the script to find and never writes.
    let scripted = leave.is_some_and(|leave| leave.2);
    let mut shown = use_signal(|| {
        if scripted {
            text.to_string()
        } else {
            decode::blank(text)
        }
    });
    let leaving = move || leave.is_some_and(|leave| *leave.0.peek());
    // A splash's pace, in percent, shortens both passes with its timeline.
    let pace = leave.map_or(100, |leave| leave.1);
    use_effect(move || {
        let run = replay();
        if scripted || !open(gate) {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                shown.set(text.to_string());
                return;
            }
            let ms = u64::from(decode::DURATION_MS).saturating_mul(u64::from(pace)) / 100;
            resolve(text, shown, ms, || replay.peek().ne(&run) || leaving()).await;
        });
    });
    use_effect(move || {
        if !leave.is_some_and(|leave| (leave.0)()) {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                return;
            }
            evaporate(text, shown, decode::EXIT_MS.saturating_mul(pace) / 100).await;
        });
    });
    // Under the pointer a band of noise travels across the art, over and over,
    // so it shimmers rather than dissolving and resolving in turns.
    use_effect(move || {
        if !hover.is_some_and(|hover| hover()) {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                return;
            }
            let columns = decode::columns(text);
            let mut center: usize = 0;
            while hover.is_some_and(|hover| *hover.peek()) {
                sleep_ms(decode::TICK_MS).await;
                center = decode::next_shimmer_center(center, columns);
                shown.set(decode::shimmer_frame(text, center));
            }
            shown.set(text.to_string());
        });
    });
    rsx! { "{shown()}" }
}

/// One pass of the front across `text`, writing each frame into `shown`.
/// Stops early when `give_up` says someone else owns the art now.
async fn resolve(
    text: &'static str,
    mut shown: Signal<String>,
    ms: u64,
    give_up: impl Fn() -> bool,
) {
    let frames = decode::frames(ms);
    for frame in 1..=frames {
        sleep_ms(decode::TICK_MS).await;
        if give_up() {
            return;
        }
        shown.set(decode::entry_frame(text, frame, frames));
    }
    shown.set(text.to_string());
}

/// The way out, over `ms`: the two waves of [`resolve`] in reverse, a fast
/// wave turning the art to noise and a slower one clearing the noise.
async fn evaporate(text: &'static str, mut shown: Signal<String>, ms: u32) {
    let frames = decode::frames(u64::from(ms));
    for frame in 1..=frames {
        sleep_ms(decode::TICK_MS).await;
        shown.set(decode::exit_frame(text, frame, frames));
    }
    shown.set(decode::blank(text));
}
