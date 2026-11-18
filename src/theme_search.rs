//! The theme sheets' search field, over the themes in [`ThemeSheet`](crate::ThemeSheet) and
//! [`ThemePicker`](crate::ThemePicker). Typing narrows the themes to those whose name or color
//! section holds the text ([`filter_theme_groups`](crate::filter_theme_groups)); Enter applies the
//! top match and closes the sheet, and the down arrow moves into the themes, where the theme
//! cursor takes over. `t` opens a sheet with this field in focus; a tap or click opens it as
//! before, the field waiting for a tap. Styled by `assets/components.css`.

use dioxus::prelude::*;

/// Focuses the first theme row shown, once the theme cursor has installed its focus helper.
const FOCUS_FIRST_JS: &str = "window.__oranguiThemeFocus && window.__oranguiThemeFocus(0);";

/// The search field. `query` holds what is typed; `on_enter` runs as Enter is pressed in it.
#[component]
pub(crate) fn ThemeSearch(mut query: Signal<String>, on_enter: EventHandler<()>) -> Element {
    rsx! {
        input {
            class: "theme-search",
            r#type: "search",
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

/// What a sheet shows in place of its themes when a search matches none.
#[component]
pub(crate) fn NoThemeMatch() -> Element {
    rsx! {
        p { class: "theme-search-empty", "No themes match" }
    }
}
