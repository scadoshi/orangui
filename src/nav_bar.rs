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
//! Overlays stack on the panel. A dialog, bottom sheet or toast an item in
//! the panel opens renders at the app root, outside the nav's DOM, so a tap
//! inside it (its close button, its own dim) would count as outside. The
//! listener ignores a tap that lands in any overlay layer, and while a
//! dialog or sheet is open it does nothing at all: the top layer closes on
//! its own dismissal and the panel under it stays. A sheet that wants the
//! panel folded does that itself through [`NavPanelOpen`], as the
//! [`ThemePicker`] does.
//!
//! [`ThemePicker`]: crate::ThemePicker

use dioxus::prelude::*;

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
///
/// A tap inside an overlay layer (`AlertDialog`, `BottomSheet`, the toast
/// stack) is not an outside tap, and no tap counts while a dialog or sheet is
/// open: that layer is the top of the stack and its own dismissal closes it,
/// leaving the panel under it in place.
///
/// One listener per document: the remover of the listener in place sits on
/// `window.__oranguiNavAway`, and each mount calls it before adding its own,
/// so a remounted `NavBar` replaces the listener rather than stacking one.
const NAV_AWAY_JS: &str = r#"
const layers = '.alert-dialog-overlay, .alert-dialog, .sheet-backdrop, .bottom-sheet, .toast-container';
const shown = '.alert-dialog[data-state="open"], .sheet-backdrop.show, .bottom-sheet.show';
const away = (e) => {
    if (!document.querySelector('.nav-wrapper')) {
        document.removeEventListener('pointerup', away, true);
        return;
    }
    if (!document.querySelector('.nav-panel-open')) return;
    if (document.querySelector(shown)) return;
    const target = e.target;
    if (!(target instanceof Element) || target.closest('.nav-wrapper, .nav-backdrop, ' + layers)) return;
    dioxus.send(true);
};
if (typeof window.__oranguiNavAway === 'function') window.__oranguiNavAway();
document.addEventListener('pointerup', away, true);
window.__oranguiNavAway = () => document.removeEventListener('pointerup', away, true);
"#;

/// The folded panel's open state, which [`NavBar`] provides as context so an
/// item inside the panel can fold the menu when it raises an overlay of its
/// own, as [`ThemePicker`] does for its phone-width sheet.
///
/// [`ThemePicker`]: crate::ThemePicker
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavPanelOpen(pub Signal<bool>);

/// Scrolls the open panel's link list back to its top, now and again once
/// the panel has finished growing.
const LIST_TOP_JS: &str = "const top = () => { const l = document.querySelector('.nav-panel-open .nav-links'); if (l) l.scrollTop = 0; }; top(); setTimeout(top, 300);";

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
    /// and hide there. A pill that is one SVG and no text is a
    /// [`NavCtaIcon`].
    /// Wrapped in `div.nav-persistent`, whose children glide.
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
    // The list opens at its top: a scroller that grows from nothing can
    // come up scrolled to its end, with the first row cut off.
    use_effect(move || {
        if open() {
            document::eval(LIST_TOP_JS);
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
                    r#type: "button",
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
                    div { class: "nav-panel-inner",
                        ul { class: "nav-links", "data-nav-glide": "true", {links} }
                        if let Some(t) = trailing {
                            div { class: "nav-trailing", {t} }
                        }
                    }
                }
            }
        }
    }
}

/// An icon-only `a.nav-cta` pill: one filled 24-unit `path` inside, `label`
/// as the link's accessible name and tooltip. Sits in [`NavBar`]'s
/// `persistent` slot and in a panel `li.nav-cta-item` alike; the host closes
/// the panel from `onclick`.
#[component]
pub fn NavCtaIcon(
    href: &'static str,
    label: &'static str,
    path: &'static str,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        a {
            class: "nav-cta nav-cta-icon",
            href,
            "aria-label": label,
            title: label,
            onclick: move |event| {
                if let Some(handler) = &onclick {
                    handler.call(event);
                }
            },
            svg {
                view_box: "0 0 24 24",
                "aria-hidden": "true",
                path { d: path }
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

    /// A text check on the script: it keeps one listener per document by
    /// calling the stored remover before adding its own and storing the new one.
    #[test]
    fn the_away_listener_replaces_the_one_before_it() {
        let remover = "window.__oranguiNavAway";
        let calls = NAV_AWAY_JS.find(&format!("{remover}()"));
        let adds = NAV_AWAY_JS.find("document.addEventListener('pointerup', away, true)");
        let stores = NAV_AWAY_JS.find(&format!(
            "{remover} = () => document.removeEventListener('pointerup', away, true)"
        ));
        assert!(
            calls < adds,
            "the previous remover runs before the new listener is added"
        );
        assert!(
            adds < stores,
            "the new remover is stored after the listener is added"
        );
        assert!(stores.is_some(), "the remover is stored on the window");
    }

    #[test]
    fn the_away_listener_leaves_the_panel_under_an_open_overlay() {
        for layer in [
            ".alert-dialog-overlay",
            ".alert-dialog",
            ".sheet-backdrop",
            ".bottom-sheet",
            ".toast-container",
        ] {
            assert!(
                NAV_AWAY_JS.contains(layer),
                "{layer} is not an ignored layer"
            );
        }
        assert!(NAV_AWAY_JS.contains(r#".alert-dialog[data-state="open"]"#));
        assert!(NAV_AWAY_JS.contains(".bottom-sheet.show"));
    }
}
