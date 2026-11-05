//! The app's theme sheet: every allowed theme as a row, previewed live as it
//! is picked. Styled by `assets/app.css`.
//!
//! What a save means (an API call, a local store) and what each outcome says
//! (a toast) stay the host's, through [`ThemeSheet`]'s callbacks.
//!
//! On a keyboard, while the sheet is open and no dialog is over it, the up
//! and down arrows pick the row above or below the selected one, and Enter
//! closes the sheet unless a button or link inside the sheet has focus.

use dioxus::prelude::*;

use crate::{BottomSheet, THEME_GROUPS, ThemeConfig, display_theme_name};

/// The sheet's keys, installed once per page by [`ThemeSheet`]. The open
/// sheet is the shown `.bottom-sheet` holding theme swatches. An arrow clicks
/// the row above or below the selected one and scrolls it into view, staying
/// put at either end; Enter clicks the save check in the sheet's dock when it
/// is enabled. Nothing happens while a dialog is open.
const THEME_KEYS_JS: &str = r#"
if (!window.__oranguiThemeKeys) {
    window.__oranguiThemeKeys = true;
    document.addEventListener('keydown', (e) => {
        if (e.isComposing || document.querySelector('.alert-dialog[data-state="open"]')) return;
        const sheet = Array.from(document.querySelectorAll('.bottom-sheet.show'))
            .find((s) => s.querySelector('.theme-swatches'));
        if (!sheet) return;
        const focus = document.activeElement;
        const onControl = focus && (focus.tagName === 'BUTTON' || focus.tagName === 'A');
        if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
            const rows = Array.from(sheet.querySelectorAll('.pref-row'));
            const at = rows.findIndex((row) => row.classList.contains('selected'));
            const next = rows[e.key === 'ArrowDown' ? Math.min(at + 1, rows.length - 1) : Math.max(at - 1, 0)];
            e.preventDefault();
            if (next && next !== rows[at]) {
                next.click();
                next.scrollIntoView({ block: 'nearest' });
            }
        } else if (e.key === 'Enter' && !(onControl && sheet.contains(focus))) {
            const close = sheet.querySelector('.sheet-close');
            e.preventDefault();
            if (close) close.click();
        }
    }, true);
}
"#;

/// One selectable theme row: name on the left, swatch dots on the right. The
/// dots take their colors from the theme's own class, so colors stay defined
/// only in themes.css.
#[component]
fn ThemeRow(
    theme: String,
    mode: String,
    mut selected: Signal<String>,
    mut live: Signal<ThemeConfig>,
    dark: Signal<bool>,
    on_pick: EventHandler<ThemeConfig>,
) -> Element {
    let is_selected = selected() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| {
                if *selected.peek() == click_theme {
                    return;
                }
                selected.set(click_theme.clone());
                let picked = ThemeConfig {
                    name: click_theme.clone(),
                    is_dark: dark(),
                };
                live.set(picked.clone());
                on_pick.call(picked);
            },
            div { class: "pref-row-inner",
                span { "{display_theme_name(&theme)}" }
                div { class: "theme-swatches theme-{theme}-{mode}",
                    span { class: "theme-dot", style: "background:var(--palette-1)" }
                    span { class: "theme-dot", style: "background:var(--palette-2)" }
                    span { class: "theme-dot", style: "background:var(--palette-3)" }
                    span { class: "theme-dot", style: "background:var(--palette-4)" }
                    span { class: "theme-dot", style: "background:var(--palette-5)" }
                    span { class: "theme-dot", style: "background:var(--palette-6)" }
                }
            }
        }
    }
}

/// Bottom sheet for picking a theme. A tap draws the app in the pick by
/// setting `theme`, the app's live theme, and hands it to `on_save` at
/// once; the sheet stays open to keep browsing, and the ×, a drag down and
/// the backdrop only close it. Undoing a pick is a tap on the row before.
///
/// On a keyboard the arrows walk the rows and Enter closes (see the module
/// docs). Registers with the [`OverlayBackStack`](crate::OverlayBackStack)
/// through [`BottomSheet`].
#[component]
pub fn ThemeSheet(
    mut open: Signal<bool>,
    /// The app's live theme. Each pick is written here.
    mut theme: Signal<ThemeConfig>,
    /// Called with each pick as it is made.
    on_save: EventHandler<ThemeConfig>,
    /// A hint's open signal, for the sheet header's "?".
    #[props(default)]
    hint: Option<Signal<bool>>,
) -> Element {
    use_effect(|| {
        let _ = document::eval(THEME_KEYS_JS);
    });
    let mut selected = use_signal(|| theme.peek().name.clone());
    let mut dark = use_signal(|| theme.peek().is_dark);

    // Opening reads the live theme, so the sheet marks what the app draws.
    use_effect(move || {
        if open() {
            let current = theme.peek().clone();
            selected.set(current.name.clone());
            dark.set(current.is_dark);
        }
    });

    let mode = (if dark() { "dark" } else { "light" }).to_string();

    rsx! {
        BottomSheet {
            open,
            title: "Themes".to_string(),
            hint,
            closable: true,
            on_dismiss: move |()| open.set(false),
            for group in THEME_GROUPS {
                div { class: "theme-group-label",
                    span { class: "stat-chip stat-chip-p1", "{group.label}" }
                }
                for t in group.themes.iter().copied() {
                    ThemeRow {
                        theme: t.to_string(),
                        mode: mode.clone(),
                        selected,
                        live: theme,
                        dark,
                        on_pick: on_save,
                    }
                }
            }
        }
    }
}
