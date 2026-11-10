//! Collapsible sections in a list: each section's header opens and shuts
//! its body, and one icon button collapses or expands them all.
//!
//! [`use_collapse_set`] holds which sections are shut, keyed by a string the
//! host picks (a version, an id); everything starts open. Each section draws
//! its header as a [`CollapseHeader`], which a click anywhere on toggles, or
//! puts a lone [`CollapseCaret`] in a header of its own, and wraps its body
//! in [`CollapseBody`]; the list draws one [`CollapseAll`] over the keys it
//! shows. A shut body eases closed, and is inert so Tab and screen readers
//! pass over it.

use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::icons::{DockIcon, FOLD, UNFOLD};

/// The keys of the shut sections. Everything not in it is open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shut(BTreeSet<String>);

impl Shut {
    /// Whether the section `key` is open.
    #[must_use]
    pub fn is_open(&self, key: &str) -> bool {
        !self.0.contains(key)
    }

    /// Opens a shut section, or shuts an open one.
    pub fn toggle(&mut self, key: &str) {
        if !self.0.remove(key) {
            self.0.insert(key.to_owned());
        }
    }

    /// Whether every one of `keys` is shut. False for no keys, so an empty
    /// list offers Collapse all.
    #[must_use]
    pub fn all_shut<'a>(&self, keys: impl IntoIterator<Item = &'a str>) -> bool {
        let mut any = false;
        for key in keys {
            if self.is_open(key) {
                return false;
            }
            any = true;
        }
        any
    }

    /// Shuts every one of `keys`.
    pub fn collapse_all<'a>(&mut self, keys: impl IntoIterator<Item = &'a str>) {
        self.0.extend(keys.into_iter().map(str::to_owned));
    }

    /// Opens every section.
    pub fn expand_all(&mut self) {
        self.0.clear();
    }
}

/// A list's open and shut sections, in a `Copy` handle. From
/// [`use_collapse_set`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollapseSet {
    shut: Signal<Shut>,
}

impl CollapseSet {
    /// Whether the section `key` is open. Reading subscribes the caller.
    #[must_use]
    pub fn is_open(&self, key: &str) -> bool {
        self.shut.read().is_open(key)
    }

    /// Opens a shut section, or shuts an open one.
    pub fn toggle(&mut self, key: &str) {
        self.shut.write().toggle(key);
    }

    /// Whether every one of `keys` is shut. Reading subscribes the caller.
    #[must_use]
    pub fn all_shut<'a>(&self, keys: impl IntoIterator<Item = &'a str>) -> bool {
        self.shut.read().all_shut(keys)
    }

    /// Shuts every one of `keys`.
    pub fn collapse_all<'a>(&mut self, keys: impl IntoIterator<Item = &'a str>) {
        self.shut.write().collapse_all(keys);
    }

    /// Opens every section.
    pub fn expand_all(&mut self) {
        self.shut.write().expand_all();
    }
}

/// A list's collapse state, every section open.
pub fn use_collapse_set() -> CollapseSet {
    CollapseSet {
        shut: use_signal(Shut::default),
    }
}

/// The caret on a section's header: points right while shut and down while
/// open, and toggles `section` in `set` on a tap. `label` names the section for a
/// screen reader, as in "1.18.0".
#[component]
pub fn CollapseCaret(set: CollapseSet, section: String, label: String) -> Element {
    let open = set.is_open(&section);
    let action = if open { "Collapse" } else { "Expand" };
    rsx! {
        button {
            r#type: "button",
            class: "collapse-caret",
            "aria-expanded": "{open}",
            "aria-label": "{action} {label}",
            onclick: move |_| {
                let mut set = set;
                set.toggle(&section);
            },
            span { class: "collapse-caret-glyph", "aria-hidden": "true", "\u{25b8}" }
        }
    }
}

/// A section's header row that toggles `section` in `set` from a click
/// anywhere on it: the caret, then `children` (a heading, a date, tags).
/// The caret is the row's one Tab stop and carries `aria-expanded`; Enter or
/// Space on it clicks it, and the click reaches the row. `label` names the
/// section for a screen reader, as in "1.18.0", and `class` adds the host's
/// classes after `collapse-header`; adding `collapse-header-quiet` drops the
/// hover fill, for a header that heads a card which has its own hover. A link
/// in `children` stops its click's propagation if following it should not
/// toggle the section.
#[component]
pub fn CollapseHeader(
    set: CollapseSet,
    section: String,
    label: String,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    let open = set.is_open(&section);
    let action = if open { "Collapse" } else { "Expand" };
    let class = match class {
        Some(extra) => format!("collapse-header {extra}"),
        None => "collapse-header".to_owned(),
    };
    rsx! {
        div {
            class,
            onclick: move |_| {
                let mut set = set;
                set.toggle(&section);
            },
            button {
                r#type: "button",
                class: "collapse-caret",
                "aria-expanded": "{open}",
                "aria-label": "{action} {label}",
                span { class: "collapse-caret-glyph", "aria-hidden": "true", "\u{25b8}" }
            }
            {children}
        }
    }
}

/// A section's body, eased shut while `section` is shut in `set`, and inert
/// while it is.
#[component]
pub fn CollapseBody(set: CollapseSet, section: String, children: Element) -> Element {
    let open = set.is_open(&section);
    rsx! {
        div {
            class: if open { "collapse-body open" } else { "collapse-body" },
            inert: (!open).then_some(true),
            div { class: "collapse-body-inner", {children} }
        }
    }
}

/// One icon button for the whole list: two arrows pointing in ("Collapse
/// all") while any of `keys` is open, two pointing out ("Expand all") once
/// every one is shut. The words are its accessible name.
#[component]
pub fn CollapseAll(set: CollapseSet, keys: Vec<String>) -> Element {
    let all_shut = set.all_shut(keys.iter().map(String::as_str));
    rsx! {
        button {
            r#type: "button",
            class: "btn-action btn-icon collapse-all",
            "aria-label": if all_shut { "Expand all" } else { "Collapse all" },
            onclick: move |_| {
                let mut set = set;
                if all_shut {
                    set.expand_all();
                } else {
                    set.collapse_all(keys.iter().map(String::as_str));
                }
            },
            DockIcon { path: if all_shut { UNFOLD } else { FOLD } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_starts_open_and_a_toggle_flips_one() {
        let mut shut = Shut::default();
        assert!(shut.is_open("1.0.0"));
        shut.toggle("1.0.0");
        assert!(!shut.is_open("1.0.0"));
        assert!(shut.is_open("1.1.0"), "only the toggled one shuts");
        shut.toggle("1.0.0");
        assert!(shut.is_open("1.0.0"));
    }

    #[test]
    fn collapse_all_shuts_every_key_and_any_reopens() {
        let keys = ["1.1.0", "1.0.0"];
        let mut shut = Shut::default();
        assert!(!shut.all_shut(keys));
        shut.collapse_all(keys);
        assert!(shut.all_shut(keys));
        shut.toggle("1.0.0");
        assert!(shut.is_open("1.0.0"));
        assert!(!shut.all_shut(keys), "one open again offers Collapse all");
        shut.expand_all();
        assert!(keys.iter().all(|key| shut.is_open(key)));
    }

    #[test]
    fn an_empty_list_is_never_all_shut() {
        assert!(!Shut::default().all_shut([]));
    }
}
