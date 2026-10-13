//! A centered dialog for the apps: a title, a body and a row of actions over
//! a dim. Styled by `assets/app.css`.
//!
//! A `position: fixed` overlay inside a screen's scroll container gets
//! trapped by it in a mobile WebView, and the dim stops at the content
//! column. So dialogs draw at the app root: the host provides a
//! [`DialogHost`] from [`use_dialog_host`] and mounts [`DialogHostView`]
//! beside its router, and a dialog anywhere below describes itself into the
//! host with [`AlertDialog`] or [`use_hosted_dialog`].
//!
//! Closing fades the dim and the dialog out before they leave the DOM. Every
//! label is the host's: the kit draws the buttons it is given.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::use_overlay_back;

/// One button in a dialog's action row. Every action closes the dialog.
#[derive(Clone, PartialEq)]
pub enum DialogAction {
    /// Closes the dialog and does nothing else: Cancel, Got it.
    Dismiss(String),
    /// Closes the dialog, then runs `on_confirm`. `danger` draws it in the
    /// error color, for an action that cannot be undone.
    Confirm {
        /// The button's text.
        label: String,
        /// Marks the action as destructive.
        danger: bool,
        /// Runs after the dialog starts closing.
        on_confirm: Callback<()>,
    },
    /// A link styled as a button, opened in a new tab (an app's WebView
    /// hands a `_blank` https link to the system browser), and the dialog
    /// closes.
    Link {
        /// The link's text.
        label: String,
        /// Where it goes.
        href: String,
    },
}

/// What a dialog draws: its title, its body, and its actions left to right.
#[derive(Clone, PartialEq)]
pub struct DialogSpec {
    /// The heading.
    pub title: String,
    /// The body, inside the scrolling `.alert-dialog-description`.
    pub body: Element,
    /// The action row.
    pub actions: Vec<DialogAction>,
}

/// The dialog on screen, with the owner that put it there.
#[derive(Clone)]
struct Shown {
    spec: DialogSpec,
    owner: u64,
    /// The owner's close: sets its open signal false.
    close: Callback<()>,
    /// Fading out; inert until the animation ends and the slot clears.
    leaving: bool,
}

/// The app root's dialog slot. A `Copy` handle; create it with
/// [`use_dialog_host`], provide it with `use_context_provider` above the
/// router, and mount [`DialogHostView`] beside the router.
#[derive(Clone, Copy)]
pub struct DialogHost {
    slot: Signal<Option<Shown>>,
}

/// Creates the dialog slot.
pub fn use_dialog_host() -> DialogHost {
    DialogHost {
        slot: use_signal(|| None),
    }
}

impl DialogHost {
    /// Starts the fade out of `owner`'s dialog, if it is the one showing and
    /// is not already leaving. Returns whether it started.
    fn leave(self, owner: u64) -> bool {
        let mut slot = self.slot;
        let showing = slot
            .peek()
            .as_ref()
            .is_some_and(|s| s.owner == owner && !s.leaving);
        if showing && let Some(shown) = slot.write().as_mut() {
            shown.leaving = true;
        }
        showing
    }
}

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// Keeps the host's slot in step with `open`: when it turns true the dialog
/// `build` returns is shown, when it turns false the dialog fades out, and
/// another owner's dialog is never touched. Registers `open` with the
/// [`OverlayBackStack`](crate::OverlayBackStack), so the OS back gesture
/// closes the dialog first.
///
/// The spec is built when `open` turns true. A body that changes while it is
/// showing does so through its own components and signals.
pub fn use_hosted_dialog(open: Signal<bool>, build: impl Fn() -> DialogSpec + 'static) {
    let host = use_context::<DialogHost>();
    let owner = use_hook(|| NEXT_OWNER.fetch_add(1, Ordering::Relaxed));
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    use_overlay_back(open);
    use_drop(move || {
        host.leave(owner);
    });
    use_effect(move || {
        if open() {
            let mut slot = host.slot;
            slot.set(Some(Shown {
                spec: build(),
                owner,
                close,
                leaving: false,
            }));
        } else {
            host.leave(owner);
        }
    });
}

/// A dialog drawn by the app root's [`DialogHostView`]. `open` is the
/// caller's, so it keeps the state that drives the dialog; the dialog sets it
/// false when any action, the dim or the OS back gesture closes it.
/// `children` is the body. Renders nothing in place.
#[component]
pub fn AlertDialog(
    open: Signal<bool>,
    title: String,
    actions: Vec<DialogAction>,
    children: Element,
) -> Element {
    let spec = DialogSpec {
        title,
        body: children,
        actions,
    };
    let mut latest = use_hook(|| CopyValue::new(spec.clone()));
    latest.set(spec);
    use_hosted_dialog(open, move || latest.cloned());
    rsx! {}
}

/// Draws the dialog in the [`DialogHost`], if any, above everything.
#[component]
pub fn DialogHostView() -> Element {
    let host = use_context::<DialogHost>();
    let Some(shown) = host.slot.read().clone() else {
        return rsx! {};
    };
    let Shown {
        spec,
        owner,
        close,
        leaving,
    } = shown;
    let state = if leaving { "closed" } else { "open" };
    let dismiss = move || {
        if host.leave(owner) {
            close.call(());
        }
    };
    rsx! {
        div {
            class: "alert-dialog-overlay",
            "data-state": state,
            onclick: move |_| dismiss(),
        }
        div {
            class: "alert-dialog",
            role: "dialog",
            aria_modal: "true",
            "data-state": state,
            onanimationend: move |evt| {
                if evt.animation_name() == "alert-dialog-out" {
                    let mut slot = host.slot;
                    if slot.peek().as_ref().is_some_and(|s| s.owner == owner && s.leaving) {
                        slot.set(None);
                    }
                }
            },
            h2 { class: "alert-dialog-title", "{spec.title}" }
            hr { class: "alert-dialog-rule" }
            div { class: "alert-dialog-description", {spec.body} }
            hr { class: "alert-dialog-rule" }
            div { class: "alert-dialog-actions",
                for action in spec.actions {
                    match action {
                        DialogAction::Dismiss(label) => rsx! {
                            button {
                                class: "alert-dialog-cancel",
                                r#type: "button",
                                onclick: move |_| dismiss(),
                                "{label}"
                            }
                        },
                        DialogAction::Confirm { label, danger, on_confirm } => rsx! {
                            button {
                                class: if danger { "alert-dialog-action-danger" } else { "alert-dialog-action" },
                                r#type: "button",
                                onclick: move |_| {
                                    if host.leave(owner) {
                                        close.call(());
                                        on_confirm.call(());
                                    }
                                },
                                "{label}"
                            }
                        },
                        DialogAction::Link { label, href } => rsx! {
                            a {
                                class: "alert-dialog-cancel",
                                href,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                onclick: move |_| dismiss(),
                                "{label}"
                            }
                        },
                    }
                }
            }
        }
    }
}
