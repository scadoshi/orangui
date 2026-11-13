//! The keys every page answers, as a vim reader expects them: `g` `g` to the top,
//! Shift+G to the bottom, `j` and `k` a step down and up, `t` the theme sheet and Shift+H the
//! shortcuts sheet; and the grouped rows a shortcuts sheet lists for them. orangui's `use_page_keys` runs
//! them in the browser; this module decides what a key means.

/// What a page key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageKey {
    /// `g` `g`: to the top of the page.
    Top,
    /// Shift+G: to the bottom of the page.
    Bottom,
    /// `j`: a step down.
    StepDown,
    /// `k`: a step up.
    StepUp,
    /// `t`: opens the theme sheet.
    Themes,
    /// Shift+H: opens the shortcuts sheet.
    Shortcuts,
}

impl PageKey {
    /// Whether the key scrolls a step, which a host turns on per page: a page whose own keys use
    /// `j` and `k` keeps them.
    #[must_use]
    pub fn is_step(self) -> bool {
        matches!(self, Self::StepDown | Self::StepUp)
    }
}

/// Every key [`page_key`] reads, as a page's key listener names them: letters with their case from
/// Shift alone, so `G` is Shift+G.
pub const PAGE_KEYS: [&str; 6] = ["g", "G", "j", "k", "t", "H"];

/// How long a first `g` waits for the second, in milliseconds.
pub const GG_WINDOW_MS: u32 = 800;

/// The share of the window's height a step scrolls.
pub const STEP_SHARE: f64 = 0.1;

/// The page key `key` stands for, `after_g` saying a first `g` came within [`GG_WINDOW_MS`]. A
/// lone `g` stands for nothing yet: the page remembers it and waits for the second.
#[must_use]
pub fn page_key(key: &str, after_g: bool) -> Option<PageKey> {
    match key {
        "g" if after_g => Some(PageKey::Top),
        "G" => Some(PageKey::Bottom),
        "j" => Some(PageKey::StepDown),
        "k" => Some(PageKey::StepUp),
        "t" => Some(PageKey::Themes),
        "H" => Some(PageKey::Shortcuts),
        _ => None,
    }
}

/// How far a step scrolls in a window `height` pixels tall.
#[must_use]
pub fn step_px(height: f64) -> f64 {
    (height * STEP_SHARE).max(40.0)
}

/// One row of a shortcuts sheet: its keys as the sheet writes them, and what they do.
pub type KeyRow = (&'static [&'static str], &'static str);

/// `g` `g`'s row.
pub const TOP_ROW: KeyRow = (&["gg"], "To the top");

/// Shift+G's row.
pub const BOTTOM_ROW: KeyRow = (&["Shift+G"], "To the bottom");

/// The step keys' rows.
pub const STEP_ROWS: [KeyRow; 2] = [(&["j"], "Scroll down"), (&["k"], "Scroll up")];

/// `t`'s row.
pub const THEMES_ROW: KeyRow = (&["t"], "Themes");

/// The theme sheet's own rows, listed under [`IN_THEMES`]: its cursor moves through the themes,
/// `g` `g` and Shift+G go to the first and last, and Enter picks one.
pub const THEME_SHEET_ROWS: [KeyRow; 4] = [
    (&["↑", "↓", "←", "→"], "Move between themes"),
    (&["h", "j", "k", "l"], "Move between themes"),
    (&["gg", "Shift+G"], "First or last theme"),
    (&["Enter"], "Pick the theme"),
];

/// Shift+H's row.
pub const SHORTCUTS_ROW: KeyRow = (&["Shift+H"], "These shortcuts");

/// Esc's row.
pub const CLOSE_ROW: KeyRow = (&["Esc"], "Close the open sheet");

/// The heading over the keys that move around the page.
pub const NAVIGATION: &str = "Navigation";

/// The heading over the keys that act on a card.
pub const CARD: &str = "Card";

/// The heading over a swipe screen's four directions.
pub const SWIPE: &str = "Swipe";

/// The heading over the screen's own tools and the keys every page has.
pub const PAGE: &str = "Page";

/// The heading over the theme sheet's own keys.
pub const IN_THEMES: &str = "In Themes";

/// A shortcuts sheet's group: a small heading over its rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyGroup {
    /// The heading.
    pub title: &'static str,
    /// The rows under it, in order.
    pub rows: Vec<KeyRow>,
}

impl KeyGroup {
    /// A group titled `title` over `rows`.
    #[must_use]
    pub fn new(title: &'static str, rows: impl IntoIterator<Item = KeyRow>) -> Self {
        Self {
            title,
            rows: rows.into_iter().collect(),
        }
    }
}

/// A site's shortcuts sheet for the page keys, the step rows only where `steps` are on:
/// Navigation (the ends, then the steps), Page (Themes and the sheets) and the theme sheet's own.
#[must_use]
pub fn page_groups(steps: bool) -> Vec<KeyGroup> {
    let step_rows: &[KeyRow] = if steps { &STEP_ROWS } else { &[] };
    vec![
        KeyGroup::new(
            NAVIGATION,
            [TOP_ROW, BOTTOM_ROW]
                .into_iter()
                .chain(step_rows.iter().copied()),
        ),
        KeyGroup::new(PAGE, [THEMES_ROW, SHORTCUTS_ROW, CLOSE_ROW]),
        KeyGroup::new(IN_THEMES, THEME_SHEET_ROWS),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g_waits_for_a_second_g_and_shift_g_goes_to_the_bottom() {
        assert_eq!(page_key("g", false), None, "a lone g waits");
        assert_eq!(page_key("g", true), Some(PageKey::Top));
        assert_eq!(page_key("G", false), Some(PageKey::Bottom));
        assert_eq!(page_key("G", true), Some(PageKey::Bottom));
    }

    #[test]
    fn the_letters_map_to_their_keys_and_others_do_nothing() {
        assert_eq!(page_key("j", false), Some(PageKey::StepDown));
        assert_eq!(page_key("k", false), Some(PageKey::StepUp));
        assert_eq!(page_key("t", false), Some(PageKey::Themes));
        assert_eq!(page_key("H", false), Some(PageKey::Shortcuts));
        for key in ["J", "K", "T", "h", "x", "Enter", " "] {
            assert_eq!(page_key(key, false), None, "{key}");
        }
    }

    #[test]
    fn page_keys_lists_every_key_page_key_reads() {
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            let reads = page_key(&key, true).is_some() || page_key(&key, false).is_some();
            assert_eq!(reads, PAGE_KEYS.contains(&key.as_str()), "{key}");
        }
    }

    #[test]
    fn only_j_and_k_are_steps() {
        assert!(PageKey::StepDown.is_step() && PageKey::StepUp.is_step());
        for key in [
            PageKey::Top,
            PageKey::Bottom,
            PageKey::Themes,
            PageKey::Shortcuts,
        ] {
            assert!(!key.is_step(), "{key:?}");
        }
    }

    #[test]
    fn a_step_is_a_share_of_the_window_and_never_tiny() {
        assert!((step_px(1000.0) - 100.0).abs() < f64::EPSILON);
        assert!((step_px(100.0) - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_site_s_groups_list_the_steps_only_where_they_are_on() {
        let with = page_groups(true);
        let without = page_groups(false);
        let titles: Vec<&str> = with.iter().map(|group| group.title).collect();
        assert_eq!(titles, [NAVIGATION, PAGE, IN_THEMES]);
        let nav = |groups: &[KeyGroup]| groups.first().map_or_else(Vec::new, |g| g.rows.clone());
        assert!(STEP_ROWS.iter().all(|row| nav(&with).contains(row)));
        assert!(STEP_ROWS.iter().all(|row| !nav(&without).contains(row)));
        assert_eq!(nav(&without), vec![TOP_ROW, BOTTOM_ROW]);
        assert!(
            without
                .iter()
                .any(|g| g.title == IN_THEMES && g.rows == THEME_SHEET_ROWS)
        );
    }
}
