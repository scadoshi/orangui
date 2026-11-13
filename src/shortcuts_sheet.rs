//! The keyboard shortcuts sheet: one row per key, its keys in the hint's key style at the left and
//! what it does after them, above every overlay. Focus moves to its close button as it opens and
//! back as it closes. A host lists its rows as [`ShortcutRow`]s, or the page keys' own with
//! [`PageShortcuts`].

use dioxus::prelude::*;
use orangui_core::page_keys::page_rows;

use crate::sheet_focus::use_sheet_focus;
use crate::{BottomSheet, HintKey, SheetFrame, SheetWide};

/// The sheet's title, also the name of a button that opens it.
pub const SHORTCUTS_TITLE: &str = "Keyboard shortcuts";

/// What takes focus as the sheet opens: its close button, so Esc, Enter and Tab act on the sheet
/// however it opened.
const SHEET_FOCUS: &str = ".shortcuts-layer .bottom-sheet.show .sheet-close";

/// The shortcuts sheet, `note` the line above the keys when the page needs one, `children` its
/// [`ShortcutRow`]s, `rows` how many there are: given, a wide window lays them in two columns, read
/// top to bottom then across.
#[component]
pub fn ShortcutsSheet(
    open: Signal<bool>,
    #[props(default)] note: Option<String>,
    #[props(default)] rows: usize,
    children: Element,
) -> Element {
    let always = use_signal(|| true);
    use_sheet_focus(open.into(), always.into(), SHEET_FOCUS, "");
    rsx! {
        div { class: "shortcuts-layer",
            SheetFrame { wide: SheetWide::Modal,
                BottomSheet { open, title: SHORTCUTS_TITLE.to_string(), closable: true,
                    if let Some(note) = note {
                        p { class: "shortcuts-note", "{note}" }
                    }
                    ul { class: "shortcuts", style: column_style(rows), {children} }
                }
            }
        }
    }
}

/// The list's style for `rows` rows: the first column's length, half of them rounded up, when
/// there are any.
fn column_style(rows: usize) -> Option<String> {
    (rows > 0).then(|| format!("--shortcut-rows: {}", rows.div_ceil(2)))
}

/// One row: `keys` as the sheet writes them, each in the hint's key style, and what they do.
#[component]
pub fn ShortcutRow(keys: Vec<String>, children: Element) -> Element {
    rsx! {
        li { class: "shortcut",
            span { class: "shortcut-keys",
                for (i, label) in keys.into_iter().enumerate() {
                    span { key: "{i}",
                        if i > 0 {
                            " "
                        }
                        HintKey { "{label}" }
                    }
                }
            }
            span { class: "shortcut-does", {children} }
        }
    }
}

/// The page keys' rows ([`orangui_core::page_keys::page_rows`]), the step rows where `steps`; give
/// the sheet `page_rows(steps).len()` as its `rows`.
#[component]
pub fn PageShortcuts(steps: bool) -> Element {
    rsx! {
        for (at, (keys, does)) in page_rows(steps).into_iter().enumerate() {
            ShortcutRow { key: "{at}", keys: keys.iter().map(ToString::to_string).collect::<Vec<_>>(), "{does}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_column_takes_half_the_rows_rounded_up() {
        assert_eq!(column_style(23).as_deref(), Some("--shortcut-rows: 12"));
        assert_eq!(column_style(10).as_deref(), Some("--shortcut-rows: 5"));
        assert_eq!(column_style(0), None, "no count, one column");
    }

    #[test]
    fn two_columns_only_on_a_wide_window_and_only_with_a_count() {
        let css = crate::COMPONENTS_CSS;
        let wide = css
            .find("@media (min-width: 60rem) {\n    .shortcuts-layer")
            .expect("the wide rule");
        let rule = css
            .get(wide..)
            .and_then(|rest| rest.get(..800))
            .unwrap_or_default();
        assert!(
            rule.contains(".shortcuts[style*=\"--shortcut-rows\"]"),
            "{rule}"
        );
        assert!(
            rule.contains("grid-auto-flow: column;"),
            "top to bottom then across: {rule}"
        );
        assert!(
            rule.contains("width: min(52rem, calc(100vw - 4rem));"),
            "{rule}"
        );
    }
}
