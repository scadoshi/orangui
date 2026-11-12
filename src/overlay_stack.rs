//! Back-aware overlay stack, for the apps.
//!
//! The OS back intent (the iOS edge swipe, Android's hardware or gesture
//! back) is bridged by [`use_os_back`](crate::use_os_back), which normally
//! routes it to the router's `go_back`. Overlays, anything shown on top of the current
//! screen without a route change (bottom sheets, dialogs, in-place overlay
//! screens), are not routes, so a raw `go_back` would blow past them and
//! leave the screen. This stack lets the back intent close the top-most open
//! overlay first, falling through to the router only when none are open.
//!
//! Each overlay registers a close action while it is open, through
//! [`use_overlay_back`] for one toggled by a signal or
//! [`use_overlay_back_action`] for one that closes through a callback. The
//! back bridge calls [`OverlayBackStack::close_top`]. Because closing flips
//! the overlay's open state, the registration effect then deregisters it: the
//! stack stays truthful with no manual bookkeeping.
//!
//! The same order sets which overlay is drawn on top: an overlay's place in
//! the stack gives its z-index ([`overlay_z`]), so the one opened last sits
//! over every one opened before it, as the back intent closes it first.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// Monotonic source of per-overlay-instance ids. A component takes its id
/// once through `use_hook`, so it stays stable across that instance's
/// re-renders and unique across instances.
static NEXT_OVERLAY_ID: AtomicU64 = AtomicU64::new(0);

fn next_overlay_id() -> u64 {
    NEXT_OVERLAY_ID.fetch_add(1, Ordering::Relaxed)
}

/// The z-index of the overlay tier's first layer: the backdrop of the overlay
/// opened first. Sheets, drawers and full-screen overlays share the tier;
/// dialogs sit above it from 1000 and toasts above those.
pub const OVERLAY_Z: u32 = 200;

/// The z-index of the backdrop of the overlay at `place` in the stack, its
/// panel one above: two layers for each overlay opened before it, on top of
/// [`OVERLAY_Z`].
#[must_use]
pub fn overlay_z(place: usize) -> u32 {
    let place = u32::try_from(place).unwrap_or(u32::MAX);
    OVERLAY_Z.saturating_add(place.saturating_mul(2))
}

/// Where a sheet that lives inside a stacking context of its own, such as a
/// [`NavBar`](crate::NavBar)'s, reports its z-index while it shows, so that
/// container rises to it and the sheet can sit over overlays outside it.
/// The container provides it and reads it; `None` while no sheet in it shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayLift(pub Signal<Option<u32>>);

/// App-level LIFO of the currently open overlays, provided once above the
/// router. A `Copy` handle over a signal; each entry pairs an overlay
/// instance's id with the `Callback` that closes it.
#[derive(Clone, Copy)]
pub struct OverlayBackStack {
    entries: Signal<Vec<(u64, Callback<()>)>>,
}

/// Creates the stack. Provide it with `use_context_provider` above the router
/// so both the overlays and the back bridge can reach it.
pub fn use_overlay_back_stack() -> OverlayBackStack {
    OverlayBackStack {
        entries: use_signal(Vec::new),
    }
}

impl fmt::Debug for OverlayBackStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayBackStack").finish_non_exhaustive()
    }
}

impl OverlayBackStack {
    /// Registers an open overlay's close action. Re-pushing the same instance
    /// moves it to the top rather than duplicating it.
    fn push(&mut self, id: u64, close: Callback<()>) {
        let mut entries = self.entries.write();
        entries.retain(|(i, _)| *i != id);
        entries.push((id, close));
    }

    /// The overlay's place in the stack, the first opened at 0; `None` while
    /// it is not open. Reactive: a read subscribes to the stack.
    fn place(&self, id: u64) -> Option<usize> {
        self.entries.read().iter().position(|(i, _)| *i == id)
    }

    /// Removes an overlay's entry, on close or unmount. No-op if absent.
    fn remove(&mut self, id: u64) {
        self.entries.write().retain(|(i, _)| *i != id);
    }

    /// Closes the most recently opened overlay if there is one, popping it
    /// and invoking its close action. Returns `true` when an overlay was
    /// closed, so the back bridge skips the router.
    pub fn close_top(&mut self) -> bool {
        let popped = self.entries.write().pop();
        if let Some((_, close)) = popped {
            close.call(());
            true
        } else {
            false
        }
    }
}

/// Keeps `close` on the stack while `is_open` is true. For an overlay that
/// closes through a callback rather than a settable signal, such as a dialog
/// whose open state its consumer drives. With no stack provided above it, as
/// on a site with no OS back to bridge, it does nothing.
///
/// Returns the overlay's backdrop z-index ([`overlay_z`] of its place) while
/// it is open on a stack, `None` otherwise, for the overlay to draw at.
pub fn use_overlay_back_action(
    is_open: ReadSignal<bool>,
    close: Callback<()>,
) -> Memo<Option<u32>> {
    let stack = try_use_context::<OverlayBackStack>();
    let id = use_hook(next_overlay_id);

    use_effect(move || {
        let Some(mut stack) = stack else {
            return;
        };
        if is_open() {
            stack.push(id, close);
        } else {
            stack.remove(id);
        }
    });

    use_drop(move || {
        if let Some(mut stack) = stack {
            stack.remove(id);
        }
    });

    use_memo(move || stack.and_then(|stack| stack.place(id)).map(overlay_z))
}

/// Registers a signal-toggled overlay so the back intent closes it by setting
/// `open` false instead of navigating the router. Returns its z-index as
/// [`use_overlay_back_action`] does.
#[expect(
    clippy::must_use_candidate,
    reason = "most overlays only register; the z-index is for those that draw at it"
)]
pub fn use_overlay_back(open: Signal<bool>) -> Memo<Option<u32>> {
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    use_overlay_back_action(open.into(), close)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use dioxus::dioxus_core::NoOpMutations;

    use super::*;

    /// Two overlays' open signals and z-indexes, read back from the app.
    #[derive(Clone, Copy)]
    struct Probe {
        a: Signal<bool>,
        b: Signal<bool>,
        a_z: Memo<Option<u32>>,
        b_z: Memo<Option<u32>>,
        stack: OverlayBackStack,
    }

    thread_local! {
        static PROBE: Cell<Option<Probe>> = const { Cell::new(None) };
    }

    fn probe() -> Probe {
        PROBE.with(Cell::get).unwrap()
    }

    fn app() -> Element {
        let stack = use_overlay_back_stack();
        use_context_provider(|| stack);
        rsx! { Overlays {} }
    }

    #[component]
    fn Overlays() -> Element {
        let a = use_signal(|| false);
        let b = use_signal(|| false);
        let a_z = use_overlay_back(a);
        let b_z = use_overlay_back(b);
        let stack = use_context::<OverlayBackStack>();
        PROBE.with(|p| {
            p.set(Some(Probe {
                a,
                b,
                a_z,
                b_z,
                stack,
            }));
        });
        rsx! {}
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..8 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
    }

    fn z(memo: Memo<Option<u32>>) -> Option<u32> {
        *memo.peek()
    }

    #[test]
    fn each_place_takes_two_layers_from_the_tier_base() {
        assert_eq!(overlay_z(0), OVERLAY_Z);
        assert_eq!(overlay_z(1), OVERLAY_Z + 2);
        assert_eq!(overlay_z(usize::MAX), u32::MAX);
    }

    #[test]
    fn the_overlay_opened_last_sits_on_top_and_closes_first() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let p = probe();
        dom.in_runtime(|| {
            assert_eq!((z(p.a_z), z(p.b_z)), (None, None), "nothing open");
            let mut a = p.a;
            a.set(true);
        });
        settle(&mut dom);
        dom.in_runtime(|| {
            let mut b = p.b;
            b.set(true);
        });
        settle(&mut dom);
        dom.in_runtime(|| {
            assert_eq!(
                (z(p.a_z), z(p.b_z)),
                (Some(OVERLAY_Z), Some(OVERLAY_Z + 2)),
                "b over a"
            );
            let mut stack = p.stack;
            assert!(stack.close_top(), "back closes the top");
        });
        settle(&mut dom);
        dom.in_runtime(|| {
            assert!(!*p.b.peek(), "the top one, b, closed");
            assert_eq!(
                (z(p.a_z), z(p.b_z)),
                (Some(OVERLAY_Z), None),
                "a stays where it was"
            );
            let mut b = p.b;
            b.set(true);
            let mut a = p.a;
            a.set(false);
        });
        settle(&mut dom);
        dom.in_runtime(|| {
            assert_eq!(z(p.b_z), Some(OVERLAY_Z), "b alone takes the first layer");
            let mut a = p.a;
            a.set(true);
        });
        settle(&mut dom);
        dom.in_runtime(|| {
            assert_eq!(
                (z(p.a_z), z(p.b_z)),
                (Some(OVERLAY_Z + 2), Some(OVERLAY_Z)),
                "a reopened goes over b"
            );
        });
    }
}
