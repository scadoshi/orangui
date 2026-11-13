//! Focus for a sheet. Every open sheet takes the keyboard: as it opens its × (or, with none, the
//! sheet itself) takes focus, and as it closes focus goes back to where it was ([`use_sheet_entry`]).
//! A sheet with a field or a row to start on names it with [`use_sheet_focus`], which takes over
//! from the ×. While focus is in the topmost open sheet, Tab and Shift+Tab go round its own
//! controls and Esc closes it.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// The next sheet's key in the page's focus store, unique across instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// How long after opening the target takes focus, in milliseconds: the sheet's first frame has
/// drawn it by then.
const FOCUS_DELAY_MS: u32 = 60;

/// How long the target keeps trying to take focus, in milliseconds: a sheet refuses focus until it
/// has started to slide in.
const FOCUS_TRY_MS: u32 = 1000;

/// How often the target tries while it has not taken focus, in milliseconds.
const FOCUS_RETRY_MS: u32 = 50;

/// Focuses the element matching `target` (a selector naming something inside the open sheet, such
/// as `.bottom-sheet.show .sheet-close`) as `open` turns true and `when` holds, remembering what
/// had focus, and gives focus back as `open` turns false. `on_focus` is a script that runs once the
/// target has focus, or `""`.
pub fn use_sheet_focus(
    open: ReadSignal<bool>,
    when: ReadSignal<bool>,
    target: &'static str,
    on_focus: &'static str,
) {
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let mut was_open = use_signal(|| false);
    use_effect(move || {
        let now = open() && when();
        if now == *was_open.peek() {
            return;
        }
        was_open.set(now);
        let script = if now {
            format!("{TRAP_JS} {}", focus_js(id, target, on_focus))
        } else {
            restore_js(id)
        };
        document::eval(&script);
    });
    use_drop(move || {
        if *was_open.peek() {
            document::eval(&restore_js(id));
        }
    });
}

/// What takes focus as a sheet opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OpenFocus {
    /// The sheet's × when it has one, else the sheet itself, so the keyboard starts inside it and
    /// no field takes focus to raise a phone's keyboard. Focus goes back as it closes.
    #[default]
    Sheet,
    /// Nothing moves: for a sheet the keyboard stays outside of, such as a card's details beside
    /// a list a cursor walks.
    Kept,
}

/// Takes focus into the open sheet labeled `title` as `open` turns true, unless `focus` is
/// [`OpenFocus::Kept`]: its ×, or the sheet itself with none, remembering what had focus, which
/// gets it back as `open` turns false or the sheet unmounts open. A [`use_sheet_focus`] target in
/// the same sheet takes over from it. A sheet with no title is left alone.
pub fn use_sheet_entry(open: ReadSignal<bool>, title: &str, focus: OpenFocus) {
    let title = title.to_owned();
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let mut was_open = use_signal(|| false);
    use_effect(use_reactive!(|title, focus| {
        let now = open() && focus == OpenFocus::Sheet && !title.is_empty();
        if now == *was_open.peek() {
            return;
        }
        was_open.set(now);
        let script = if now {
            format!("{TRAP_JS} {}", entry_js(id, &title))
        } else {
            restore_js(id)
        };
        document::eval(&script);
    }));
    use_drop(move || {
        if *was_open.peek() {
            document::eval(&restore_js(id));
        }
    });
}

/// Installs, once a page, the sheet keys: while focus is in the topmost open sheet, or on the page
/// with nothing focused, Tab and Shift+Tab go round that sheet's visible controls instead of
/// leaving it, and Esc, when nothing else took it (a host with no overlay back stack), presses its
/// × or, with none, its backdrop. An open dialog keeps its own keys.
const TRAP_JS: &str = r#"if (!window.__oranguiSheetKeys) { window.__oranguiSheetKeys = true; const held = () => { const open = [...document.querySelectorAll('.alert-dialog[data-state="open"], .bottom-sheet.show')]; const top = open[open.length - 1]; const a = document.activeElement; return top && top.classList.contains('bottom-sheet') && (!a || a === document.body || top.contains(a)) ? top : null; }; window.addEventListener('keydown', (e) => { if (e.key !== 'Escape' || e.defaultPrevented) return; const top = held(); if (!top) return; const back = top.previousElementSibling; const close = top.querySelector(':scope > .sheet-header .sheet-close') || (back && back.classList.contains('sheet-backdrop') ? back : null); if (close) { e.preventDefault(); close.click(); } }); document.addEventListener('keydown', (e) => { if (e.key !== 'Tab' || e.ctrlKey || e.altKey || e.metaKey) return; const top = held(); if (!top) return; const items = [...top.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])')].filter((el) => el.offsetParent !== null); if (!items.length) return; e.preventDefault(); const at = items.indexOf(document.activeElement); const next = at < 0 ? (e.shiftKey ? items[items.length - 1] : items[0]) : items[(at + (e.shiftKey ? items.length - 1 : 1)) % items.length]; next.focus(); }, true); }"#;

/// Remembers what has focus under `id` and focuses the open sheet labeled `title` once it has
/// drawn: its ×, or the sheet itself with none, marked as the page's sheet entry so a
/// [`use_sheet_focus`] target takes over from it. It leaves focus already inside the sheet.
fn entry_js(id: u64, title: &str) -> String {
    format!(
        "{{ const store = window.__oranguiSheetFocus = window.__oranguiSheetFocus || {{}}; const was = document.activeElement; store['{id}'] = was; store['{id}s'] = null; const name = {title:?}; const until = Date.now() + {FOCUS_TRY_MS}; const take = () => {{ const s = [...document.querySelectorAll('.bottom-sheet.show')].find((x) => x.getAttribute('aria-label') === name); const t = s && (s.querySelector(':scope > .sheet-header .sheet-close') || s); const a = document.activeElement; if (s) store['{id}s'] = s; if (s && a && s.contains(a)) return; if (t && (!a || a === document.body || a === was)) {{ t.focus({{ preventScroll: true }}); if (document.activeElement === t) {{ window.__oranguiSheetEntry = t; return; }} }} if (Date.now() < until) setTimeout(take, {FOCUS_RETRY_MS}); }}; setTimeout(take, {FOCUS_DELAY_MS}); }}"
    )
}

/// Remembers what has focus under `id` and focuses `target` once the sheet has drawn, trying again
/// while focus is on the page, where it was or on a sheet's entry ([`use_sheet_entry`]), then runs
/// `on_focus`.
fn focus_js(id: u64, target: &str, on_focus: &str) -> String {
    format!(
        "{{ const store = window.__oranguiSheetFocus = window.__oranguiSheetFocus || {{}}; const was = document.activeElement; store['{id}'] = was; store['{id}s'] = null; const until = Date.now() + {FOCUS_TRY_MS}; const take = () => {{ const t = document.querySelector('{target}'); const a = document.activeElement; if (t) store['{id}s'] = t.closest('.bottom-sheet') || t; if (t && a === t) {{ {on_focus} return; }} if (t && (!a || a === document.body || a === was || a === window.__oranguiSheetEntry)) t.focus({{ preventScroll: true }}); if (t && document.activeElement === t) {{ {on_focus} return; }} if (Date.now() < until) setTimeout(take, {FOCUS_RETRY_MS}); }}; setTimeout(take, {FOCUS_DELAY_MS}); }}"
    )
}

/// Gives focus back to what had it under `id` when that is still in the page, else takes it out
/// of the sheet it went into (remembered as it opened, since the sheet may still be sliding away).
fn restore_js(id: u64) -> String {
    format!(
        "{{ const store = window.__oranguiSheetFocus || {{}}; const was = store['{id}']; const sheet = store['{id}s']; delete store['{id}']; delete store['{id}s']; const a = document.activeElement; const inClosed = !!a && !!sheet && sheet.contains(a); if (was && was !== document.body && was.isConnected && !was.closest('[inert]')) {{ if (!a || a === document.body || inClosed) was.focus({{ preventScroll: true }}); }} else if (inClosed) {{ a.blur(); }} }}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_remembers_focus_and_takes_it_once_the_sheet_draws() {
        let js = focus_js(0, ".bottom-sheet.show .x", "done();");
        assert!(
            js.contains("document.querySelector('.bottom-sheet.show .x')"),
            "{js}"
        );
        assert!(js.contains("t.focus({ preventScroll: true })"), "{js}");
        assert!(
            js.contains("a === document.body || a === was"),
            "only from the page or where it was: {js}"
        );
        assert!(
            js.contains(&format!("setTimeout(take, {FOCUS_DELAY_MS})")),
            "{js}"
        );
        assert!(js.contains("done();"), "{js}");
    }

    #[test]
    fn tab_and_esc_act_on_the_topmost_sheet_holding_focus() {
        assert!(TRAP_JS.contains("if (e.key !== 'Tab'"), "{TRAP_JS}");
        assert!(
            TRAP_JS.contains("const top = open[open.length - 1];"),
            "only the topmost overlay: {TRAP_JS}"
        );
        assert!(
            TRAP_JS.contains("top.classList.contains('bottom-sheet') && (!a || a === document.body || top.contains(a))"),
            "a sheet holding focus, never a dialog: {TRAP_JS}"
        );
        assert!(TRAP_JS.contains("e.preventDefault();"), "{TRAP_JS}");
        assert!(
            TRAP_JS.contains("window.addEventListener('keydown', (e) => { if (e.key !== 'Escape' || e.defaultPrevented) return;"),
            "Esc closes it last, where no back stack did: {TRAP_JS}"
        );
        assert!(
            TRAP_JS.contains("top.querySelector(':scope > .sheet-header .sheet-close') || (back && back.classList.contains('sheet-backdrop') ? back : null)"),
            "by its ×, or its backdrop with none: {TRAP_JS}"
        );
        assert!(
            TRAP_JS.contains("items[(at + (e.shiftKey ? items.length - 1 : 1)) % items.length]"),
            "wraps both ways: {TRAP_JS}"
        );
    }

    #[test]
    fn an_opening_sheet_focuses_its_close_or_itself_and_a_named_target_takes_over() {
        let js = entry_js(0, "Filter \"x\"");
        assert!(js.contains(r#"const name = "Filter \"x\"";"#), "{js}");
        assert!(
            js.contains("s.querySelector(':scope > .sheet-header .sheet-close') || s"),
            "{js}"
        );
        assert!(
            js.contains("if (s && a && s.contains(a)) return;"),
            "focus already in stays: {js}"
        );
        assert!(js.contains("window.__oranguiSheetEntry = t;"), "{js}");
        assert!(
            focus_js(0, ".x", "").contains("a === window.__oranguiSheetEntry"),
            "a named target takes over from the entry"
        );
    }

    #[test]
    fn closing_gives_focus_back_or_takes_it_out_of_the_closed_sheet() {
        let js = restore_js(0);
        assert!(js.contains("was.isConnected"), "{js}");
        assert!(js.contains("was.focus({ preventScroll: true })"), "{js}");
        assert!(js.contains("else if (inClosed) { a.blur(); }"), "{js}");
        assert!(
            js.contains("const inClosed = !!a && !!sheet && sheet.contains(a);"),
            "the sheet it focused into, whatever its classes say while it slides away: {js}"
        );
    }
}
