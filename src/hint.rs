//! The pieces of a hint: the small "?" that opens one, styled by
//! `assets/components.css` since the sites use it too, and the lines inside
//! it, styled by `assets/app.css`.
//!
//! A hint is a [`HintDialog`] whose body is these lines; what each hint says,
//! and when it opens, stays the app's.

use std::fmt;

use dioxus::prelude::*;

use crate::{AlertDialog, DialogAction};

/// A hint's dialog: the title, the body in a `hint-body` block, and a single
/// dismiss button, "Got it" unless `dismiss` names another. `actions` go
/// before the dismiss button, so a call to action sits in the footer rather
/// than in the body.
///
/// Compose the body from [`HintLine`], [`HintBullets`], [`HintKey`] and
/// [`HintChip`].
#[component]
pub fn HintDialog(
    open: Signal<bool>,
    title: String,
    children: Element,
    #[props(default)] actions: Vec<DialogAction>,
    #[props(default = "Got it".to_string())] dismiss: String,
) -> Element {
    rsx! {
        AlertDialog { open, title, actions: hint_actions(actions, dismiss),
            div { class: "hint-body", {children} }
        }
    }
}

/// The host's actions, then the dismiss button.
fn hint_actions(mut actions: Vec<DialogAction>, dismiss: String) -> Vec<DialogAction> {
    actions.push(DialogAction::Dismiss(dismiss));
    actions
}

/// The slot a screen offers its hint through, for an app whose header is
/// drawn above the router and so cannot reach into the screen. Provide it at
/// the app root with [`use_screen_hint_slot`], hand [`ScreenHint::opener`] to
/// [`PageHeader`](crate::PageHeader)'s `on_hint`, and call
/// [`use_screen_hint`] in each screen that has a hint.
#[derive(Clone, Copy, PartialEq)]
pub struct ScreenHint(Signal<Option<Callback<()>>>);

impl fmt::Debug for ScreenHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ScreenHint").finish_non_exhaustive()
    }
}

impl ScreenHint {
    /// What opens the current screen's hint, or `None` when the screen has
    /// none. Reading it subscribes the caller, so the header redraws as
    /// screens change.
    pub fn opener(&self) -> Option<Callback<()>> {
        (self.0)()
    }
}

/// Creates the [`ScreenHint`] slot and provides it to everything below.
/// Call once at the app root.
pub fn use_screen_hint_slot() -> ScreenHint {
    let slot = ScreenHint(use_signal(|| None));
    use_context_provider(|| slot)
}

/// Offers this screen's hint, opened by setting `open`, through the
/// [`ScreenHint`] slot for as long as the screen is mounted.
pub fn use_screen_hint(open: Signal<bool>) {
    let ScreenHint(slot) = use_context::<ScreenHint>();
    let opener = use_callback(move |()| {
        let mut open = open;
        open.set(true);
    });
    use_drop(move || {
        let mut slot = slot;
        slot.set(None);
    });
    use_effect(move || {
        let mut slot = slot;
        slot.set(Some(opener));
    });
}

/// The small "?" that sits beside a label and opens a hint. Stops the click
/// there, so a "?" inside a tappable row or accordion header does not also
/// toggle it.
#[component]
pub fn InfoButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "info-button",
            r#type: "button",
            aria_label: "What is this?",
            onclick: move |evt| {
                evt.stop_propagation();
                onclick.call(evt);
            },
            "?"
        }
    }
}

/// One body line of a hint.
#[component]
pub fn HintLine(children: Element) -> Element {
    rsx! {
        p { class: "hint-line", {children} }
    }
}

/// A bulleted list of hint lines.
#[component]
pub fn HintBullets(children: Element) -> Element {
    rsx! {
        ul { class: "hint-bullets", {children} }
    }
}

/// One bullet within [`HintBullets`].
#[component]
pub fn HintBullet(children: Element) -> Element {
    rsx! {
        li { {children} }
    }
}

/// An inert reference to an on-screen button, styled like one so the reader
/// recognizes what to press, and deliberately not tappable since the hint
/// points at the real button rather than replacing it. `color` is a CSS
/// variable name.
#[component]
pub fn HintKey(
    #[props(default = "--palette-2".to_string())] color: String,
    children: Element,
) -> Element {
    rsx! {
        span {
            class: "hint-key",
            style: "border-color: var({color}); color: var({color});",
            {children}
        }
    }
}

/// A real chip dropped into hint text as its own example. It is a
/// `stat-chip`, so pass the modifier the screen uses (`stat-chip-error`,
/// `stat-chip-p2`) in `class` and the example cannot drift from the real one.
#[component]
pub fn HintChip(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        span { class: "stat-chip hint-chip {class}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use dioxus::dioxus_core::NoOpMutations;

    use super::*;
    use crate::{DialogHostView, use_dialog_host, use_overlay_back_stack};

    #[test]
    fn hint_actions_end_with_the_dismiss() {
        let actions = hint_actions(
            vec![DialogAction::Dismiss("Dictionary".into())],
            "Got it".into(),
        );
        let labels: Vec<_> = actions
            .iter()
            .map(|a| match a {
                DialogAction::Dismiss(label) => label.as_str(),
                _ => "",
            })
            .collect();
        assert_eq!(labels, ["Dictionary", "Got it"]);
    }

    #[test]
    fn hint_dialog_draws_its_body_and_got_it() {
        fn app() -> Element {
            let stack = use_overlay_back_stack();
            use_context_provider(|| stack);
            let host = use_dialog_host();
            use_context_provider(|| host);
            let open = use_signal(|| true);
            rsx! {
                HintDialog { open, title: "Goal",
                    HintLine { "How many a day" }
                    HintChip { class: "stat-chip-error", "40 to go" }
                }
                DialogHostView {}
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"<div class="hint-body"><p class="hint-line">How many a day</p>"#));
        assert!(
            html.contains(r#"<span class="stat-chip hint-chip stat-chip-error">40 to go</span>"#)
        );
        assert!(html.contains(">Got it</button>"));
    }

    thread_local! {
        static SLOT: Cell<Option<ScreenHint>> = const { Cell::new(None) };
        static SHOWN: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
        static OPEN: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..8 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
    }

    #[test]
    fn a_screen_offers_its_hint_while_mounted() {
        fn app() -> Element {
            let slot = use_screen_hint_slot();
            let shown = use_signal(|| true);
            SLOT.with(|s| s.set(Some(slot)));
            SHOWN.with(|s| s.set(Some(shown)));
            rsx! {
                if shown() {
                    Screen {}
                }
            }
        }

        #[component]
        fn Screen() -> Element {
            let open = use_signal(|| false);
            OPEN.with(|o| o.set(Some(open)));
            use_screen_hint(open);
            rsx! {}
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let slot = SLOT.with(Cell::get).unwrap();
        let open = OPEN.with(Cell::get).unwrap();
        dom.in_runtime(|| {
            let opener = slot.0.peek().unwrap();
            opener.call(());
            assert!(*open.peek());
        });

        let mut shown = SHOWN.with(Cell::get).unwrap();
        dom.in_runtime(|| shown.set(false));
        settle(&mut dom);
        dom.in_runtime(|| assert!(slot.0.peek().is_none()));
    }
}
