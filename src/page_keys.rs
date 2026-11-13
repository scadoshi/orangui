//! The keys every page answers on a computer ([`orangui_core::page_keys`]): `g` `g` to the top,
//! Shift+G to the bottom, `j` and `k` a step down and up where the host turns steps on, `t` the
//! theme sheet and Shift+H the shortcuts sheet. Never while typing in a field, with Ctrl, Alt or
//! Meta held, or while a dialog or sheet is open, and never off the computer layout
//! ([`COMPUTER_QUERY`]). A key a page's own listener already took (`preventDefault`) passes by, so
//! a page whose keys use `j` and `k` keeps them.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use orangui_core::page_keys::{GG_WINDOW_MS, PAGE_KEYS, PageKey, STEP_SHARE, page_key};

/// The query for a computer: a pointer that hovers and is precise, which a phone's touch is not,
/// with the window on the wide layout, from 900px.
pub const COMPUTER_QUERY: &str = "(hover: hover) and (pointer: fine) and (min-width: 900px)";

/// The `data-action` on the button that opens the theme sheet, which `t` presses:
/// [`ThemePicker`](crate::ThemePicker)'s trigger carries it, and a host's own theme button can.
pub const THEMES_ACTION: &str = "themes";

/// The key a keydown `e` stands for, as a script expression: `KeyboardEvent.key`, with a letter's
/// case taken from Shift alone, so Caps Lock never turns `g` into Shift+G.
pub const PRESSED_KEY: &str = "((k, shift) => k.length === 1 && k.toLowerCase() !== k.toUpperCase() ? (shift ? k.toUpperCase() : k.toLowerCase()) : k)(e.key, e.shiftKey)";

/// Open overlays that keep a page's keys: a dialog and a sheet or drawer.
const OVERLAYS: &str = ".alert-dialog[data-state=\"open\"], .bottom-sheet.show";

/// Not typing in a field and no Ctrl, Alt or Meta, as a script expression over a keydown `e`.
const NOT_TYPING: &str = "(() => { const f = document.activeElement; const typing = !!f && (['INPUT', 'TEXTAREA', 'SELECT'].includes(f.tagName) || f.isContentEditable); return !typing && !(e.ctrlKey || e.altKey || e.metaKey) && !e.isComposing; })()";

/// The guard for an open sheet's own keys, as a script expression over a keydown `e`: not typing
/// and no Ctrl, Alt or Meta, and the topmost open dialog or sheet is a shown sheet holding an
/// element that matches `holding`.
#[must_use]
pub(crate) fn top_sheet_guard_js(holding: &str) -> String {
    format!(
        "({NOT_TYPING} && (() => {{ const open = [...document.querySelectorAll('{OVERLAYS}')]; const top = open[open.length - 1]; return !!top && top.matches('.bottom-sheet.show') && !!top.querySelector('{holding}'); }})())"
    )
}

/// What a host turns on in [`use_page_keys`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageKeys {
    /// Whether `t` presses the button marked [`THEMES_ACTION`].
    pub themes: bool,
    /// The shortcuts sheet's open state, which Shift+H sets; `None` leaves Shift+H alone.
    pub shortcuts: Option<Signal<bool>>,
    /// More open overlays that keep the keys, as selectors, beyond a dialog and a sheet.
    pub blocked_by: &'static [&'static str],
    /// The element the page scrolls in, as a selector; `None` or no match scrolls the document.
    pub scroller: Option<&'static str>,
}

/// The next registration's key among the page's listeners, unique across instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Registers the page keys under `__ID__`: the newest registration answers. It listens on the
/// window after the page's own listeners, sends `[key, after_g]` for a page key under the
/// guard, and remembers a first `g` for `__GG__` milliseconds. Steps answer only while
/// `__oranguiSteps__ID__` is true.
const KEYS_JS: &str = r"
window.__oranguiPageKeys = window.__oranguiPageKeys || [];
window.__oranguiPageKeys.push('__ID__');
let lastG = 0;
const onKey = (e) => {
    const stack = window.__oranguiPageKeys;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    if (e.defaultPrevented) return;
    const key = __KEY__;
    if (!__KEYS__.includes(key)) { lastG = 0; return; }
    if (!matchMedia('__COMPUTER__').matches || !__NOT_TYPING__) return;
    if (document.querySelector('__BLOCKED__')) return;
    if ((key === 'j' || key === 'k') && !window['__oranguiSteps__ID__']) return;
    const now = Date.now();
    const afterG = key === 'g' && now - lastG < __GG__;
    lastG = key === 'g' && !afterG ? now : 0;
    e.preventDefault();
    dioxus.send([key, afterG]);
};
window['__oranguiPageKey__ID__'] = onKey;
window.addEventListener('keydown', onKey);
";

/// Removes one registration.
const KEYS_DROP_JS: &str = r"
const onKey = window['__oranguiPageKey__ID__'];
if (onKey) window.removeEventListener('keydown', onKey);
delete window['__oranguiPageKey__ID__'];
delete window['__oranguiSteps__ID__'];
window.__oranguiPageKeys = (window.__oranguiPageKeys || []).filter((id) => id !== '__ID__');
";

/// Scrolls the page by `__BY__` at once, as vim does: `top` and `bottom` jump to an end, and a
/// signed step count moves that many steps of the view's height.
const SCROLL_JS: &str = r"
(() => {
    const sel = __SCROLLER__;
    const found = sel && document.querySelector(sel);
    const el = found && found.scrollHeight > found.clientHeight ? found : document.scrollingElement;
    if (!el) return;
    const by = __BY__;
    const step = Math.max(40, (found ? el.clientHeight : window.innerHeight) * __SHARE__);
    if (by === 'top') el.scrollTo({ top: 0, behavior: 'auto' });
    else if (by === 'bottom') el.scrollTo({ top: el.scrollHeight, behavior: 'auto' });
    else el.scrollBy({ top: by * step, behavior: 'auto' });
})();
";

/// Answers the page keys for the page, as the module describes. `steps` turns `j` and `k` on and
/// off as the page changes. Call once, above every route.
pub fn use_page_keys(keys: PageKeys, steps: ReadSignal<bool>) {
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_effect(move || {
        document::eval(&format!("window['__oranguiSteps{id}'] = {};", steps()));
    });
    use_hook(move || {
        let script = keys_js(id, keys.blocked_by);
        spawn(async move {
            let mut pressed = document::eval(&script);
            while let Ok((key, after_g)) = pressed.recv::<(String, bool)>().await {
                let Some(action) = page_key(&key, after_g) else {
                    continue;
                };
                match action {
                    PageKey::Top => scroll(keys.scroller, "'top'"),
                    PageKey::Bottom => scroll(keys.scroller, "'bottom'"),
                    PageKey::StepDown => scroll(keys.scroller, "1"),
                    PageKey::StepUp => scroll(keys.scroller, "-1"),
                    PageKey::Themes => {
                        if keys.themes {
                            document::eval(&press_themes_js());
                        }
                    }
                    PageKey::Shortcuts => {
                        if let Some(mut open) = keys.shortcuts {
                            open.set(true);
                        }
                    }
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// [`KEYS_JS`] for the registration `id`, with `blocked_by` beside the dialogs and sheets.
fn keys_js(id: u64, blocked_by: &[&str]) -> String {
    let blocked = std::iter::once(OVERLAYS)
        .chain(blocked_by.iter().copied())
        .collect::<Vec<_>>()
        .join(", ");
    KEYS_JS
        .replace("__ID__", &id.to_string())
        .replace("__KEY__", PRESSED_KEY)
        .replace("__KEYS__", &format!("{PAGE_KEYS:?}"))
        .replace("__COMPUTER__", COMPUTER_QUERY)
        .replace("__NOT_TYPING__", NOT_TYPING)
        .replace("__BLOCKED__", &blocked)
        .replace("__GG__", &GG_WINDOW_MS.to_string())
}

/// Scrolls the page `by` (a script value: `'top'`, `'bottom'` or a signed step count).
fn scroll(scroller: Option<&str>, by: &str) {
    document::eval(&scroll_js(scroller, by));
}

/// [`SCROLL_JS`] for `scroller` and `by`.
fn scroll_js(scroller: Option<&str>, by: &str) -> String {
    let scroller = scroller.map_or_else(|| "null".to_owned(), |sel| format!("'{sel}'"));
    SCROLL_JS
        .replace("__SCROLLER__", &scroller)
        .replace("__BY__", by)
        .replace("__SHARE__", &STEP_SHARE.to_string())
}

/// Clicks the first button marked [`THEMES_ACTION`].
fn press_themes_js() -> String {
    format!("document.querySelector('[data-action=\"{THEMES_ACTION}\"]')?.click();")
}

/// True on a computer's wide layout ([`COMPUTER_QUERY`]), kept current as the window changes.
/// Off the browser it stays false.
#[must_use]
pub fn use_computer() -> Signal<bool> {
    let mut matches = use_signal(|| false);
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_hook(move || {
        let script = MEDIA_JS.replace("__ID__", &id.to_string());
        spawn(async move {
            let mut changes = document::eval(&script);
            while let Ok(now) = changes.recv::<bool>().await {
                matches.set(now);
            }
        });
    });
    use_drop(move || {
        document::eval(&MEDIA_DROP_JS.replace("__ID__", &id.to_string()));
    });
    matches
}

/// Sends whether the window is a computer's, then again on each change.
const MEDIA_JS: &str = r"
const query = window.matchMedia('(hover: hover) and (pointer: fine) and (min-width: 900px)');
const onChange = (e) => dioxus.send(e.matches);
window['__oranguiMedia__ID__'] = [query, onChange];
dioxus.send(query.matches);
query.addEventListener('change', onChange);
";

/// Stops one media listener.
const MEDIA_DROP_JS: &str = r"
const held = window['__oranguiMedia__ID__'];
if (held) held[0].removeEventListener('change', held[1]);
delete window['__oranguiMedia__ID__'];
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keys_stay_off_while_typing_or_with_an_overlay_open_or_off_a_computer() {
        let js = keys_js(0, &[".big-modal"]);
        assert!(
            js.contains(&format!("matchMedia('{COMPUTER_QUERY}')")),
            "{js}"
        );
        assert!(
            js.contains("['INPUT', 'TEXTAREA', 'SELECT'].includes(f.tagName)"),
            "{js}"
        );
        assert!(js.contains("!(e.ctrlKey || e.altKey || e.metaKey)"), "{js}");
        assert!(
            js.contains("document.querySelector('.alert-dialog[data-state=\"open\"], .bottom-sheet.show, .big-modal')"),
            "a host's own overlays block too: {js}"
        );
    }

    #[test]
    fn a_key_the_page_took_passes_by_and_steps_wait_for_the_host() {
        let js = keys_js(0, &[]);
        assert!(js.contains("if (e.defaultPrevented) return;"), "{js}");
        assert!(
            js.contains("window.addEventListener('keydown', onKey)"),
            "after the page's own listeners: {js}"
        );
        assert!(
            js.contains("(key === 'j' || key === 'k') && !window['__oranguiSteps0']"),
            "{js}"
        );
    }

    #[test]
    fn a_first_g_waits_for_the_second() {
        let js = keys_js(0, &[]);
        assert!(
            js.contains(&format!("now - lastG < {GG_WINDOW_MS}")),
            "{js}"
        );
        assert!(
            js.contains("lastG = key === 'g' && !afterG ? now : 0;"),
            "{js}"
        );
    }

    #[test]
    fn the_listener_names_the_core_s_keys() {
        assert!(keys_js(0, &[]).contains(&format!("{PAGE_KEYS:?}")));
    }

    #[test]
    fn scrolling_jumps_to_the_ends_or_a_step_at_once() {
        let top = scroll_js(None, "'top'");
        assert!(top.contains("const sel = null;"), "{top}");
        assert!(
            top.contains("if (by === 'top') el.scrollTo({ top: 0, behavior: 'auto' })"),
            "{top}"
        );
        let step = scroll_js(Some(".main"), "1");
        assert!(step.contains("const sel = '.main';"), "{step}");
        assert!(
            step.contains("el.scrollBy({ top: by * step, behavior: 'auto' })"),
            "{step}"
        );
        assert!(step.contains(&format!("* {STEP_SHARE})")), "{step}");
        assert!(
            !step.contains("smooth"),
            "no glide that needs animation frames: {step}"
        );
    }

    #[test]
    fn t_presses_the_themes_button() {
        assert_eq!(
            press_themes_js(),
            "document.querySelector('[data-action=\"themes\"]')?.click();"
        );
    }

    #[test]
    fn the_media_listener_asks_the_computer_query() {
        assert!(MEDIA_JS.contains(COMPUTER_QUERY), "{MEDIA_JS}");
    }

    #[test]
    fn an_open_sheet_s_keys_need_it_on_top() {
        let js = top_sheet_guard_js(".mark");
        assert!(
            js.contains("top.matches('.bottom-sheet.show') && !!top.querySelector('.mark')"),
            "{js}"
        );
    }
}
