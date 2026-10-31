//! The rolling counter: a number whose digits flicker inside a window that
//! narrows to nothing, and a formatted figure whose digits settle in place.
//!
//! A client draws frame `1` through [`FRAMES`], one every [`TICK_MS`], and
//! then the value itself. orangui's `CountUp` draws [`count_frame`] and its
//! `Figure` draws [`figure_frame`]; while a count has not arrived, `CountUp`
//! draws [`waiting`] each tick instead.

use crate::rng;

/// How long a number takes to reach its value, in milliseconds.
pub const DURATION_MS: u32 = 1000;
/// The time between frames, in milliseconds.
pub const TICK_MS: u32 = 16;
/// Frames in one roll: [`DURATION_MS`] over [`TICK_MS`], rounded up.
pub const FRAMES: u32 = DURATION_MS.div_ceil(TICK_MS);
/// What a count shows before any frame runs: four figure spaces, about the
/// width of four digits.
pub const PLACEHOLDER: &str = "\u{2007}\u{2007}\u{2007}\u{2007}";

/// How far frame `frame` is through the roll, from 0 to 1. Past [`FRAMES`]
/// it stays at 1.
#[must_use]
pub fn fraction(frame: u32) -> f64 {
    (f64::from(frame) / f64::from(FRAMES)).min(1.0)
}

/// A starting seed for the `index`th counter on a screen, so counters side by
/// side do not flicker in step.
#[must_use]
pub fn seed_for(index: u64) -> u64 {
    index.wrapping_mul(2_654_435_761)
}

/// Frame `frame` of a count rolling to `value`. Frame [`FRAMES`] and later
/// are `value`.
#[must_use]
pub fn count_frame(value: u64, frame: u32) -> u64 {
    rolling(value, fraction(frame), u64::from(frame))
}

/// Frame `frame` of a formatted figure rolling to `text`, for the counter
/// seeded `seed`. Frame [`FRAMES`] and later are `text`.
#[must_use]
pub fn figure_frame(text: &str, frame: u32, seed: u64) -> String {
    scrambled(text, fraction(frame), seed.wrapping_add(u64::from(frame)))
}

/// Digits for a count that has not arrived, at `tick` of the wait for the
/// counter seeded `seed`: always four of them, so the width holds still.
#[must_use]
pub fn waiting(seed: u64, tick: u64) -> u64 {
    1_000_u64.saturating_add(rng::noise(seed.wrapping_add(tick)) % 9_000)
}

/// `value` with a random offset that shrinks to nothing as `fraction` reaches
/// 1, held inside the digit count `value` prints at so the row never widens
/// mid-roll. `fraction` is clamped to 0 through 1.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "counts sit far below 2^53 so u64 to f64 is exact, and the roll is clamped at zero and rounded before going back to u64; std has no From between floats and u64"
)]
pub fn rolling(value: u64, fraction: f64, seed: u64) -> u64 {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction >= 1.0 || value == 0 {
        return value;
    }
    // Squared, so the digits are wild early and barely move at the end.
    let spread = (1.0 - fraction).powi(2);
    // -1.0 to 1.0.
    let swing = (rng::noise(seed) % 2001) as f64 / 1000.0 - 1.0;
    let rolled = (value as f64) + (value as f64) * spread * swing;
    (rolled.max(0.0).round() as u64).min(digit_ceiling(value))
}

/// The largest number of the same digit count: 72 gives 99, 3,960 gives 9,999.
fn digit_ceiling(value: u64) -> u64 {
    let mut ceiling = 9;
    while ceiling < value {
        ceiling = ceiling.saturating_mul(10).saturating_add(9);
    }
    ceiling
}

/// `text` with each digit replaced by a random one with a probability that
/// falls to zero as `fraction` reaches 1, squared so the figure is wild early
/// and barely moves at the end. Everything that is not a digit is left alone,
/// so the width never moves.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "every cast is of a value bounded by a modulo (under 1000, or under 10 for the digit), so none loses anything"
)]
pub fn scrambled(text: &str, fraction: f64, seed: u64) -> String {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction >= 1.0 {
        return text.to_string();
    }
    let spread = (1.0 - fraction).powi(2);
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            if !c.is_ascii_digit() {
                return c;
            }
            let roll = rng::noise(seed.wrapping_add((i as u64).wrapping_mul(7919)));
            if (roll % 1000) as f64 / 1000.0 < spread {
                char::from(b'0'.saturating_add((roll / 1000 % 10) as u8))
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roll_is_sixty_three_frames_of_sixteen_milliseconds() {
        assert_eq!(FRAMES, 63);
        assert!(fraction(0).abs() < f64::EPSILON);
        assert!((fraction(FRAMES) - 1.0).abs() < f64::EPSILON);
        assert!((fraction(FRAMES + 10) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scrambled_keeps_everything_but_the_digits_and_lands_on_the_text() {
        assert_eq!(scrambled("12.3k", 1.0, 7), "12.3k");
        for frame in 1..60u64 {
            let out = scrambled(
                "1,234.6/day",
                f64::from(u32::try_from(frame).unwrap()) / 60.0,
                frame,
            );
            assert_eq!(out.len(), "1,234.6/day".len());
            for (a, b) in out.chars().zip("1,234.6/day".chars()) {
                if b.is_ascii_digit() {
                    assert!(a.is_ascii_digit(), "{out}");
                } else {
                    assert_eq!(a, b, "{out}");
                }
            }
        }
        assert_eq!(scrambled("goal met", 0.1, 3), "goal met");
        // Early on, something moves.
        assert_ne!(scrambled("123,456,789", 0.05, 11), "123,456,789");
    }

    #[test]
    fn the_roll_lands_on_its_value_and_stays_the_same_width() {
        assert_eq!(rolling(3_657, 1.0, 7), 3_657, "the end is the value");
        assert_eq!(rolling(3_657, 1.5, 7), 3_657);
        assert_eq!(rolling(0, 0.0, 7), 0, "nothing to roll");
        for frame in 1..200u64 {
            let shown = rolling(
                3_657,
                f64::from(u32::try_from(frame).unwrap()) / 200.0,
                frame,
            );
            assert!(shown <= 9_999, "{shown} keeps four digits");
        }
    }

    #[test]
    fn the_ceiling_is_the_largest_number_of_the_same_width() {
        assert_eq!(digit_ceiling(7), 9);
        assert_eq!(digit_ceiling(72), 99);
        assert_eq!(digit_ceiling(3_960), 9_999);
        assert_eq!(digit_ceiling(0), 9);
    }

    #[test]
    fn count_frames_hold_the_width_and_land_on_the_last_frame() {
        for value in [7, 72, 3_657, 1_234_567] {
            for frame in 1..FRAMES {
                let shown = count_frame(value, frame);
                assert!(shown <= digit_ceiling(value), "{value} at {frame}: {shown}");
            }
            assert_eq!(count_frame(value, FRAMES), value);
            assert_eq!(count_frame(value, FRAMES + 1), value);
        }
        assert_ne!(
            (1..10).map(|f| count_frame(3_657, f)).collect::<Vec<u64>>(),
            [3_657; 9],
            "the early frames move"
        );
    }

    #[test]
    fn count_frames_are_the_same_on_every_run() {
        let a: Vec<u64> = (1..=FRAMES).map(|f| count_frame(48_211, f)).collect();
        let b: Vec<u64> = (1..=FRAMES).map(|f| count_frame(48_211, f)).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn figure_frames_keep_the_shape_and_land_on_the_text() {
        let seed = seed_for(3);
        for frame in 1..FRAMES {
            let out = figure_frame("+1,606.4%", frame, seed);
            assert_eq!(out.chars().count(), 9);
            assert_eq!(out.chars().filter(char::is_ascii_digit).count(), 5, "{out}");
        }
        assert_eq!(figure_frame("+1,606.4%", FRAMES, seed), "+1,606.4%");
    }

    #[test]
    fn neighbors_roll_different_digits() {
        let one: Vec<String> = (1..10)
            .map(|f| figure_frame("8,888", f, seed_for(1)))
            .collect();
        let two: Vec<String> = (1..10)
            .map(|f| figure_frame("8,888", f, seed_for(2)))
            .collect();
        assert_ne!(one, two);
    }

    #[test]
    fn a_wait_is_always_four_digits() {
        for tick in 0..500 {
            let shown = waiting(seed_for(1), tick);
            assert!((1_000..10_000).contains(&shown), "{shown}");
        }
    }
}
