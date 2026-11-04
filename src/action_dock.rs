//! The action dock: a screen's or a sheet's icon actions in one floating
//! squircle bar, on the `.action-dock` rules in [`COMPONENTS_CSS`].
//!
//! [`ActionDock`] is the bar and [`DockAction`] one button in it, an icon
//! from [`icons`] named for assistive tech by its label. [`DockFilter`] is
//! the filter button every list screen shares: the funnel, a dot while a
//! filter is on and, when the host gives it one, a small reset on its corner.
//! A host writes the bar's other children (a divider, a toggle with
//! `aria-pressed`) as markup on the same classes.
//!
//! [`COMPONENTS_CSS`]: crate::COMPONENTS_CSS
//! [`icons`]: crate::icons

use dioxus::prelude::*;

use crate::icons::{DockIcon, FUNNEL};

/// Where a dock floats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DockPlace {
    /// A screen's bottom right corner.
    #[default]
    Right,
    /// A bottom sheet's bottom right corner, over the sheet's content.
    Sheet,
    /// Centered along the bottom edge.
    Center,
}

impl DockPlace {
    /// The bar's classes for this place.
    #[must_use]
    pub fn class(self) -> &'static str {
        match self {
            Self::Right => "action-dock action-dock-right",
            Self::Sheet => "action-dock action-dock-right sheet-dock",
            Self::Center => "action-dock",
        }
    }
}

/// The floating bar of a screen's or a sheet's icon actions. `class` adds
/// the host's own classes, such as an entrance.
#[component]
pub fn ActionDock(
    #[props(default)] place: DockPlace,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "{place.class()} {class}", {children} }
    }
}

/// One action in the dock: `icon`, one of the [`icons`] paths, in a
/// squircle, `label` as its accessible name, grayed and inert while
/// `disabled`. `class` adds the host's own classes to the button.
///
/// [`icons`]: crate::icons
#[component]
pub fn DockAction(
    label: &'static str,
    icon: &'static str,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            class: "action-dock-btn {class}",
            "aria-label": label,
            disabled,
            onclick: move |_| onclick.call(()),
            DockIcon { path: icon }
        }
    }
}

/// The dock's filter button: the funnel, its dot while a filter or a sort is
/// on, and, when `on_reset` is given and `active`, a small reset on its
/// corner named `reset_label`.
#[component]
pub fn DockFilter(
    label: &'static str,
    dot: bool,
    #[props(default)] active: bool,
    onclick: EventHandler<()>,
    #[props(default)] on_reset: Option<EventHandler<()>>,
    #[props(default)] reset_label: &'static str,
) -> Element {
    rsx! {
        div { class: "action-dock-group",
            button {
                class: "action-dock-btn",
                "aria-label": label,
                onclick: move |_| onclick.call(()),
                DockIcon { path: FUNNEL }
                if dot {
                    span { class: "filter-dot" }
                }
            }
            if let (true, Some(on_reset)) = (active, on_reset) {
                button {
                    class: "action-dock-reset",
                    "aria-label": reset_label,
                    onclick: move |_| on_reset.call(()),
                    "\u{d7}"
                }
            }
        }
    }
}
