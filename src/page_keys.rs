//! The keys every page answers ([`orangui_core::page_keys`]), at any width, a phone with a
//! keyboard included: `g` `g` to the top, Shift+G to the bottom, `j` and `k` a step down and up
//! where the host turns steps on, `t` the theme sheet and Shift+H the shortcuts sheet. Never while
//! typing in a field or with Ctrl, Alt or Meta held. While a dialog or sheet is open only `g` `g`
//! and Shift+G answer, for it, and `j` and `k` too when it is a hint or the shortcuts sheet, so
//! either, taller than the window, steps like the page. A key a page's own listener already took (`preventDefault`) passes
//! by, so a page whose keys use `j` and `k` keeps them.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use orangui_core::page_keys::{GG_WINDOW_MS, PAGE_KEYS, PageKey, STEP_SHARE, page_key};

use crate::js_str::js_str;

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

/// What marks an open overlay as one to read, which `j` and `k` step through: a hint's lines
/// ([`HintDialog`](crate::HintDialog)'s body and the [`HintLine`](crate::HintLine) and
/// [`HintBullets`](crate::HintBullets) a host composes one from) and the shortcuts sheet's groups
/// ([`ShortcutsSheet`](crate::ShortcutsSheet)).
const HINT_PARTS: &str = ".hint-body, .hint-line, .hint-bullets, .shortcut-groups";

/// Not typing in a field and no Ctrl, Alt or Meta, as a script expression over a keydown `e`.
const NOT_TYPING: &str = "(() => { const f = document.activeElement; const typing = !!f && (['INPUT', 'TEXTAREA', 'SELECT'].includes(f.tagName) || f.isContentEditable); return !typing && !(e.ctrlKey || e.altKey || e.metaKey) && !e.isComposing; })()";

/// The guard for an open sheet's own keys, as a script expression over a keydown `e`: not typing
/// and no Ctrl, Alt or Meta, and the topmost open dialog or sheet is a shown sheet holding an
/// element that matches `holding`.
#[must_use]
pub(crate) fn top_sheet_guard_js(holding: &str) -> String {
    format!(
        "({NOT_TYPING} && (() => {{ const open = [...document.querySelectorAll('{OVERLAYS}')]; const top = open[open.length - 1]; return !!top && top.matches('.bottom-sheet.show') && !!top.querySelector({holding}); }})())",
        holding = js_str(holding),
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
    /// The elements the page scrolls in, as a selector: the last one shown, not inert and with
    /// room to scroll takes the keys, so a layer over the page scrolls rather than the page under
    /// it. `None` or no match scrolls the document.
    pub scroller: Option<&'static str>,
}

/// The next registration's key among the page's listeners, unique across instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Registers the page keys under `__ID__`: the newest registration answers. It listens on the
/// window after the page's own listeners, at any width, sends `[key, after_g, held, in_overlay]` for a page key under the
/// guard, and remembers a first `g` for `__GG__` milliseconds. Steps answer only while
/// `__oranguiSteps__ID__` is true. With a dialog, sheet or host overlay open, only `g` `g` and
/// Shift+G answer, for that overlay, and the steps too when the topmost one holds `__HINT__`,
/// whatever the screen under it leaves to the page.
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
    if (!__NOT_TYPING__) return;
    const inOverlay = !!document.querySelector(__BLOCKED__);
    const step = key === 'j' || key === 'k';
    const inHint = inOverlay && (() => { const open = [...document.querySelectorAll(__BLOCKED__)]; const top = open[open.length - 1]; return !!top && !!top.querySelector(__HINT__); })();
    if (inOverlay && key !== 'g' && key !== 'G' && !(step && inHint)) return;
    if (step && !inHint && !window['__oranguiSteps__ID__']) return;
    const now = Date.now();
    const afterG = key === 'g' && now - lastG < __GG__;
    lastG = key === 'g' && !afterG ? now : 0;
    e.preventDefault();
    dioxus.send([key, afterG, e.repeat, inOverlay]);
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

/// Scrolls the page by `__BY__`, gliding unless motion is reduced or the key is held: `top` to the
/// top, `bottom` to the bottom, a signed step count that many steps.
const SCROLL_JS: &str = r"
(() => {
    const el = __TARGET__;
    if (!el) return;
    const page = el === document.scrollingElement;
    const still = __HELD__ || matchMedia('(prefers-reduced-motion: reduce)').matches;
    const behavior = still ? 'auto' : 'smooth';
    const by = __BY__;
    const step = Math.max(40, (page ? window.innerHeight : el.clientHeight) * __SHARE__);
    if (by === 'top') { el.scrollTo({ top: 0, behavior }); return; }
    if (by !== 'bottom') { el.scrollBy({ top: by * step, behavior }); return; }
    el.scrollTo({ top: el.scrollHeight - el.clientHeight, behavior });
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
            while let Ok((key, after_g, held, in_overlay)) =
                pressed.recv::<(String, bool, bool, bool)>().await
            {
                let target = if in_overlay {
                    Target::Overlay(keys.blocked_by)
                } else {
                    Target::Page(keys.scroller)
                };
                let Some(action) = page_key(&key, after_g) else {
                    continue;
                };
                match action {
                    PageKey::Top => scroll(target, "'top'", held),
                    PageKey::Bottom => scroll(target, "'bottom'", held),
                    PageKey::StepDown => scroll(target, "1", held),
                    PageKey::StepUp => scroll(target, "-1", held),
                    PageKey::Themes => {
                        if keys.themes {
                            document::eval(&open_themes_js());
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
        .replace("__NOT_TYPING__", NOT_TYPING)
        .replace("__BLOCKED__", &js_str(&blocked))
        .replace("__HINT__", &js_str(HINT_PARTS))
        .replace("__GG__", &GG_WINDOW_MS.to_string())
}

/// Scrolls the page `by` (a script value: `'top'`, `'bottom'` or a signed step count).
fn scroll(target: Target, by: &str, held: bool) {
    document::eval(&scroll_js(target, by, held));
}

/// What a page key scrolls.
#[derive(Debug, Clone, Copy)]
enum Target {
    /// The page: the last shown, live and scrollable element a selector names (a layer over the
    /// page comes after it), else the document.
    Page(Option<&'static str>),
    /// The topmost open dialog, sheet or host overlay (the extra selectors): its most scrollable
    /// part.
    Overlay(&'static [&'static str]),
}

/// A script expression for the element `target` scrolls, `null` when nothing scrolls.
fn target_js(target: Target) -> String {
    match target {
        Target::Page(scroller) => {
            let sel = scroller.map_or_else(|| "null".to_owned(), js_str);
            format!(
                "(() => {{ const sel = {sel}; const found = sel && [...document.querySelectorAll(sel)].filter((el) => el.getClientRects().length > 0 && !el.closest('[inert]') && el.scrollHeight > el.clientHeight).pop(); return found || document.scrollingElement; }})()"
            )
        }
        Target::Overlay(blocked_by) => {
            let open = std::iter::once(OVERLAYS)
                .chain(blocked_by.iter().copied())
                .collect::<Vec<_>>()
                .join(", ");
            let open = js_str(&open);
            format!(
                "(() => {{ const open = [...document.querySelectorAll({open})]; const top = open[open.length - 1]; if (!top) return null; const room = (el) => {{ const o = getComputedStyle(el).overflowY; return (o === 'auto' || o === 'scroll') ? el.scrollHeight - el.clientHeight : 0; }}; return [top, ...top.querySelectorAll('*')].filter((el) => room(el) > 1).sort((a, b) => room(b) - room(a))[0] || null; }})()"
            )
        }
    }
}

/// [`SCROLL_JS`] for `target`, `by` and whether the key is `held`.
fn scroll_js(target: Target, by: &str, held: bool) -> String {
    SCROLL_JS
        .replace("__TARGET__", &target_js(target))
        .replace("__HELD__", &held.to_string())
        .replace("__BY__", by)
        .replace("__SHARE__", &STEP_SHARE.to_string())
}

/// The attribute `t` sets on the page's root while it opens the theme sheet and focuses its search
/// field, so the sheet's own opening focus leaves the theme in use alone. `t` clears it once the
/// field has focus, or after a second.
pub(crate) const THEME_SEARCH_MARK: &str = "data-orangui-theme-search";

/// The open theme sheet's search field, which `t` focuses.
const THEME_SEARCH_FIELD: &str = ".bottom-sheet.show:has(.theme-group-label) .theme-search";

/// How long after `t` the search field takes focus, in milliseconds: the sheet's first frame has
/// drawn it by then.
const SEARCH_DELAY_MS: u32 = 60;

/// How long `t` keeps trying to focus the search field, in milliseconds: a sheet refuses focus
/// until it has started to slide in.
const SEARCH_TRY_MS: u32 = 1000;

/// How often `t` tries while the search field has not taken focus, in milliseconds.
const SEARCH_RETRY_MS: u32 = 50;

/// What `t` runs: clicks the first button marked [`THEMES_ACTION`] and focuses the sheet's search
/// field once it draws, marking the page root meanwhile so the sheet leaves the theme in use
/// unfocused. Characters typed before the field has focus are held from the page and the sheet,
/// then go into the field in order as it takes focus; after the second's tries they are dropped.
/// Esc passes through and ends the hold, dropping what it held.
/// A page with no such button is left as it was.
#[must_use]
pub fn open_themes_js() -> String {
    format!(
        "{{ const b = document.querySelector('[data-action=\"{THEMES_ACTION}\"]'); if (b) {{ const root = document.documentElement; root.setAttribute('{THEME_SEARCH_MARK}', ''); let over = false; const typed = []; const hold = (e) => {{ if (e.key === 'Escape') {{ typed.length = 0; done(); return; }} if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {{ e.preventDefault(); e.stopImmediatePropagation(); typed.push(e.key); }} }}; window.addEventListener('keydown', hold, true); const done = () => {{ over = true; window.removeEventListener('keydown', hold, true); root.removeAttribute('{THEME_SEARCH_MARK}'); }}; b.click(); const until = Date.now() + {SEARCH_TRY_MS}; const take = () => {{ if (over) return; const f = document.querySelector('{THEME_SEARCH_FIELD}'); if (f) f.focus({{ preventScroll: true }}); if (f && document.activeElement === f) {{ done(); if (typed.length) {{ const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; set.call(f, f.value + typed.join('')); f.dispatchEvent(new Event('input', {{ bubbles: true }})); }} return; }} if (Date.now() < until) setTimeout(take, {SEARCH_RETRY_MS}); else done(); }}; setTimeout(take, {SEARCH_DELAY_MS}); }} }}"
    )
}

/// The query for a pointer that hovers and is precise (a mouse or a trackpad), at any width: where
/// a keyboard is likely. Without it the input is touch.
pub const POINTER_QUERY: &str = "(hover: hover) and (pointer: fine)";

/// True on a computer's wide layout ([`COMPUTER_QUERY`]), kept current as the window changes.
/// Off the browser it stays false.
#[must_use]
pub fn use_computer() -> Signal<bool> {
    use_media(COMPUTER_QUERY)
}

/// True while the main input is a pointer that hovers and is precise ([`POINTER_QUERY`]), at any
/// width, kept current as it changes. Off the browser it stays false.
#[must_use]
pub fn use_pointer() -> Signal<bool> {
    use_media(POINTER_QUERY)
}

/// True while the window matches `query`, kept current as it changes. Off the browser it stays
/// false.
pub(crate) fn use_media(query: &'static str) -> Signal<bool> {
    let mut matches = use_signal(|| false);
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_hook(move || {
        let script = MEDIA_JS
            .replace("__ID__", &id.to_string())
            .replace("__QUERY__", query);
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

/// Sends whether the window matches `__QUERY__`, then again on each change.
const MEDIA_JS: &str = r"
const query = window.matchMedia('__QUERY__');
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
    fn the_keys_stay_off_while_typing_or_with_an_overlay_open_at_any_width() {
        let js = keys_js(0, &[".big-modal"]);
        assert!(!js.contains("matchMedia"), "any width: {js}");
        assert!(
            js.contains("['INPUT', 'TEXTAREA', 'SELECT'].includes(f.tagName)"),
            "{js}"
        );
        assert!(js.contains("!(e.ctrlKey || e.altKey || e.metaKey)"), "{js}");
        assert!(
            js.contains(r#"document.querySelector(".alert-dialog[data-state=\"open\"], .bottom-sheet.show, .big-modal")"#),
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
            js.contains("if (step && !inHint && !window['__oranguiSteps0']) return;"),
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
    fn scrolling_glides_unless_motion_is_reduced_or_the_key_is_held() {
        let top = scroll_js(Target::Page(None), "'top'", false);
        assert!(top.contains("const sel = null;"), "the document: {top}");
        assert!(
            top.contains(
                "const still = false || matchMedia('(prefers-reduced-motion: reduce)').matches;"
            ),
            "{top}"
        );
        assert!(
            top.contains("const behavior = still ? 'auto' : 'smooth';"),
            "{top}"
        );
        assert!(
            top.contains("if (by === 'top') { el.scrollTo({ top: 0, behavior }); return; }"),
            "{top}"
        );
        let held = scroll_js(Target::Page(Some(".main")), "1", true);
        assert!(held.contains(r#"const sel = ".main";"#), "{held}");
        assert!(
            held.contains("const still = true ||"),
            "a held key moves at once: {held}"
        );
        assert!(held.contains(&format!("* {STEP_SHARE})")), "{held}");
    }

    #[test]
    fn in_an_overlay_only_gg_and_shift_g_answer_and_they_scroll_the_overlay() {
        let js = keys_js(0, &[".big-modal"]);
        assert!(
            js.contains(
                "if (inOverlay && key !== 'g' && key !== 'G' && !(step && inHint)) return;"
            ),
            "{js}"
        );
        let target = target_js(Target::Overlay(&[".big-modal"]));
        assert!(
            target.contains(r#"document.querySelectorAll(".alert-dialog[data-state=\"open\"], .bottom-sheet.show, .big-modal")"#),
            "{target}"
        );
        assert!(
            target.contains("const top = open[open.length - 1];"),
            "the topmost: {target}"
        );
        assert!(
            target.contains(".sort((a, b) => room(b) - room(a))[0]"),
            "its most scrollable part: {target}"
        );
    }

    #[test]
    fn in_a_hint_or_the_shortcuts_sheet_j_and_k_step_it_whatever_the_screen_leaves_to_the_page() {
        let js = keys_js(0, &[".big-modal"]);
        assert!(
            js.contains("const step = key === 'j' || key === 'k';"),
            "{js}"
        );
        assert!(
            js.contains(&format!(
                "const inHint = inOverlay && (() => {{ const open = [...document.querySelectorAll({blocked})]; const top = open[open.length - 1]; return !!top && !!top.querySelector({parts}); }})();",
                blocked = js_str(".alert-dialog[data-state=\"open\"], .bottom-sheet.show, .big-modal"),
                parts = js_str(HINT_PARTS),
            )),
            "the topmost overlay holds a hint's lines: {js}"
        );
        assert_eq!(
            HINT_PARTS, ".hint-body, .hint-line, .hint-bullets, .shortcut-groups",
            "a hint dialog's lines and the shortcuts sheet's groups"
        );
        assert!(
            js.contains("if (step && !inHint && !window['__oranguiSteps0']) return;"),
            "a hint's steps answer even where the screen keeps j and k: {js}"
        );
    }

    #[test]
    fn the_page_scrolls_its_topmost_shown_scroller_not_one_under_a_layer() {
        let target = target_js(Target::Page(Some(".screen")));
        assert!(
            target.contains("[...document.querySelectorAll(sel)].filter((el) => el.getClientRects().length > 0 && !el.closest('[inert]') && el.scrollHeight > el.clientHeight).pop()"),
            "the last shown, live, scrollable match: a layer over the page, not the page under it: {target}"
        );
    }

    #[test]
    fn shift_g_glides_to_the_bottom() {
        let js = scroll_js(Target::Page(None), "'bottom'", false);
        assert!(
            js.contains("el.scrollTo({ top: el.scrollHeight - el.clientHeight, behavior });"),
            "{js}"
        );
    }

    #[test]
    fn t_marks_the_search_then_presses_the_themes_button() {
        assert_eq!(
            open_themes_js(),
            "{ const b = document.querySelector('[data-action=\"themes\"]'); if (b) { const root = document.documentElement; root.setAttribute('data-orangui-theme-search', ''); let over = false; const typed = []; const hold = (e) => { if (e.key === 'Escape') { typed.length = 0; done(); return; } if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) { e.preventDefault(); e.stopImmediatePropagation(); typed.push(e.key); } }; window.addEventListener('keydown', hold, true); const done = () => { over = true; window.removeEventListener('keydown', hold, true); root.removeAttribute('data-orangui-theme-search'); }; b.click(); const until = Date.now() + 1000; const take = () => { if (over) return; const f = document.querySelector('.bottom-sheet.show:has(.theme-group-label) .theme-search'); if (f) f.focus({ preventScroll: true }); if (f && document.activeElement === f) { done(); if (typed.length) { const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; set.call(f, f.value + typed.join('')); f.dispatchEvent(new Event('input', { bubbles: true })); } return; } if (Date.now() < until) setTimeout(take, 50); else done(); }; setTimeout(take, 60); } }"
        );
    }

    #[test]
    fn letters_typed_before_the_search_takes_focus_land_in_it() {
        let js = open_themes_js();
        assert!(
            js.contains("if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) { e.preventDefault(); e.stopImmediatePropagation(); typed.push(e.key); } }; window.addEventListener('keydown', hold, true);"),
            "a letter waits for the field, kept from the page and the sheet: {js}"
        );
        assert!(
            js.contains("if (f && document.activeElement === f) { done(); if (typed.length) { const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; set.call(f, f.value + typed.join('')); f.dispatchEvent(new Event('input', { bubbles: true })); } return; }"),
            "the held letters go into the field in order once it has focus: {js}"
        );
        assert!(
            js.contains("const done = () => { over = true; window.removeEventListener('keydown', hold, true); root.removeAttribute('data-orangui-theme-search'); };"),
            "the hold ends with the mark, focused or timed out: {js}"
        );
        assert!(
            js.contains("const hold = (e) => { if (e.key === 'Escape') { typed.length = 0; done(); return; }"),
            "Esc passes through, drops what was held and ends the hold: {js}"
        );
        assert!(
            js.contains("const take = () => { if (over) return;"),
            "an ended hold stops trying to focus the field: {js}"
        );
    }

    #[test]
    fn the_media_listener_asks_the_query_it_is_given() {
        assert!(
            MEDIA_JS.contains("window.matchMedia('__QUERY__')"),
            "{MEDIA_JS}"
        );
        assert!(COMPUTER_QUERY.starts_with(POINTER_QUERY));
    }

    #[test]
    fn an_open_sheet_s_keys_need_it_on_top() {
        let js = top_sheet_guard_js(".mark");
        assert!(
            js.contains(r#"top.matches('.bottom-sheet.show') && !!top.querySelector(".mark")"#),
            "{js}"
        );
    }

    #[test]
    fn a_host_selector_with_a_quote_stays_one_string() {
        let sel = "[aria-label='x']";
        let quoted = r#""[aria-label='x']""#;
        let page = target_js(Target::Page(Some("[aria-label='x']")));
        assert!(page.contains(&format!("const sel = {quoted};")), "{page}");
        let overlay = target_js(Target::Overlay(&["[aria-label='x']"]));
        assert!(!overlay.contains(&format!("'{sel}'")), "{overlay}");
        assert!(overlay.contains(&format!(", {sel}\"")), "{overlay}");
        let keys = keys_js(0, &["[aria-label='x']"]);
        assert!(keys.contains(&format!(", {sel}\")")), "{keys}");
        let guard = top_sheet_guard_js(sel);
        assert!(
            guard.contains(&format!("top.querySelector({quoted})")),
            "{guard}"
        );
    }
}
