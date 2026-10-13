//! The pieces of a hint: the small "?" that opens one, styled by
//! `assets/components.css` since the sites use it too, and the lines inside
//! it, styled by `assets/app.css`.
//!
//! A hint is an [`AlertDialog`](crate::AlertDialog) whose body is these
//! lines; what each hint says stays the app's.

use dioxus::prelude::*;

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
