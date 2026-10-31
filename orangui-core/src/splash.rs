//! The opening's timeline: the backdrop rains in, the host's mark resolves
//! over it and leaves the way it came, and the overlay fades. Every time is
//! in milliseconds from the start at full pace; [`paced`] scales one.

use crate::decode;

/// When the mark mounts, so it resolves as the field is landing.
pub const MARK_AT_MS: u32 = 400;
/// When the mark starts to leave, just after it has resolved.
pub const LEAVE_AT_MS: u32 = 1350;
/// When the fade starts: once the mark has left.
pub const OUT_AT_MS: u32 = LEAVE_AT_MS + decode::EXIT_MS;
/// The fade itself, which keeps its length at any pace.
pub const FADE_MS: u32 = 300;

/// `ms` scaled to `pace`, a percent of the full timeline.
#[must_use]
pub fn paced(ms: u32, pace: u32) -> u32 {
    ms.saturating_mul(pace) / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_steps_run_in_order() {
        const { assert!(MARK_AT_MS < LEAVE_AT_MS && LEAVE_AT_MS < OUT_AT_MS) };
        assert_eq!(OUT_AT_MS, 1800);
    }

    #[test]
    fn pace_scales_a_time_and_never_overflows() {
        assert_eq!(paced(LEAVE_AT_MS, 100), LEAVE_AT_MS);
        assert_eq!(paced(400, 50), 200);
        assert_eq!(paced(u32::MAX, 200), u32::MAX / 100);
    }
}
