//! The app's place in its lifecycle: in the foreground, or in the background.
//!
//! An app's page keeps its state while the app sits in the background, and
//! nothing tells a screen it came back, so a count, a day or a cache it holds
//! stays as it was when the app left. [`use_app_lifecycle_root`] keeps the
//! phase in one [`AppLifecycle`] for the whole app, and a screen reacts to a
//! return with [`use_on_resume`].
//!
//! The page reports the phase itself through `visibilitychange`. The native
//! shell knows sooner and more surely, so a host on `dioxus/mobile` also
//! forwards the window events from `use_wry_event_handler`, which only it can
//! reach:
//!
//! ```ignore
//! let life = orangui::use_app_lifecycle_root();
//! dioxus::mobile::use_wry_event_handler(move |event, _| {
//!     if let Event::WindowEvent { event, .. } = event {
//!         match event {
//!             WindowEvent::Resumed | WindowEvent::Focused(true) => life.resumed(),
//!             WindowEvent::Suspended => life.suspended(),
//!             _ => {}
//!         }
//!     }
//! });
//! ```
//!
//! A return counts once however many of those report it: [`AppLifecycle::resumed`]
//! acts only on an app that was in the background.

use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;

/// Reports each change of the page's visibility: `true` on visible, `false`
/// on hidden. One listener, replaced if the root mounts again.
const VISIBILITY_JS: &str = "\
if (window.__oranguiVisibility) document.removeEventListener('visibilitychange', window.__oranguiVisibility);\
window.__oranguiVisibility = () => dioxus.send(document.visibilityState === 'visible');\
document.addEventListener('visibilitychange', window.__oranguiVisibility);";

/// Where the app is: on screen, or in the background.
///
/// The set is closed, so a match need not carry a `_` arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppPhase {
    /// On screen and taking input.
    Foreground,
    /// In the background, or behind the lock screen.
    Background,
}

/// The app's lifecycle, shared from the root by [`use_app_lifecycle_root`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppLifecycle {
    phase: Signal<AppPhase>,
    resumes: Signal<u32>,
}

impl AppLifecycle {
    /// The phase now. Reading it subscribes the caller.
    #[must_use]
    pub fn phase(&self) -> AppPhase {
        *self.phase.read()
    }

    /// How many times the app has come back to the foreground. Reading it
    /// subscribes the caller, so an effect that reads it runs again on each
    /// return.
    #[must_use]
    pub fn resumes(&self) -> u32 {
        *self.resumes.read()
    }

    /// The app is back in the foreground. Counts a return only when the app
    /// was in the background.
    pub fn resumed(mut self) {
        if *self.phase.peek() == AppPhase::Background {
            self.phase.set(AppPhase::Foreground);
            let next = self.resumes.peek().wrapping_add(1);
            self.resumes.set(next);
        }
    }

    /// The app went into the background.
    pub fn suspended(mut self) {
        if *self.phase.peek() == AppPhase::Foreground {
            self.phase.set(AppPhase::Background);
        }
    }
}

/// Keeps the app's lifecycle and shares it with every screen. Call it once,
/// at the app's root, and forward the native shell's events into the handle
/// it returns as the module docs show.
#[must_use = "forward the native shell's window events into it"]
pub fn use_app_lifecycle_root() -> AppLifecycle {
    let life = use_context_provider(|| AppLifecycle {
        phase: Signal::new(AppPhase::Foreground),
        resumes: Signal::new(0),
    });
    use_future(move || async move {
        let mut eval = document::eval(VISIBILITY_JS);
        while let Ok(visible) = eval.recv::<bool>().await {
            if visible {
                life.resumed();
            } else {
                life.suspended();
            }
        }
    });
    life
}

/// The app's lifecycle, from [`use_app_lifecycle_root`] above, or `None`
/// under a tree with none.
#[must_use]
pub fn use_app_lifecycle() -> Option<AppLifecycle> {
    try_use_context()
}

/// Runs `on_resume` each time the app comes back to the foreground, never at
/// mount. Does nothing under a tree with no [`use_app_lifecycle_root`].
pub fn use_on_resume(mut on_resume: impl FnMut() + 'static) {
    let life = use_hook(try_consume_context::<AppLifecycle>);
    let seen = use_hook(|| Rc::new(Cell::new(life.map_or(0, |l| *l.resumes.peek()))));
    use_effect(move || {
        let Some(life) = life else { return };
        let resumes = life.resumes();
        if resumes != seen.get() {
            seen.set(resumes);
            on_resume();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_no_root_there_is_no_lifecycle() {
        let mut dom = VirtualDom::new(|| {
            assert_eq!(use_app_lifecycle(), None);
            rsx! {}
        });
        dom.rebuild_in_place();
    }

    #[test]
    fn a_return_counts_once_and_only_from_the_background() {
        let mut dom = VirtualDom::new(|| {
            let life = use_app_lifecycle_root();
            use_hook(move || {
                life.resumed();
                assert_eq!(
                    *life.resumes.peek(),
                    0,
                    "a foreground app does not count a return"
                );
                life.suspended();
                life.resumed();
                life.resumed();
                assert_eq!(
                    *life.resumes.peek(),
                    1,
                    "two reports of one return count once"
                );
                assert_eq!(*life.phase.peek(), AppPhase::Foreground);
            });
            rsx! {}
        });
        dom.rebuild_in_place();
    }
}
