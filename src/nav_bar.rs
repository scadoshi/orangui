//! Sticky site nav shell: brand on the left, a link panel on the right that
//! collapses behind a hamburger toggle below the 60rem breakpoint.
//!
//! The shell owns the structure (wrapper, toggle, collapsing panel) and its
//! CSS; the host owns the content via slots: its brand link, its `li` link
//! items, and an optional trailing panel item (typically [`ThemePicker`]).
//! The host also owns the `open` signal so link `onclick`s can close the
//! panel after navigating. A tap outside the nav closes it too.
//!
//! The outside tap lands two ways. The open panel renders a transparent
//! `div.nav-backdrop` (`position: fixed`) before `.nav-wrapper`, which eats
//! the tap so the page under it doesn't take it as well. A host that
//! transforms an ancestor of the nav (or gives it a filter, `contain` or
//! `will-change: transform`) makes that ancestor the backdrop's containing
//! block, so the backdrop covers the ancestor rather than the viewport. A
//! document listener covers the rest: a tap that ends outside every
//! `.nav-wrapper` and off the backdrop closes the panel too, whatever the
//! backdrop covers. The kit itself never transforms the wrapper or its
//! ancestors.
//!
//! The folded panel carries its own still [`GridCells`] copy, the same
//! backdrop the app screens and the bottom sheet draw, under the links and
//! the trailing slot. At bar widths it hides.
//!
//! [`ThemePicker`]: crate::ThemePicker

use dioxus::prelude::*;

use crate::grid_cells::GridCells;

/// JS run on brand-click by convention: smooth-scroll to top and restart the
/// `.logo` entrance animation. Exported so every surface's brand link shares
/// one copy (`document::eval(BRAND_RESET_JS)`).
pub const BRAND_RESET_JS: &str = r"
    window.scrollTo({ top: 0, behavior: 'smooth' });
    const el = document.querySelector('.logo');
    if (el) {
        el.style.animation = 'none';
        void el.offsetHeight;
        el.style.animation = '';
    }
";

/// Sends once for each tap that ends outside every `.nav-wrapper` and off the
/// `.nav-backdrop` (which closes the panel itself) while a panel is open.
/// Capture phase, so a host handler that stops the event can't hide it, and
/// `pointerup` rather than `click`, which iOS Safari only sends for elements
/// it thinks are clickable. A tap that finds no nav on the page removes the
/// listener.
const NAV_AWAY_JS: &str = r"
const away = (e) => {
    if (!document.querySelector('.nav-wrapper')) {
        document.removeEventListener('pointerup', away, true);
        return;
    }
    const target = e.target;
    if (!(target instanceof Element) || target.closest('.nav-wrapper, .nav-backdrop')) return;
    if (document.querySelector('.nav-panel-open')) dioxus.send(true);
};
document.addEventListener('pointerup', away, true);
";

/// The folded panel's open state, which [`NavBar`] provides as context so an
/// item inside the panel can fold the menu when it raises an overlay of its
/// own, as [`ThemePicker`] does for its phone-width sheet.
///
/// [`ThemePicker`]: crate::ThemePicker
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavPanelOpen(pub Signal<bool>);

/// Nav shell with a hamburger-collapsing link panel.
#[component]
pub fn NavBar(
    /// Panel open state, host-owned: pass a fresh signal and close it from
    /// your link `onclick`s.
    open: Signal<bool>,
    /// Brand element (the host's home link).
    brand: Element,
    /// Optional content pinned outside the collapsing panel, between the
    /// brand and the toggle, shown only at hamburger widths: typically the
    /// site's `a.nav-cta` pills, whose panel copies sit in `li.nav-cta-item`
    /// and hide there. Wrapped in `div.nav-persistent`, whose children glide.
    persistent: Option<Element>,
    /// Link items (`li` elements) rendered inside the panel's `ul.nav-links`.
    links: Element,
    /// Optional trailing panel item after the links (e.g. the theme picker),
    /// wrapped in `div.nav-trailing`, which at hamburger widths is the
    /// folded menu's solid footer band.
    trailing: Option<Element>,
) -> Element {
    let mut open = open;
    use_context_provider(|| NavPanelOpen(open));
    use_future(move || async move {
        let mut away = document::eval(NAV_AWAY_JS);
        while away.recv::<bool>().await.is_ok() {
            open.set(false);
        }
    });
    let panel_class = if open() {
        "nav-panel nav-panel-open"
    } else {
        "nav-panel"
    };
    let toggle_class = if open() {
        "nav-toggle nav-toggle-open"
    } else {
        "nav-toggle"
    };

    rsx! {
        // A tap anywhere outside the nav closes the open panel. It sits under
        // the nav, so the panel's own links and the toggle still take clicks.
        if open() {
            div { class: "nav-backdrop", onclick: move |_| open.set(false) }
        }
        div { class: "nav-wrapper",
            nav {
                {brand}
                if let Some(p) = persistent {
                    div { class: "nav-persistent", "data-nav-glide": "true", {p} }
                }
                button {
                    class: "{toggle_class}",
                    aria_label: "Toggle navigation menu",
                    aria_expanded: "{open()}",
                    onclick: move |_| {
                        let next = !open();
                        open.set(next);
                    },
                    span { class: "nav-toggle-bar" }
                    span { class: "nav-toggle-bar" }
                    span { class: "nav-toggle-bar" }
                }
                div { class: "{panel_class}",
                    // The same backdrop as the screen, pinned to the folded panel.
                    GridCells { still: true }
                    div { class: "nav-panel-inner",
                        ul { class: "nav-links scroll-fade-y", "data-nav-glide": "true", {links} }
                        if let Some(t) = trailing {
                            div { class: "nav-trailing", {t} }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_away_listener_skips_the_nav_and_its_backdrop() {
        assert!(NAV_AWAY_JS.contains(".nav-wrapper, .nav-backdrop"));
        assert!(NAV_AWAY_JS.contains(".nav-panel-open"));
        assert!(NAV_AWAY_JS.contains("removeEventListener"));
    }
}
