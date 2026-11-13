//! Focus for a sheet that takes the keyboard: as it opens, the element a selector names inside it
//! takes focus once the sheet has drawn, and while it is the topmost overlay Tab and Shift+Tab go
//! round its own controls; as it closes, focus goes back to where it was, or, with that gone,
//! leaves the closed sheet so it never sits on a hidden control.

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
            format!(
                "{TRAP_JS} window.__oranguiTabTrap.add('{target}'); {}",
                focus_js(id, target, on_focus)
            )
        } else {
            format!(
                "{TRAP_JS} window.__oranguiTabTrap.delete('{target}'); {}",
                restore_js(id)
            )
        };
        document::eval(&script);
    });
    use_drop(move || {
        if *was_open.peek() {
            document::eval(&format!(
                "{TRAP_JS} window.__oranguiTabTrap.delete('{target}'); {}",
                restore_js(id)
            ));
        }
    });
}

/// Installs, once a page, the Tab trap: while the topmost open dialog or sheet holds an element
/// one of the open sheets' targets names (`window.__oranguiTabTrap`), Tab and Shift+Tab go round
/// that overlay's visible controls instead of leaving it, and Esc presses its close button when
/// nothing else took it (a host with no overlay back stack).
const TRAP_JS: &str = r#"if (!window.__oranguiTabTrap) { window.__oranguiTabTrap = new Set(); const trapped = () => { const open = [...document.querySelectorAll('.alert-dialog[data-state="open"], .bottom-sheet.show')]; const top = open[open.length - 1]; return top && [...window.__oranguiTabTrap].some((sel) => { const t = document.querySelector(sel); return !!t && top.contains(t); }) ? top : null; }; window.addEventListener('keydown', (e) => { if (e.key !== 'Escape' || e.defaultPrevented) return; const top = trapped(); const close = top && top.querySelector('.sheet-close'); if (close) { e.preventDefault(); close.click(); } }); document.addEventListener('keydown', (e) => { if (e.key !== 'Tab' || e.ctrlKey || e.altKey || e.metaKey) return; const top = trapped(); if (!top) return; const items = [...top.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])')].filter((el) => el.offsetParent !== null); if (!items.length) return; e.preventDefault(); const at = items.indexOf(document.activeElement); const next = at < 0 ? (e.shiftKey ? items[items.length - 1] : items[0]) : items[(at + (e.shiftKey ? items.length - 1 : 1)) % items.length]; next.focus(); }, true); }"#;

/// Remembers what has focus under `id` and focuses `target` once the sheet has drawn, trying again
/// while focus is on the page or where it was, then runs `on_focus`.
fn focus_js(id: u64, target: &str, on_focus: &str) -> String {
    format!(
        "{{ const store = window.__oranguiSheetFocus = window.__oranguiSheetFocus || {{}}; const was = document.activeElement; store['{id}'] = was; store['{id}s'] = null; const until = Date.now() + {FOCUS_TRY_MS}; const take = () => {{ const t = document.querySelector('{target}'); const a = document.activeElement; if (t) store['{id}s'] = t.closest('.bottom-sheet') || t; if (t && a === t) {{ {on_focus} return; }} if (t && (!a || a === document.body || a === was)) t.focus({{ preventScroll: true }}); if (t && document.activeElement === t) {{ {on_focus} return; }} if (Date.now() < until) setTimeout(take, {FOCUS_RETRY_MS}); }}; setTimeout(take, {FOCUS_DELAY_MS}); }}"
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
    fn tab_goes_round_the_topmost_sheet_s_own_controls() {
        assert!(TRAP_JS.contains("if (e.key !== 'Tab'"), "{TRAP_JS}");
        assert!(
            TRAP_JS.contains("const top = open[open.length - 1];"),
            "only the topmost overlay: {TRAP_JS}"
        );
        assert!(
            TRAP_JS.contains("top.contains(t)"),
            "only a sheet that asked for it: {TRAP_JS}"
        );
        assert!(TRAP_JS.contains("e.preventDefault();"), "{TRAP_JS}");
        assert!(
            TRAP_JS.contains("window.addEventListener('keydown', (e) => { if (e.key !== 'Escape' || e.defaultPrevented) return;"),
            "Esc closes it last, where no back stack did: {TRAP_JS}"
        );
        assert!(
            TRAP_JS.contains("items[(at + (e.shiftKey ? items.length - 1 : 1)) % items.length]"),
            "wraps both ways: {TRAP_JS}"
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
