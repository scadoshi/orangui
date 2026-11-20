//! How long a toast stays up, and whether it carries over to the next screen.
//!
//! Three bands rather than a number per call site, so tuning one band moves
//! every toast meant to feel that way, and a call site says which it meant.
//!
//! Shared so every surface agrees: a confirmation should not linger twice as
//! long in one app as another. Nothing but `std::time::Duration` is involved,
//! so this costs consumers no dependency.
//!
//! **If a host's stylesheet drives the toast lifecycle, these are load-bearing.**
//! With the shared `toast.css` the library removes the toast and a band can
//! change here alone. A host whose own sheet animates the whole life keyed on
//! `data-type` has to change there too, or a toast fades out while it is
//! still up, or sits invisible waiting its turn.

use std::time::Duration;

/// Routine confirmations: saved, copied, added.
pub const QUICK: Duration = Duration::from_secs(3);

/// Worth actually reading before it leaves.
pub const NORMAL: Duration = Duration::from_secs(5);

/// Failures, and anything naming a next step.
pub const LONG: Duration = Duration::from_secs(8);

/// The mark at the front of a carried toast's title. A host that tells the toast stack which
/// screen is up has every other toast leave when the screen changes; a carried one lives out its
/// own life on the next screen.
pub const CARRY_MARK: &str = "\u{1f}carry:";

/// The title a toast that carries over to the next screen is pushed to the toast library with. The
/// stack reads the mark before any other, so it sits outside one.
#[must_use]
pub fn carry_title(title: &str) -> String {
    format!("{CARRY_MARK}{title}")
}

/// The title without its carry mark, when a library title carries one.
#[must_use]
pub fn carried(title: &str) -> Option<&str> {
    title.strip_prefix(CARRY_MARK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_rise_from_quick_to_long() {
        assert!(QUICK < NORMAL && NORMAL < LONG);
        assert_eq!(QUICK.as_secs(), 3);
        assert_eq!(LONG.as_secs(), 8);
    }

    #[test]
    fn a_carried_title_gives_back_the_title_it_was_made_from() {
        assert_eq!(
            carried(&carry_title("Copied to clipboard")),
            Some("Copied to clipboard")
        );
        assert_eq!(
            carried("Filter applied"),
            None,
            "a plain title carries nothing"
        );
    }

    #[test]
    fn a_carried_keyed_title_keeps_its_key_under_the_carry_mark() {
        let title = carry_title(&crate::live_toast::live_title("import"));
        let inner = carried(&title).unwrap();
        assert_eq!(crate::live_toast::live_key(inner), Some("import"));
    }
}
