//! The bottom sheet's drag to close: how far a drag pulls the sheet, how fast
//! it moves, and whether letting go closes the sheet or springs it back.
//!
//! Distances are CSS pixels and times milliseconds, measured by the caller
//! from the pointer events; nothing here keeps time.

/// The share of the sheet's height a drag must pull it down to close on
/// release.
pub const CLOSE_FRACTION: f64 = 0.25;

/// The downward speed, in pixels per millisecond, at which a release closes
/// the sheet however far it was pulled: a flick.
pub const FLICK_PX_PER_MS: f64 = 0.5;

/// How long a pointer may rest after its last move before its speed counts
/// as zero, so a drag that stops and then lets go is no flick.
pub const FLICK_STALE_MS: f64 = 100.0;

/// How far the sheet sits below its open place for a pointer that went down
/// at `start_y` and is now at `y`. Never negative: a drag up holds the sheet
/// at its open place.
#[must_use]
pub fn offset(start_y: f64, y: f64) -> f64 {
    (y - start_y).max(0.0)
}

/// The pointer's speed over one move, `dy` pixels in `dt_ms` milliseconds,
/// positive downward. Zero when no time passed.
#[must_use]
pub fn velocity(dy: f64, dt_ms: f64) -> f64 {
    if dt_ms > 0.0 { dy / dt_ms } else { 0.0 }
}

/// The speed a release carries: the last move's speed, or zero once the
/// pointer has rested `FLICK_STALE_MS` since that move.
#[must_use]
pub fn release_velocity(last_velocity: f64, since_last_move_ms: f64) -> f64 {
    if since_last_move_ms > FLICK_STALE_MS {
        0.0
    } else {
        last_velocity
    }
}

/// Whether letting go closes the sheet: pulled past `CLOSE_FRACTION` of its
/// `height`, or flicked down at `FLICK_PX_PER_MS` or faster. An unknown
/// (zero) height closes on a flick alone.
#[must_use]
pub fn closes(offset: f64, height: f64, velocity: f64) -> bool {
    velocity >= FLICK_PX_PER_MS || (height > 0.0 && offset >= height * CLOSE_FRACTION)
}

/// How far, in pixels, a pointer on a sheet pulled sideways moves before its way is read: about
/// a touch's slop, under which a browser has not yet started a scroll either.
pub const ARM_SLOP_PX: f64 = 8.0;

/// What a pointer that went down on a sheet pulled sideways is doing.
///
/// The set is closed, so a match need not carry a `_` arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Pulling the sheet: its move runs more along the pull than across it.
    Pull,
    /// Scrolling the sheet's content, or moving against the pull: the sheet stays put.
    Scroll,
}

/// The [`Intent`] of a pointer that has moved `along` pixels the way the sheet closes and
/// `across` pixels the other axis since it went down, or `None` while both stay under
/// [`ARM_SLOP_PX`].
#[must_use]
pub fn intent(along: f64, across: f64) -> Option<Intent> {
    if along.abs() < ARM_SLOP_PX && across.abs() < ARM_SLOP_PX {
        return None;
    }
    Some(if along > across.abs() {
        Intent::Pull
    } else {
        Intent::Scroll
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_armed_pull_waits_for_a_move_past_the_slop_then_reads_its_way() {
        assert_eq!(intent(3.0, 2.0), None, "inside the slop");
        assert_eq!(
            intent(-ARM_SLOP_PX + 1.0, ARM_SLOP_PX - 1.0),
            None,
            "still inside"
        );
        assert_eq!(
            intent(ARM_SLOP_PX, 2.0),
            Some(Intent::Pull),
            "along the pull"
        );
        assert_eq!(
            intent(12.0, 11.0),
            Some(Intent::Pull),
            "more along than across"
        );
        assert_eq!(
            intent(2.0, ARM_SLOP_PX),
            Some(Intent::Scroll),
            "across: a scroll"
        );
        assert_eq!(
            intent(11.0, 12.0),
            Some(Intent::Scroll),
            "more across than along"
        );
        assert_eq!(
            intent(-ARM_SLOP_PX, 0.0),
            Some(Intent::Scroll),
            "against the pull"
        );
        assert_eq!(
            intent(0.0, -20.0),
            Some(Intent::Scroll),
            "across, the other way"
        );
    }

    #[test]
    fn a_drag_up_holds_the_sheet_in_place() {
        assert!(offset(300.0, 250.0).abs() < f64::EPSILON);
        assert!((offset(300.0, 340.0) - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_quarter_of_the_height_closes_and_less_springs_back() {
        assert!(closes(100.0, 400.0, 0.0));
        assert!(!closes(99.0, 400.0, 0.0));
    }

    #[test]
    fn a_flick_closes_from_any_distance() {
        assert!(closes(10.0, 400.0, FLICK_PX_PER_MS));
        assert!(!closes(10.0, 400.0, FLICK_PX_PER_MS / 2.0));
        assert!(closes(10.0, 0.0, 1.0), "unknown height still flicks");
        assert!(!closes(500.0, 0.0, 0.0), "unknown height needs a flick");
    }

    #[test]
    fn a_pause_before_letting_go_is_no_flick() {
        assert!((release_velocity(2.0, 50.0) - 2.0).abs() < f64::EPSILON);
        assert!(release_velocity(2.0, FLICK_STALE_MS + 1.0).abs() < f64::EPSILON);
        assert!(velocity(10.0, 0.0).abs() < f64::EPSILON);
        assert!((velocity(10.0, 20.0) - 0.5).abs() < f64::EPSILON);
    }
}
