//! The generators the motion draws from: enough randomness for digits and
//! glyphs that flicker, seeded so a frame is the same every time it is asked
//! for. None of them is fit for anything that needs real randomness.

/// The golden-ratio multiplier that spreads a small seed across all 64 bits.
const GOLDEN: u64 = 0x9e37_79b9_7f4a_7c15;

/// One xorshift round on a spread `seed`: a single draw, the same for the
/// same seed. The count-up's digits and the decode's shimmer take theirs here.
#[must_use]
pub fn noise(seed: u64) -> u64 {
    let mut state = seed.wrapping_mul(GOLDEN) | 1;
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

/// An endless xorshift stream from one seed, the draws a decode frame takes
/// glyph by glyph. The seed is spread the way [`noise`] spreads it, so the
/// first draw equals `noise(seed)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Xorshift {
    state: u64,
}

impl Xorshift {
    /// A stream starting from `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_mul(GOLDEN) | 1,
        }
    }

    /// The next draw.
    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

impl Iterator for Xorshift {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        Some(self.next_u64())
    }
}

/// The 32-bit linear congruential generator the backdrop's rain draws from,
/// with the constants from Numerical Recipes. Each draw is the top 24 bits of
/// the new state, matching `lcg` in orangui's `grid-cells.js` draw for draw.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lcg {
    state: u32,
}

impl Lcg {
    /// The multiplier.
    pub const MULTIPLIER: u32 = 1_664_525;
    /// The increment.
    pub const INCREMENT: u32 = 1_013_904_223;

    /// A stream starting from `seed`.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// The next draw, under 2^24.
    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT);
        self.state >> 8
    }
}

impl Iterator for Lcg {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        Some(self.next_u32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stream_starts_with_the_one_shot_draw() {
        for seed in [0, 1, 7, 1_000, u64::MAX] {
            assert_eq!(Xorshift::new(seed).next_u64(), noise(seed), "seed {seed}");
        }
    }

    #[test]
    fn a_seed_gives_the_same_draws_every_time() {
        let a: Vec<u64> = Xorshift::new(42).take(8).collect();
        let b: Vec<u64> = Xorshift::new(42).take(8).collect();
        assert_eq!(a, b);
        assert_ne!(a, Xorshift::new(43).take(8).collect::<Vec<u64>>());
    }

    #[test]
    fn the_lcg_matches_the_scripts_draws() {
        // The script's `lcg(0x9e3779b9)`, worked through in 32-bit arithmetic.
        let draws: Vec<u32> = Lcg::new(0x9e37_79b9).take(6).collect();
        assert_eq!(
            draws,
            [
                4_378_839, 15_279_616, 4_167_945, 6_335_539, 13_998_437, 1_443_175
            ]
        );
        assert!(Lcg::new(1).take(1_000).all(|draw| draw < 1 << 24));
    }
}
