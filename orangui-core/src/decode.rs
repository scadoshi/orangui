//! Block-glyph art that resolves out of noise: a fast wave of noise draws the
//! shape out of blank, left to right, and a slower decode wave follows it,
//! turning the noise into the art. The way out runs the same two waves the
//! other way, art to noise to blank. Under a pointer, a band of noise
//! travels across the settled art instead.
//!
//! The blank columns are spaces, so the art keeps its size the whole time.
//! A pass of `ms` milliseconds is [`frames`]`(ms)` frames, one every
//! [`TICK_MS`]; frame `n` of the way in is [`entry_frame`], of the way out
//! [`exit_frame`], and the art itself follows the last.

use crate::rng::{self, Xorshift};

/// How long the way in takes, in milliseconds, at full pace.
pub const DURATION_MS: u32 = 850;
/// How long the way out takes, in milliseconds, at full pace.
pub const EXIT_MS: u32 = 450;
/// The time between frames, in milliseconds.
pub const TICK_MS: u32 = 40;
/// Columns the decode front runs past the end, so the last ones get their wobble.
pub const OVERRUN: usize = 4;
/// The share of the pass, in percent, the noise wave takes to cover the art.
pub const NOISE_PCT: usize = 45;
/// How far into the pass, in percent, the decode wave starts.
pub const DECODE_DELAY_PCT: usize = 35;
/// How many columns the shimmer's traveling band covers.
pub const SHIMMER_SPAN: usize = 10;
/// How far the shimmer's band moves each frame, in columns.
pub const SHIMMER_STEP: usize = 2;
/// The glyphs a column shows before it resolves.
pub const NOISE_GLYPHS: [char; 5] = ['░', '▒', '▓', '▎', '▍'];

/// Where the two waves stand partway through a pass, in columns from the
/// left. On the way in, art lies behind `decode` and noise between it and
/// `noise`; on the way out, blank lies behind `decode` (the clearing front)
/// and noise between it and `noise`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fronts {
    /// The leading front, the noise wave's edge.
    pub noise: usize,
    /// The trailing front: the decode wave on the way in, the clearing wave on the way out.
    pub decode: usize,
}

/// Frames in a pass of `ms` milliseconds: one per [`TICK_MS`], rounded up, at
/// least one.
#[must_use]
pub fn frames(ms: u64) -> usize {
    usize::try_from(ms.div_ceil(u64::from(TICK_MS)))
        .unwrap_or(1)
        .max(1)
}

/// Where the two waves stand at `frame` of `frames` across art `columns`
/// wide: the noise front covers the art in the first [`NOISE_PCT`] of the
/// pass, and the decode front starts at [`DECODE_DELAY_PCT`] and runs
/// [`OVERRUN`] past the end by the finish.
#[must_use]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "frame is at most frames, a few dozen, and columns is one line of art, so every product stays far below usize::MAX"
)]
pub fn fronts(columns: usize, frame: usize, frames: usize) -> Fronts {
    let pct = frame * 100;
    let noise = ((columns + 1) * pct / (frames * NOISE_PCT).max(1)).min(columns + 1);
    let decode = (columns + OVERRUN) * pct.saturating_sub(frames * DECODE_DELAY_PCT)
        / (frames * (100 - DECODE_DELAY_PCT)).max(1);
    Fronts { noise, decode }
}

/// The widest line in `text`, in characters.
#[must_use]
pub fn columns(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

/// Frame `frame` of `frames` on the way in.
#[must_use]
pub fn entry_frame(text: &str, frame: usize, frames: usize) -> String {
    let fronts = fronts(columns(text), frame, frames);
    decoded(
        text,
        fronts.decode,
        fronts.noise,
        u64::try_from(frame).unwrap_or(0),
    )
}

/// Frame `frame` of `frames` on the way out. [`blank`] follows the last.
#[must_use]
pub fn exit_frame(text: &str, frame: usize, frames: usize) -> String {
    let fronts = fronts(columns(text), frame, frames);
    encoded(
        text,
        fronts.noise,
        fronts.decode,
        u64::try_from(frame).unwrap_or(0),
    )
}

/// Where the shimmer's band moves next from `center` across art `columns`
/// wide: [`SHIMMER_STEP`] on, wrapping once it has passed the end by a band.
#[must_use]
pub fn next_shimmer_center(center: usize, columns: usize) -> usize {
    let lap = columns.saturating_add(SHIMMER_SPAN);
    center
        .saturating_add(SHIMMER_STEP)
        .checked_rem(lap)
        .unwrap_or(0)
}

/// The shimmer with its band ending at `center`.
#[must_use]
pub fn shimmer_frame(text: &str, center: usize) -> String {
    shimmered(text, center, u64::try_from(center).unwrap_or(0))
}

/// `text` with the band of [`SHIMMER_SPAN`] columns ending at `center`
/// swapped for noise drawn from `seed`, and everything else as it is.
#[must_use]
pub fn shimmered(text: &str, center: usize, seed: u64) -> String {
    let mut out = String::with_capacity(text.len());
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    for line in text.lines() {
        for (column, glyph) in line.chars().enumerate() {
            let inside = column <= center && column.saturating_add(SHIMMER_SPAN) >= center;
            if glyph == ' ' || !inside {
                out.push(glyph);
                continue;
            }
            state = rng::noise(state);
            out.push(noise_glyph(state));
        }
        out.push('\n');
    }
    out
}

/// `text` on its way out: spaces behind the `clear` front, noise from there
/// to the `noise` front, and the art past that. The few columns just behind
/// each front wobble, so the edges are ragged rather than ruled. Every line
/// ends in a newline.
#[must_use]
pub fn encoded(text: &str, noise: usize, clear: usize, seed: u64) -> String {
    let mut out = String::with_capacity(text.len());
    let mut draws = Xorshift::new(seed);
    for line in text.lines() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == ' ' {
                out.push(' ');
                continue;
            }
            let reach = column.saturating_add(wobble(&mut draws));
            if reach < clear {
                out.push(' ');
            } else if reach < noise {
                out.push(noise_glyph(draws.next_u64()));
            } else {
                out.push(glyph);
            }
        }
        out.push('\n');
    }
    out
}

/// `text` with every glyph a space, the art before the front reaches it.
#[must_use]
pub fn blank(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\n' { c } else { ' ' })
        .collect()
}

/// `text` as of the two waves: the glyphs behind the decode `front`, noise
/// from there to the `noise` front, and spaces past that. The noise changes
/// with `seed` so it flickers between frames; the few columns just behind
/// each front wobble too, so the edges are ragged rather than ruled. Every
/// line ends in a newline.
#[must_use]
pub fn decoded(text: &str, front: usize, noise: usize, seed: u64) -> String {
    let mut out = String::with_capacity(text.len());
    let mut draws = Xorshift::new(seed);
    for line in text.lines() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == ' ' {
                out.push(' ');
                continue;
            }
            let reach = column.saturating_add(wobble(&mut draws));
            if reach < front {
                out.push(glyph);
            } else if reach >= noise {
                out.push(' ');
            } else {
                out.push(noise_glyph(draws.next_u64()));
            }
        }
        out.push('\n');
    }
    out
}

/// Up to three columns a glyph reaches past its own.
fn wobble(draws: &mut Xorshift) -> usize {
    usize::try_from(draws.next_u64() % 4).unwrap_or(0)
}

/// One of the [`NOISE_GLYPHS`], picked by `draw`.
fn noise_glyph(draw: u64) -> char {
    let pick = usize::try_from(draw % 5).unwrap_or(0);
    NOISE_GLYPHS.get(pick).copied().unwrap_or('░')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_front_past_the_end_is_the_text_and_at_the_start_is_noise() {
        let art = "▎▉▉\n ▉ \n";
        assert_eq!(decoded(art, 100, 100, 1), art);
        let noisy = decoded(art, 0, 100, 1);
        assert_eq!(
            noisy.chars().count(),
            art.chars().count(),
            "same glyph count"
        );
        assert_eq!(noisy.chars().nth(4), Some(' '), "spaces stay spaces");
        assert!(
            noisy
                .chars()
                .all(|c| c == ' ' || c == '\n' || NOISE_GLYPHS.contains(&c))
        );
    }

    #[test]
    fn the_art_builds_from_blank_with_noise_ahead_of_the_decode() {
        let art = "▉▉▉▉▉▉▉▉▉▉\n";
        assert_eq!(blank(art), "          \n");
        for seed in 1..50u64 {
            let out: Vec<char> = decoded(art, 2, 7, seed).chars().collect();
            // Past the noise front the art is not there yet.
            for c in out.iter().skip(7).take(3) {
                assert_eq!(*c, ' ', "{out:?}");
            }
            // Well behind it, only the art or noise.
            for c in out.iter().take(4) {
                assert!(*c == '▉' || NOISE_GLYPHS.contains(c), "{out:?}");
            }
        }
    }

    #[test]
    fn the_way_out_turns_the_art_to_noise_then_clears_it() {
        let art = "▎▉▉▉▉▉▉▉▉▉\n ▉ ▉ ▉ ▉ ▉\n";
        assert_eq!(encoded(art, 0, 0, 1), art, "before the waves, the art");
        assert_eq!(encoded(art, 100, 100, 1), blank(art), "after them, blank");
        for seed in 1..50u64 {
            let out: Vec<char> = encoded(art, 100, 0, seed).chars().collect();
            for (c, a) in out.iter().zip(art.chars()) {
                if a == ' ' || a == '\n' {
                    assert_eq!(*c, a, "spaces stay spaces");
                } else {
                    assert!(NOISE_GLYPHS.contains(c), "{out:?}");
                }
            }
        }
    }

    #[test]
    fn the_noise_wave_leads_and_both_land_by_the_last_frame() {
        let (columns, frames) = (40, 22);
        let mut last = Fronts {
            noise: 0,
            decode: 0,
        };
        for frame in 1..=frames {
            let now = fronts(columns, frame, frames);
            assert!(
                now.noise >= last.noise && now.decode >= last.decode,
                "fronts only move forward"
            );
            if now.decode > 0 && now.decode <= columns {
                assert!(now.noise > now.decode, "noise leads at frame {frame}");
            }
            last = now;
        }
        assert!(last.noise > columns && last.decode >= columns + OVERRUN);
        assert_eq!(fronts(columns, 1, frames).decode, 0, "decode waits");
    }

    #[test]
    fn a_full_pace_entry_is_twenty_two_frames_and_an_exit_twelve() {
        assert_eq!(frames(u64::from(DURATION_MS)), 22);
        assert_eq!(frames(u64::from(EXIT_MS)), 12);
        assert_eq!(frames(0), 1, "never zero frames");
    }

    const ART: &str = "▄▀▀▄ █▀▀█\n█  █ █▄▄▀\n ▀▀  ▀  ▀\n";

    #[test]
    fn entry_frames_go_from_noise_to_the_art() {
        let total = frames(u64::from(DURATION_MS));
        let first = entry_frame(ART, 1, total);
        assert!(
            first
                .chars()
                .all(|c| c == ' ' || c == '\n' || NOISE_GLYPHS.contains(&c)),
            "the first frame has no art yet: {first:?}"
        );
        assert_eq!(
            entry_frame(ART, total, total),
            ART,
            "the last frame is the art"
        );
        for frame in 1..=total {
            let out = entry_frame(ART, frame, total);
            assert_eq!(out.lines().count(), ART.lines().count());
            for (got, want) in out.lines().zip(ART.lines()) {
                assert_eq!(got.chars().count(), want.chars().count(), "frame {frame}");
            }
        }
    }

    #[test]
    fn exit_frames_go_from_the_art_to_blank() {
        let total = frames(u64::from(EXIT_MS));
        assert_eq!(
            exit_frame(ART, total, total),
            blank(ART),
            "the last frame is blank"
        );
        let mid = exit_frame(ART, total / 2, total);
        assert!(mid.chars().any(|c| NOISE_GLYPHS.contains(&c)), "{mid:?}");
    }

    #[test]
    fn frames_are_the_same_on_every_run() {
        let total = frames(u64::from(DURATION_MS));
        for frame in 1..=total {
            assert_eq!(
                entry_frame(ART, frame, total),
                entry_frame(ART, frame, total)
            );
        }
    }

    #[test]
    fn the_shimmer_band_travels_and_wraps() {
        let width = columns(ART);
        assert_eq!(width, 9);
        assert_eq!(next_shimmer_center(0, width), 2);
        assert_eq!(
            next_shimmer_center(18, width),
            1,
            "wraps after a band past the end"
        );
        let out = shimmer_frame(ART, 4);
        for (got, want) in out.chars().zip(ART.chars()) {
            if want == ' ' || want == '\n' {
                assert_eq!(got, want);
            }
        }
        assert_eq!(
            shimmer_frame("▉▉▉\n", 100),
            "▉▉▉\n",
            "a band past the art leaves it"
        );
    }
}
