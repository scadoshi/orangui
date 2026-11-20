//! The theme sheets' search field, over the themes in [`ThemeSheet`](crate::ThemeSheet) and
//! [`ThemePicker`](crate::ThemePicker). Typing narrows the themes to those whose name or color
//! section holds the text ([`filter_theme_groups`](crate::filter_theme_groups)); Enter applies the
//! top match and closes the sheet, the down arrow moves into the themes, where the theme cursor
//! takes over, and Escape puts the cursor on the top theme shown, a second Escape closing the
//! sheet. The kit's clear
//! ([`ClearWrap`]) empties it and keeps it in focus. `t` opens a sheet with this field in focus; a tap or click opens it as
//! before, the field waiting for a tap. Styled by `assets/components.css`.

use dioxus::prelude::*;

use crate::ClearWrap;

/// The field's Escape, installed once per page: with the search field in focus, Escape leaves it
/// for the top theme shown (the filtered list's first), where the theme cursor's keys carry on, or
/// simply leaves it when no theme matches; a second Escape closes the sheet.
/// It listens first, on the window as the event starts out, so nothing else sees that Escape.
const SEARCH_KEYS_JS: &str = "if (!window.__oranguiThemeSearchKeys) { window.__oranguiThemeSearchKeys = true; window.addEventListener('keydown', (e) => { if (e.key !== 'Escape' || e.isComposing) return; const field = document.activeElement && document.activeElement.closest && document.activeElement.closest('.theme-search'); if (!field) return; e.preventDefault(); e.stopImmediatePropagation(); if (window.__oranguiThemeFocus) window.__oranguiThemeFocus(0); if (!document.activeElement || document.activeElement === field) field.blur(); }, true); }";

/// Focuses the open sheet's search field.
const FOCUS_FIELD_JS: &str =
    "document.querySelector('.bottom-sheet.show .theme-search')?.focus({ preventScroll: true });";

/// Focuses the first theme row shown, once the theme cursor has installed its focus helper.
const FOCUS_FIRST_JS: &str = "window.__oranguiThemeFocus && window.__oranguiThemeFocus(0);";

/// The theme Enter in the search applies, which its row shows: the top match `top` while `query`
/// has text other than spaces, else none, so an empty search leaves every row plain until the
/// cursor moves.
pub(crate) fn enter_target(query: &str, top: Option<&'static str>) -> Option<&'static str> {
    if query.trim().is_empty() { None } else { top }
}

/// What Enter in the search does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnterPress {
    /// Applies the highlighted theme and closes.
    Apply(&'static str),
    /// Closes, applying nothing: the search is empty, so no theme is highlighted.
    Close,
    /// Leaves the sheet open as it is: the search matches nothing, so the typing can be fixed.
    Stay,
}

/// What Enter does for `query` with top match `top`: applies the theme [`enter_target`]
/// highlights; with an empty search only closes; with text that matches nothing stays open. Enter
/// never applies a theme the sheet doesn't show as its target.
pub(crate) fn enter_press(query: &str, top: Option<&'static str>) -> EnterPress {
    match enter_target(query, top) {
        Some(name) => EnterPress::Apply(name),
        None if query.trim().is_empty() => EnterPress::Close,
        None => EnterPress::Stay,
    }
}

/// The search field, its `Search` label chip on its left. `query` holds what is typed;
/// `on_enter` runs as Enter is pressed in it.
#[component]
pub(crate) fn ThemeSearch(mut query: Signal<String>, on_enter: EventHandler<()>) -> Element {
    use_effect(|| {
        document::eval(SEARCH_KEYS_JS);
    });
    rsx! {
        // The bar holds the field at the top of the sheet's scroll as the themes pass under it.
        div { class: "theme-search-bar",
            // The row is the field's label, so a click on the chip focuses the field.
            label { class: "theme-search-row",
            span { class: "stat-chip stat-chip-p1 theme-search-label", "Search" }
            ClearWrap {
                filled: !query().is_empty(),
                label: "Clear search".to_string(),
                on_clear: move |()| {
                    query.set(String::new());
                    document::eval(FOCUS_FIELD_JS);
                },
                input {
                    class: "theme-search",
                    r#type: "text",
                    role: "searchbox",
                    enterkeyhint: "search",
                    placeholder: "Search themes",
                    "aria-label": "Search themes",
                    autocomplete: "off",
                    spellcheck: "false",
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                    onkeydown: move |e| {
                        let key = e.key();
                        if key == Key::Enter {
                            e.prevent_default();
                            on_enter.call(());
                        } else if key == Key::ArrowDown {
                            e.prevent_default();
                            document::eval(FOCUS_FIRST_JS);
                        }
                    },
                }
            }
            }
        }
    }
}

/// What a sheet shows in place of its themes when a search matches none: the kit's no-results
/// chip.
#[component]
pub(crate) fn NoThemeMatch() -> Element {
    rsx! {
        div { class: "theme-search-empty",
            div { class: "chip-unselected", "No themes match" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_applies_only_the_highlighted_theme_and_otherwise_only_closes() {
        let top = Some("Darcula");
        assert_eq!(
            enter_press("", top),
            EnterPress::Close,
            "an empty search highlights nothing, so Enter only closes"
        );
        assert_eq!(enter_press("dar", top), EnterPress::Apply("Darcula"));
        assert_eq!(
            enter_press("zzzz", None),
            EnterPress::Stay,
            "a typo with no match keeps the sheet open to fix it"
        );
        assert_eq!(
            enter_press("   ", top),
            EnterPress::Close,
            "spaces alone are an empty search"
        );
        assert_eq!(enter_target("  ", top), None, "and highlight nothing");
    }

    /// The sequence `t`, type, Escape, `j`, Escape: Escape in the field takes no other listener's
    /// Escape and puts the cursor on the top row shown, the filtered list's first; `j` from there
    /// steps to the next row; and with focus on a row in the sheet, the second Escape closes it.
    #[test]
    fn escape_in_the_field_puts_the_cursor_on_the_top_theme_then_a_second_escape_closes() {
        for want in [
            "window.addEventListener('keydown'",
            "e.key !== 'Escape'",
            "closest('.theme-search')",
            "e.preventDefault();",
            "e.stopImmediatePropagation();",
            "window.__oranguiThemeFocus(0);",
            "}, true);",
        ] {
            assert!(SEARCH_KEYS_JS.contains(want), "{want} in {SEARCH_KEYS_JS}");
        }
        assert!(
            SEARCH_KEYS_JS.contains(
                "if (!document.activeElement || document.activeElement === field) field.blur();"
            ),
            "with no row to land on, Escape still leaves the field: {SEARCH_KEYS_JS}"
        );
        assert_eq!(
            orangui_core::grid_cursor::move_for("j"),
            Some(orangui_core::grid_cursor::Move::Down),
            "j steps down from the top row"
        );
        let trap = crate::sheet_focus::TRAP_JS;
        assert!(
            trap.contains("top.contains(a)")
                && trap.contains("if (close) { e.preventDefault(); close.click(); }"),
            "the second Escape, from a row inside the sheet, closes it: {trap}"
        );
    }

    #[test]
    fn clearing_gives_the_field_its_focus_back() {
        assert_eq!(
            FOCUS_FIELD_JS,
            "document.querySelector('.bottom-sheet.show .theme-search')?.focus({ preventScroll: true });"
        );
    }

    #[test]
    fn the_top_match_is_enter_s_target_only_while_the_search_has_text() {
        let found = crate::filter_theme_groups("gru");
        let top = crate::top_theme(&found);
        assert!(top.is_some(), "gru matches a theme");
        assert_eq!(enter_target("gru", top), top);
        assert_eq!(
            enter_target("", top),
            None,
            "an empty search leaves the rows plain"
        );
        assert_eq!(enter_target("zzzz", None), None, "no match, no target");
    }

    #[test]
    fn the_target_wears_the_cursor_s_highlight_while_the_field_has_focus() {
        let css = include_str!("../assets/components.css");
        assert!(
            css.contains(".bottom-sheet:has(.theme-search:focus) .pref-row.enter-target"),
            "the highlight shows while Enter is the field's"
        );
    }
}
