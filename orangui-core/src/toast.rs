//! How long a toast stays up.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_rise_from_quick_to_long() {
        assert!(QUICK < NORMAL && NORMAL < LONG);
        assert_eq!(QUICK.as_secs(), 3);
        assert_eq!(LONG.as_secs(), 8);
    }
}
