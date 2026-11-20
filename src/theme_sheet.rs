//! The app's theme sheet: every allowed theme as a row, previewed live as it
//! is picked. Styled by `assets/app.css`.
//!
//! What a save means (an API call, a local store) and what each outcome says
//! (a toast) stay the host's, through [`ThemeSheet`]'s callbacks.
//!
//! On a keyboard, while the sheet is open and no dialog is over it, the
//! arrows and `h` `j` `k` `l` move focus through the rows (the theme cursor),
//! Enter picks the focused one, and Enter with no row in focus closes the
//! sheet. The search field over the rows keeps its own Enter, which applies
//! the top match ([`ThemeSearch`]).

use dioxus::prelude::*;

use crate::theme_search::{EnterPress, NoThemeMatch, ThemeSearch, enter_press};
use crate::{
    BottomSheet, SheetDim, ThemeConfig, display_theme_name, filter_theme_groups, top_theme,
};

/// The sheet's Enter, installed once per page by [`ThemeSheet`]: with no
/// button or link in the open sheet in focus, Enter closes it, except in the
/// search field, whose Enter is its own. The theme
/// cursor takes the moves. Nothing happens while a dialog is open.
const THEME_KEYS_JS: &str = r#"
if (!window.__oranguiThemeKeys) {
    window.__oranguiThemeKeys = true;
    document.addEventListener('keydown', (e) => {
        if (e.isComposing || document.querySelector('.alert-dialog[data-state="open"]')) return;
        const sheet = Array.from(document.querySelectorAll('.bottom-sheet.show'))
            .find((s) => s.querySelector('.theme-swatches'));
        if (!sheet) return;
        const focus = document.activeElement;
        if (focus && focus.closest('.theme-search')) return;
        const onControl = focus && (focus.tagName === 'BUTTON' || focus.tagName === 'A');
        if (e.key === 'Enter' && !(onControl && sheet.contains(focus))) {
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
    /// Whether Enter in the search applies this theme, which the row shows as the cursor does.
    enter_target: bool,
    theme: String,
    mode: String,
    mut selected: Signal<String>,
    mut live: Signal<ThemeConfig>,
    dark: Memo<bool>,
    on_pick: EventHandler<ThemeConfig>,
) -> Element {
    let is_selected = selected() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            r#type: "button",
            class: match (is_selected, enter_target) {
                (true, true) => "pref-row selected enter-target",
                (true, false) => "pref-row selected",
                (false, true) => "pref-row enter-target",
                (false, false) => "pref-row",
            },
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
/// On a keyboard the theme cursor walks the rows, Enter picks, and Enter with no row in focus closes (see the module
/// docs). With `search` off the sheet is the rows alone. Registers with the [`OverlayBackStack`](crate::OverlayBackStack)
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
    /// Whether the search field sits over the themes. The sites and the web app keep it; a native
    /// app leaves it out.
    #[props(default = true)]
    search: bool,
) -> Element {
    use_effect(|| {
        document::eval(THEME_KEYS_JS);
    });
    crate::theme_cursor::use_theme_cursor(open);
    let mut selected = use_signal(|| theme.peek().name.clone());
    // The mode follows the live theme, so a mode flipped while the sheet is up shows in its dots
    // and stays through a pick.
    let dark = use_memo(move || theme.read().is_dark);
    let mut query = use_signal(String::new);

    // Opening reads the live theme, so the sheet marks what the app draws.
    use_effect(move || {
        if open() {
            selected.set(theme.peek().name.clone());
            query.set(String::new());
        }
    });

    let mode = (if dark() { "dark" } else { "light" }).to_string();
    let found = filter_theme_groups(&query());
    let top = top_theme(&found);
    let target = crate::theme_search::enter_target(&query(), top);
    // Enter in the search applies the highlighted match, as a click on its row does, and
    // closes; an empty search only closes; a search that matches nothing stays open.
    let apply_top = move |()| match enter_press(&query.peek(), top) {
        EnterPress::Apply(name) => {
            if *selected.peek() != name {
                selected.set(name.to_string());
                let picked = ThemeConfig {
                    name: name.to_string(),
                    is_dark: *dark.peek(),
                };
                theme.set(picked.clone());
                on_save.call(picked);
            }
            open.set(false);
        }
        EnterPress::Close => open.set(false),
        EnterPress::Stay => {}
    };

    rsx! {
        BottomSheet {
            open,
            title: "Themes".to_string(),
            dim: SheetDim::Phone,
            hint,
            closable: true,
            on_dismiss: move |()| open.set(false),
            if search {
                ThemeSearch { query, on_enter: apply_top }
            }
            if found.is_empty() {
                NoThemeMatch {}
            }
            for group in found {
                div { class: "theme-group-label",
                    span { class: "stat-chip stat-chip-p1", "{group.label}" }
                }
                for t in group.themes {
                    ThemeRow {
                        enter_target: target == Some(t),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_open_sheet_starts_with_its_search_field() {
        fn app() -> Element {
            let open = use_signal(|| true);
            let theme = use_signal(ThemeConfig::default);
            rsx! { ThemeSheet { open, theme, on_save: |_| {} } }
        }
        let html = render(app);
        let field = html.find(r#"class="theme-search""#);
        let first_group = html.find("theme-group-label");
        assert!(field.is_some(), "{html}");
        assert!(field < first_group, "above the themes: {html}");
        assert!(
            html.contains(r#"type="text""#) && html.contains(r#"role="searchbox""#),
            "a text field, so no browser draws its own clear: {html}"
        );
        assert!(!html.contains(r#"type="search""#), "{html}");
        assert!(
            html.contains(r#"<div class="theme-search-bar"><label class="theme-search-row"><span class="stat-chip stat-chip-p1 theme-search-label">Search</span><div class="input-clear-wrap"><input class="theme-search""#),
            "in its sticky bar, its Search chip on its left in the field's label, with the kit's clear: {html}"
        );
        assert!(
            html.contains(r#"class="input-clear-btn""#) && !html.contains("has-text"),
            "the clear waits, hidden, for text: {html}"
        );
        assert!(html.contains(r#"placeholder="Search themes""#), "{html}");
    }

    #[test]
    fn nothing_is_enter_s_target_with_an_empty_search() {
        fn app() -> Element {
            let open = use_signal(|| true);
            let theme = use_signal(ThemeConfig::default);
            rsx! { ThemeSheet { open, theme, on_save: |_| {} } }
        }
        let html = render(app);
        assert!(!html.contains("enter-target"), "{html}");
    }

    #[test]
    fn enter_in_the_search_is_the_fields_own() {
        assert!(
            THEME_KEYS_JS.contains("focus.closest('.theme-search')"),
            "{THEME_KEYS_JS}"
        );
    }
}
