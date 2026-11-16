//! Copy that names an icon button by its icon: `[[pencil]]` in a line of text draws the pencil in
//! a [`HintKey`] squircle, as a hint's key does, so a toast or a note never names a button by a
//! word the button no longer shows. [`spoken`] gives the same line in words, for a screen reader.

use dioxus::prelude::*;

use crate::icons::{DockIcon, PENCIL};
use crate::ink;

/// Each icon a line may name, by its token's name, with its path and the words a screen reader
/// reads in its place.
const ICONS: [(&str, &str, &str); 1] = [("pencil", PENCIL, "the pencil")];

/// One piece of a line: its words, or an icon by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece<'text> {
    /// Words, drawn as they are.
    Text(&'text str),
    /// A known icon: its path and its spoken words.
    Icon(&'static str, &'static str),
}

/// `text` in pieces: each `[[name]]` naming a known icon becomes that icon, and anything else,
/// an unknown name or a lone bracket included, stays as words.
fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        let after = rest.get(start.saturating_add(2)..).unwrap_or_default();
        let Some(end) = after.find("]]") else {
            break;
        };
        let name = after.get(..end).unwrap_or_default();
        let Some(&(_, path, spoken)) = ICONS.iter().find(|(known, _, _)| *known == name) else {
            out.push(Piece::Text(
                rest.get(..start.saturating_add(2)).unwrap_or_default(),
            ));
            rest = after;
            continue;
        };
        if start > 0 {
            out.push(Piece::Text(rest.get(..start).unwrap_or_default()));
        }
        out.push(Piece::Icon(path, spoken));
        rest = after.get(end.saturating_add(2)..).unwrap_or_default();
    }
    if !rest.is_empty() {
        out.push(Piece::Text(rest));
    }
    out
}

/// Whether `text` names an icon [`IconText`] draws.
#[must_use]
pub fn names_an_icon(text: &str) -> bool {
    pieces(text).iter().any(|p| matches!(p, Piece::Icon(..)))
}

/// `text` with each icon token read as its words: "Tap `[[pencil]]` to send a new one" reads "Tap
/// the pencil to send a new one".
#[must_use]
pub fn spoken(text: &str) -> String {
    pieces(text)
        .into_iter()
        .map(|piece| match piece {
            Piece::Text(words) | Piece::Icon(_, words) => words,
        })
        .collect()
}

/// The color a token's key is drawn in: a [`HintKey`](crate::HintKey)'s default.
const KEY_COLOR: &str = "--palette-2";

/// A line of copy with its icon tokens drawn as the icons, each in a hint key's squircle (the
/// markup a [`HintKey`](crate::HintKey) draws), read aloud by its words.
#[component]
pub fn IconText(
    /// The line, with `[[pencil]]` where the pencil goes.
    text: String,
) -> Element {
    let ink = ink::ink_color(KEY_COLOR);
    rsx! {
        for (i, piece) in pieces(&text).into_iter().enumerate() {
            match piece {
                Piece::Text(words) => rsx! { span { key: "{i}", "{words}" } },
                Piece::Icon(path, words) => rsx! {
                    span {
                        key: "{i}",
                        class: "hint-key",
                        role: "img",
                        "aria-label": words,
                        style: "border-color: var({KEY_COLOR}); color: {ink};",
                        DockIcon { path }
                    }
                },
            }
        }
    }
}

/// A script that draws each icon token in the shown toasts' titles as its key, the markup
/// [`IconText`] draws. A toast's title reaches the page as text and its props are sealed, so the
/// toast stack runs this as each toast mounts, again on the next frame and at 100 ms, since the
/// toast reaches the page through a portal; a title already drawn has no token left. The title is
/// the element the toast's dialog is labelled by, which carries no class.
#[cfg(feature = "toast")]
#[must_use]
pub(crate) fn toast_icons_js() -> String {
    let icons: Vec<String> = ICONS
        .iter()
        .map(|(name, path, words)| format!("{name:?}: {{ d: {path:?}, label: {words:?} }}"))
        .collect();
    let ink = ink::ink_color(KEY_COLOR);
    TOAST_ICONS_JS
        .replace("__ICONS__", &icons.join(", "))
        .replace(
            "__STYLE__",
            &format!("border-color: var({KEY_COLOR}); color: {ink};"),
        )
}

/// [`toast_icons_js`] before its icons and key style are filled in.
#[cfg(feature = "toast")]
const TOAST_ICONS_JS: &str = r#"
const icons = { __ICONS__ };
const draw = () => document.querySelectorAll('[role="alertdialog"] [id$="-label"]').forEach((title) => {
    if (!title.textContent.includes('[[')) return;
    const parts = title.textContent.split(/(\[\[[a-z]+\]\])/);
    title.textContent = '';
    for (const part of parts) {
        const named = part.match(/^\[\[([a-z]+)\]\]$/);
        const icon = named && icons[named[1]];
        if (!icon) {
            title.appendChild(document.createTextNode(part));
            continue;
        }
        const key = document.createElement('span');
        key.className = 'hint-key';
        key.setAttribute('role', 'img');
        key.setAttribute('aria-label', icon.label);
        key.style.cssText = '__STYLE__';
        key.innerHTML = '<svg class="action-dock-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><path d="' + icon.d + '"></path></svg>';
        title.appendChild(key);
    }
});
// The toast reaches the page through a portal a moment after its effect runs.
draw();
requestAnimationFrame(draw);
setTimeout(draw, 100);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_pencil_token_draws_the_pencil_in_a_key_between_the_words() {
        fn app() -> Element {
            rsx! { IconText { text: "Tap [[pencil]] to send a new one" } }
        }
        let html = render(app);
        let pencil = html.find(&format!(r#"d="{PENCIL}""#));
        let tap = html.find("Tap ");
        let rest = html.find(" to send a new one");
        assert!(
            tap < pencil && pencil < rest && tap.is_some() && rest.is_some(),
            "the words, the pencil, the words: {html}"
        );
        assert!(
            html.contains(r#"class="hint-key""#),
            "in a key's squircle: {html}"
        );
        assert!(
            html.contains(r#"role="img""#) && html.contains(r#"aria-label="the pencil""#),
            "read aloud as its words: {html}"
        );
        assert!(!html.contains("[["), "no token left: {html}");
    }

    #[test]
    fn a_line_reads_aloud_with_the_icon_in_words() {
        assert_eq!(
            spoken("That link expired. Tap [[pencil]] to send a new one"),
            "That link expired. Tap the pencil to send a new one"
        );
        assert_eq!(spoken("No icon here"), "No icon here");
    }

    #[cfg(feature = "toast")]
    #[test]
    fn the_toast_script_draws_the_same_key_for_every_icon() {
        let js = toast_icons_js();
        assert!(
            js.contains(&format!(
                r#""pencil": {{ d: "{PENCIL}", label: "the pencil" }}"#
            )),
            "{js}"
        );
        assert!(
            js.contains(r#"document.querySelectorAll('[role="alertdialog"] [id$="-label"]')"#),
            "{js}"
        );
        assert!(
            js.contains("key.setAttribute('role', 'img')")
                && js.contains("key.setAttribute('aria-label', icon.label)"),
            "reads aloud as its words: {js}"
        );
        assert!(
            !js.contains("__ICONS__") && !js.contains("__STYLE__"),
            "{js}"
        );
    }

    #[test]
    fn an_unknown_or_broken_token_stays_as_words() {
        assert_eq!(spoken("Tap [[gear]] now"), "Tap [[gear]] now");
        assert_eq!(spoken("Tap [[pencil"), "Tap [[pencil");
        assert!(names_an_icon("Tap [[pencil]]"));
        assert!(!names_an_icon("Tap [[gear]]"));
    }
}
