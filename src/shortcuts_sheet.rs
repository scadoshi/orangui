//! The keyboard shortcuts sheet: the keys in groups under small headings, each row its keys in
//! the hint's key style at the left and what they do after them, closed from a `Got it` bar at its
//! foot as a hint is, above every overlay. On a wide window the groups, each kept whole, fill two
//! columns. Focus moves to `Got it` as it opens and back as it closes. A host lists its groups as
//! [`ShortcutGroup`]s of [`ShortcutRow`]s, or the page keys' own with [`PageShortcuts`].

use dioxus::prelude::*;
use orangui_core::page_keys::page_groups;

use crate::sheet_focus::use_sheet_focus;
use crate::{BottomSheet, Button, ButtonVariant, HintKey, SheetFrame, SheetWide};

/// The sheet's title, also the name of a button that opens it.
pub const SHORTCUTS_TITLE: &str = "Keyboard shortcuts";

/// The label of the button that closes the sheet, as a hint's does.
pub const GOT_IT: &str = "Got it";

/// What takes focus as the sheet opens: its `Got it`, so Enter, Esc and Tab act on the sheet
/// however it opened.
const SHEET_FOCUS: &str = ".shortcuts-layer .bottom-sheet.show .shortcuts-got-it";

/// The shortcuts sheet, `note` the line above the keys when the page needs one, `children` its
/// [`ShortcutGroup`]s.
#[component]
pub fn ShortcutsSheet(
    mut open: Signal<bool>,
    #[props(default)] note: Option<String>,
    children: Element,
) -> Element {
    let always = use_signal(|| true);
    use_sheet_focus(open.into(), always.into(), SHEET_FOCUS, "");
    rsx! {
        div { class: "shortcuts-layer",
            SheetFrame { wide: SheetWide::Modal,
                BottomSheet {
                    open,
                    title: SHORTCUTS_TITLE.to_string(),
                    footer: rsx! {
                        Button {
                            variant: ButtonVariant::Action,
                            class: "shortcuts-got-it",
                            onclick: move |_| open.set(false),
                            "{GOT_IT}"
                        }
                    },
                    if let Some(note) = note {
                        p { class: "shortcuts-note", "{note}" }
                    }
                    div { class: "shortcut-groups", {children} }
                }
            }
        }
    }
}

/// One group: its `title` as a small heading over its [`ShortcutRow`]s.
#[component]
pub fn ShortcutGroup(title: String, children: Element) -> Element {
    rsx! {
        section { class: "shortcut-group",
            h3 { class: "shortcut-group-title", "{title}" }
            ul { class: "shortcuts", {children} }
        }
    }
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

/// The page keys' groups ([`orangui_core::page_keys::page_groups`]), the step rows where `steps`.
#[component]
pub fn PageShortcuts(steps: bool) -> Element {
    rsx! {
        for group in page_groups(steps) {
            ShortcutGroup { key: "{group.title}", title: group.title.to_string(),
                for (at, (keys, does)) in group.rows.into_iter().enumerate() {
                    ShortcutRow { key: "{at}", keys: keys.iter().map(ToString::to_string).collect::<Vec<_>>(), "{does}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn whole_groups_fill_two_columns_only_on_a_wide_window() {
        let css = crate::COMPONENTS_CSS;
        let wide = css
            .find("@media (min-width: 60rem) {\n    .shortcuts-layer")
            .expect("the wide rule");
        let rule = css
            .get(wide..)
            .and_then(|rest| rest.get(..700))
            .unwrap_or_default();
        assert!(
            rule.contains(".shortcut-groups {\n        columns: 2;"),
            "{rule}"
        );
        assert!(
            rule.contains("break-inside: avoid;"),
            "a group stays whole: {rule}"
        );
        assert!(
            rule.contains("width: min(52rem, calc(100vw - 4rem));"),
            "{rule}"
        );
    }

    #[test]
    fn the_sheet_closes_from_got_it_and_focus_starts_there() {
        assert_eq!(super::GOT_IT, "Got it");
        assert!(super::SHEET_FOCUS.ends_with(".shortcuts-got-it"));
    }
}
