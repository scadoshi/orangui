//! How a count reads: in full with thousands separators, or compact with a
//! magnitude letter for a box too narrow for every digit.

/// `n` with a comma every three digits: `12345` reads `12,345`.
#[must_use]
pub fn with_separators(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len().saturating_add(digits.len() / 3));
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// `n` to one decimal with a magnitude letter from ten thousand up: `28.0k`,
/// `1.2m`, `3.4b`. Under that, [`with_separators`]. Truncated, not rounded,
/// so a figure never reads past what it is.
#[must_use]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "every divisor is one of the three nonzero units in STEPS, or a literal"
)]
pub fn short(n: u64) -> String {
    const STEPS: [(u64, char); 3] = [(1_000_000_000, 'b'), (1_000_000, 'm'), (1_000, 'k')];
    if n < 10_000 {
        return with_separators(n);
    }
    for (unit, letter) in STEPS {
        if n >= unit {
            // Every unit is a multiple of ten, so this is n * 10 / unit without the overflow.
            let tenths = n / (unit / 10);
            return format!("{}.{}{letter}", tenths / 10, tenths % 10);
        }
    }
    with_separators(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_keeps_small_numbers_whole_and_shortens_the_rest() {
        assert_eq!(short(2_065), "2,065");
        assert_eq!(short(9_999), "9,999");
        assert_eq!(short(10_000), "10.0k");
        assert_eq!(short(28_018), "28.0k");
        assert_eq!(short(123_456), "123.4k");
        assert_eq!(short(244_855), "244.8k");
        assert_eq!(short(999_999), "999.9k");
        assert_eq!(short(1_234_567), "1.2m");
        assert_eq!(short(123_456_789), "123.4m");
        assert_eq!(short(2_500_000_000), "2.5b");
    }

    #[test]
    fn short_reaches_the_largest_counts_without_overflowing() {
        assert_eq!(short(u64::MAX / 10), "1844674407.3b");
        assert_eq!(short(u64::MAX), "18446744073.7b");
    }

    #[test]
    fn separators_every_three_digits() {
        assert_eq!(with_separators(0), "0");
        assert_eq!(with_separators(999), "999");
        assert_eq!(with_separators(1_000), "1,000");
        assert_eq!(with_separators(1_234_567), "1,234,567");
    }
}
