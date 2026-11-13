//! The theme sheet's keyboard cursor on a computer: while the sheet is open, the arrow keys and
//! `h` `j` `k` `l` move focus through the theme rows as drawn, stopping at the edges, and Home and
//! End go to the first and last row. The cursor is the browser's focus on a row's button, so Enter
//! picks the theme as a click does and Esc closes the sheet as it always has. Focus starts on the
//! theme in use as the sheet opens and goes back as it closes.
//!
//! The browser gathers the rows with where each is drawn, and [`orangui_core::grid_cursor`] decides
//! where a move lands. Only the computer layout takes the keys ([`COMPUTER_QUERY`]); elsewhere
//! [`ThemeSheet`](crate::ThemeSheet)'s up and down arrows preview the row above or below.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use orangui_core::grid_cursor::{Cell, MOVE_KEYS, Snapshot, Stop, move_for};

use crate::page_keys::{COMPUTER_QUERY, PRESSED_KEY, top_sheet_guard_js, use_computer};
use crate::sheet_focus::use_sheet_focus;

/// The next cursor's key among the page's listeners, unique across instances.
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// What marks a theme sheet: its group headings.
const THEME_SHEET_MARK: &str = ".theme-group-label";

/// A theme row: [`ThemeSheet`](crate::ThemeSheet)'s and [`ThemePicker`](crate::ThemePicker)'s.
const ROWS: &str = ".pref-row, .nav-dropdown-item";

/// The row for the theme in use.
const CURRENT: &str = ".pref-row.selected, .nav-dropdown-item.active";

/// The open theme sheet's row for the theme in use, which takes focus as the sheet opens.
const CURRENT_ROW: &str =
    ".bottom-sheet.show:has(.theme-group-label) :is(.pref-row.selected, .nav-dropdown-item.active)";

/// Scrolls the row in focus into the sheet's view once the sheet has given it focus.
const REVEAL: &str = "window.__oranguiThemeReveal && window.__oranguiThemeReveal();";

const NAV_JS: &str = r"
window.__oranguiThemeNav = window.__oranguiThemeNav || [];
window.__oranguiThemeNav.push('__ID__');
const openSheet = () => [...document.querySelectorAll('.bottom-sheet.show')].filter((s) => s.querySelector('__MARK__')).pop();
const rowsOf = (sheet) => [...sheet.querySelectorAll('__ROWS__')].filter((r) => !r.disabled);
const scroller = (row, sheet) => {
    for (let el = row.parentElement; el && el !== sheet.parentElement; el = el.parentElement) {
        const o = getComputedStyle(el).overflowY;
        if ((o === 'auto' || o === 'scroll') && el.scrollHeight > el.clientHeight) return el;
    }
    return null;
};
const reveal = (row, sheet, smooth) => {
    const box = scroller(row, sheet);
    if (!box) return;
    const b = box.getBoundingClientRect();
    const r = row.getBoundingClientRect();
    const label = row.previousElementSibling;
    const head = label && label.matches('.theme-group-label') ? label.getBoundingClientRect() : r;
    const pad = 8;
    let top = null;
    if (head.top < b.top + pad) top = Math.max(0, box.scrollTop + head.top - b.top - pad);
    else if (r.bottom > b.bottom - pad) top = box.scrollTop + r.bottom - b.bottom + pad;
    const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (top !== null) box.scrollTo({ top, behavior: smooth && !still ? 'smooth' : 'auto' });
};
window.__oranguiThemeFocus = (at) => {
    const sheet = openSheet();
    const row = sheet && rowsOf(sheet)[at];
    if (!row) return;
    row.focus({ preventScroll: true });
    reveal(row, sheet, true);
};
window.__oranguiThemeReveal = () => {
    const until = Date.now() + 1000;
    const look = () => {
        const sheet = openSheet();
        const row = document.activeElement && document.activeElement.closest && document.activeElement.closest('__ROWS__');
        if (sheet && row && sheet.contains(row)) { reveal(row, sheet, false); return; }
        if (Date.now() < until) setTimeout(look, 50);
    };
    setTimeout(look, 60);
};
const onKey = (e) => {
    const stack = window.__oranguiThemeNav;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    const key = __KEY__;
    if (!__MOVES__.includes(key) || e.shiftKey && key.startsWith('Arrow')) return;
    if (!matchMedia('__POINTER__').matches || !__GUARD__) return;
    const sheet = openSheet();
    if (!sheet) return;
    const rows = rowsOf(sheet);
    if (!rows.length) return;
    const focused = document.activeElement && document.activeElement.closest && document.activeElement.closest('__ROWS__');
    const onRow = !!focused && rows.includes(focused);
    if ((key === 'Home' || key === 'End') && !onRow) return;
    e.preventDefault();
    e.stopPropagation();
    const fromEl = onRow ? focused : sheet.querySelector('__CURRENT__');
    const from = fromEl ? rows.indexOf(fromEl) : -1;
    const box = scroller(rows[0], sheet);
    const view = box ? box.getBoundingClientRect() : { top: 0, bottom: window.innerHeight };
    const cells = rows.map((r, at) => { const b = r.getBoundingClientRect(); return [at, b.left, b.top + b.height / 2, b.bottom > view.top && b.top < view.bottom]; });
    dioxus.send([key, from < 0 ? null : from, cells]);
};
window['__oranguiThemeNav__ID__'] = onKey;
window.addEventListener('keydown', onKey, true);
";

/// Removes one sheet's cursor keys.
const NAV_DROP_JS: &str = r"
const onKey = window['__oranguiThemeNav__ID__'];
if (onKey) window.removeEventListener('keydown', onKey, true);
delete window['__oranguiThemeNav__ID__'];
window.__oranguiThemeNav = (window.__oranguiThemeNav || []).filter((id) => id !== '__ID__');
";

/// One row as the browser sends it: its place, left edge, vertical center and whether it is in
/// the sheet's view.
type SentCell = (usize, f64, f64, bool);

/// Answers the theme sheet's cursor keys, as the module describes, for the sheet `open` shows.
pub(crate) fn use_theme_cursor(open: Signal<bool>) {
    let computer = use_computer();
    use_sheet_focus(open.into(), computer.into(), CURRENT_ROW, REVEAL);
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_hook(move || {
        let script = nav_js(id);
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok((key, from, cells)) =
                keys.recv::<(String, Option<usize>, Vec<SentCell>)>().await
            {
                let Some(step) = move_for(&key) else {
                    continue;
                };
                if let Some(target) = snapshot_of(&cells).step(from, step) {
                    document::eval(&format!("window.__oranguiThemeFocus({target});"));
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&NAV_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// [`NAV_JS`] for the registration `id`.
fn nav_js(id: u64) -> String {
    NAV_JS
        .replace("__ID__", &id.to_string())
        .replace("__MARK__", THEME_SHEET_MARK)
        .replace("__ROWS__", ROWS)
        .replace("__CURRENT__", CURRENT)
        .replace("__KEY__", PRESSED_KEY)
        .replace("__MOVES__", &format!("{MOVE_KEYS:?}"))
        .replace("__POINTER__", COMPUTER_QUERY)
        .replace("__GUARD__", &top_sheet_guard_js(THEME_SHEET_MARK))
}

/// The browser's rows as a [`Snapshot`], in columns by where they are drawn.
fn snapshot_of(cells: &[SentCell]) -> Snapshot<usize> {
    let cells: Vec<Cell<usize>> = cells
        .iter()
        .map(|&(id, left, center_y, in_view)| Cell {
            stop: Stop {
                id,
                center_y,
                in_view,
            },
            left,
        })
        .collect();
    Snapshot::from_cells(&cells)
}

#[cfg(test)]
mod tests {
    use orangui_core::grid_cursor::Move;

    use super::*;

    #[test]
    fn a_list_of_rows_steps_up_and_down_and_stops_at_its_ends() {
        let cells: Vec<SentCell> = vec![
            (0, 16.0, 20.0, true),
            (1, 16.0, 60.0, true),
            (2, 16.4, 100.0, false),
        ];
        let rows = snapshot_of(&cells);
        assert_eq!(rows.step(Some(0), Move::Down), Some(1));
        assert_eq!(rows.step(Some(2), Move::Down), None, "the last row stops");
        assert_eq!(rows.step(Some(0), Move::Up), None, "the first row stops");
        assert_eq!(rows.step(Some(1), Move::Right), None, "one column");
        assert_eq!(rows.step(Some(1), Move::Last), Some(2));
    }

    #[test]
    fn the_keys_reach_only_the_theme_sheet_on_top_on_a_computer() {
        let js = nav_js(0);
        assert!(
            js.contains(&format!("matchMedia('{COMPUTER_QUERY}')")),
            "{js}"
        );
        assert!(js.contains(".querySelector('.theme-group-label')"), "{js}");
        assert!(js.contains("e.shiftKey && key.startsWith('Arrow')"), "{js}");
        assert!(
            js.contains("(key === 'Home' || key === 'End') && !onRow"),
            "{js}"
        );
    }

    #[test]
    fn a_move_never_reaches_the_sheet_s_previewing_arrows() {
        let js = nav_js(0);
        assert!(
            js.contains("window.addEventListener('keydown', onKey, true)"),
            "{js}"
        );
        assert!(js.contains("e.stopPropagation()"), "{js}");
        assert!(
            !js.contains("'Enter'"),
            "Enter stays the focused row's click: {js}"
        );
    }

    #[test]
    fn a_row_that_opens_its_group_shows_the_heading_above_it() {
        assert!(
            NAV_JS.contains(
                "label.matches('.theme-group-label') ? label.getBoundingClientRect() : r"
            ),
            "{NAV_JS}"
        );
        assert!(NAV_JS.contains("if (head.top < b.top + pad)"), "{NAV_JS}");
    }

    #[test]
    fn both_kinds_of_theme_row_are_rows_and_the_one_in_use_takes_focus() {
        assert!(ROWS.contains(".pref-row") && ROWS.contains(".nav-dropdown-item"));
        assert!(
            CURRENT_ROW.contains(".pref-row.selected")
                && CURRENT_ROW.contains(".nav-dropdown-item.active")
        );
    }
}
