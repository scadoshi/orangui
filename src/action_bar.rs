//! Footer action bar.
//!
//! Wraps the row of buttons pinned at the bottom of a screen (Back / Save /
//! Cancel and friends). Renders `div.action-bar`; the shared styling lives in
//! `assets/components.css`.

use dioxus::prelude::*;

/// A horizontal bar of actions, typically pinned to the bottom of a screen.
///
/// `class` appends extra classes (e.g. an entry animation) alongside the base
/// `action-bar`.
#[component]
pub fn ActionBar(#[props(default)] class: Option<String>, children: Element) -> Element {
    let full = match &class {
        Some(extra) => format!("action-bar {extra}"),
        None => "action-bar".to_string(),
    };
    rsx! {
        div { class: "{full}", {children} }
    }
}
