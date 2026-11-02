//! Shared content panel.
//!
//! The canonical card/dialog shape: an optional header (eyebrow + title +
//! status pill), a rule, the body, and an optional actions row under a second
//! rule: `header <hr> body <hr> actions`. Used for content cards on the site,
//! in the app, and on the portfolio so they all read as one system. `eyebrow`,
//! `title`, `status`, `title_bar` and `actions` are all optional; a rule is
//! only drawn where the adjacent section exists. Styling is in
//! `assets/components.css`.

use dioxus::prelude::*;

use crate::banner::BannerStatus;

/// A content panel: `header <hr> body <hr> actions`.
///
/// The body is `children`; the footer button row is the `actions` slot (each
/// consumer supplies its own links/buttons, since internal routing is app-specific).
/// Give action links/buttons the `panel-action` class for the shared pill look.
#[component]
pub fn Panel(
    /// Uppercase eyebrow label above the title (e.g. "One-Time").
    #[props(default)]
    eyebrow: Option<String>,
    /// Panel title.
    #[props(default)]
    title: Option<String>,
    /// Render the title as an `h1` instead of the default `h3`. For page heroes,
    /// where the panel's title IS the document's heading. A page whose only
    /// heading is an `h3` reads as a fragment to crawlers and screen readers.
    /// Cards and in-page panels leave this off.
    #[props(default = false)]
    title_h1: bool,
    /// Center the title and action row, as the apps' dialogs do; the body
    /// stays left-aligned to read. For a panel used as a dialog. A centered
    /// panel carries no eyebrow: centered, it reads as a second subtitle.
    #[props(default = false)]
    centered: bool,
    /// Rendered inside the heading after the title text, so it wraps with
    /// the title. For chips that describe the panel, such as a project's tags.
    #[props(default)]
    title_trailing: Option<Element>,
    /// Optional status pill shown beside the eyebrow.
    #[props(default)]
    status: Option<BannerStatus>,
    /// Overrides the status pill's default label.
    #[props(default)]
    status_label: Option<String>,
    /// A title bar the host builds, drawn in the head over its rule in place
    /// of the eyebrow row and the title, which it overrides. For a head that
    /// carries a control beside its heading, such as a "?" button.
    #[props(default)]
    title_bar: Option<Element>,
    /// Footer button row. Omit for a panel with no actions.
    #[props(default)]
    actions: Option<Element>,
    /// Extra classes on the card, after `panel-card`, for a host that styles
    /// one kind of panel its own way.
    #[props(default)]
    class: Option<String>,
    /// Body content.
    children: Element,
) -> Element {
    let has_header =
        title_bar.is_some() || eyebrow.is_some() || title.is_some() || status.is_some();
    let mut card_class = String::from(if centered {
        "panel-card panel-centered"
    } else {
        "panel-card"
    });
    if let Some(extra) = class {
        card_class.push(' ');
        card_class.push_str(&extra);
    }

    rsx! {
        // Marked for the sites' scroll reveal (REVEAL_JS); inert elsewhere.
        div { class: card_class, "data-reveal": "true",
            if let Some(bar) = title_bar {
                div { class: "panel-head", {bar} }
                hr { class: "panel-rule" }
            } else if has_header {
                div { class: "panel-head",
                    if eyebrow.is_some() || status.is_some() {
                        div { class: "panel-eyebrow-row",
                            if let Some(eyebrow) = eyebrow {
                                span { class: "panel-eyebrow", "{eyebrow}" }
                            }
                            if let Some(status) = status {
                                span {
                                    class: status.class(),
                                    {status_label.unwrap_or_else(|| status.default_label().to_string())}
                                }
                            }
                        }
                    }
                    if let Some(title) = title {
                        if title_h1 {
                            h1 { class: "panel-title", "{title}" {title_trailing} }
                        } else {
                            h3 { class: "panel-title", "{title}" {title_trailing} }
                        }
                    }
                }
                hr { class: "panel-rule" }
            }
            div { class: "panel-body", {children} }
            if let Some(actions) = actions {
                hr { class: "panel-rule" }
                div { class: "panel-actions", {actions} }
            }
        }
    }
}
