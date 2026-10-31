//! The backdrop's field of shaded cells and how it rains in: every square of
//! the grid gets a shade and a landing time from a seeded [`Lcg`], so the
//! field is the same on every load, and each cell drops a few cells' height
//! into place, easing out.
//!
//! orangui draws the field in `grid-cells.js`; this is the same arithmetic
//! for a client that draws its own, and a test in orangui holds the two to
//! the same constants.

use crate::rng::Lcg;

/// How far apart the rows land, in milliseconds.
pub const RAIN_STEP_MS: u32 = 18;
/// The scatter of landing times inside a row, in milliseconds: each cell
/// lands up to this much after its row.
pub const JITTER_MS: u32 = 160;
/// How long one cell takes to fall, in milliseconds.
pub const DROP_MS: u32 = 350;
/// How far a cell falls, in cells.
pub const DROP_CELLS: u32 = 3;
/// The cells' opacities over the theme's text color, lightest first.
pub const SHADES: [f64; 5] = [0.015, 0.03, 0.05, 0.075, 0.1];
/// The seed every field starts from.
pub const SEED: u32 = 0x9e37_79b9;
/// A cell's side when the host sets no `--grid-size`, in CSS pixels.
pub const CELL_PX: u32 = 28;

/// One square of the field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RainCell {
    /// Column, from the left.
    pub column: u32,
    /// Row, from the top.
    pub row: u32,
    /// Opacity once landed, one of [`SHADES`].
    pub shade: f64,
    /// When the cell starts to fall, in milliseconds from the start of the rain.
    pub at_ms: u32,
}

impl RainCell {
    /// How far through its fall the cell is `elapsed_ms` into the rain, from 0
    /// (not yet showing) to 1 (landed).
    #[must_use]
    pub fn progress(&self, elapsed_ms: f64) -> f64 {
        ((elapsed_ms - f64::from(self.at_ms)) / f64::from(DROP_MS)).clamp(0.0, 1.0)
    }

    /// The cell's opacity `elapsed_ms` into the rain: zero before it starts,
    /// [`RainCell::shade`] once landed.
    #[must_use]
    pub fn alpha(&self, elapsed_ms: f64) -> f64 {
        self.shade * ease(self.progress(elapsed_ms))
    }

    /// How far above its place the cell is `elapsed_ms` into the rain, in
    /// cells: [`DROP_CELLS`] as it starts, zero once landed.
    #[must_use]
    pub fn lift(&self, elapsed_ms: f64) -> f64 {
        (1.0 - ease(self.progress(elapsed_ms))) * f64::from(DROP_CELLS)
    }

    /// When the cell has landed, in milliseconds from the start of the rain.
    #[must_use]
    pub fn lands_at_ms(&self) -> u32 {
        self.at_ms.saturating_add(DROP_MS)
    }
}

/// Every cell of a field `columns` wide and `rows` tall, row by row from the
/// top left. The same size always gives the same field.
#[must_use]
pub fn cells(columns: u32, rows: u32) -> Vec<RainCell> {
    let capacity = usize::try_from(u64::from(columns).saturating_mul(u64::from(rows))).unwrap_or(0);
    let mut out = Vec::with_capacity(capacity);
    let mut draws = Lcg::new(SEED);
    for row in 0..rows {
        for column in 0..columns {
            let roll = draws.next_u32();
            let pick = usize::try_from(roll % 5).unwrap_or(0);
            out.push(RainCell {
                column,
                row,
                shade: SHADES.get(pick).copied().unwrap_or(0.0),
                at_ms: row
                    .saturating_mul(RAIN_STEP_MS)
                    .saturating_add((roll >> 4) % JITTER_MS),
            });
        }
    }
    out
}

/// Ease-out cubic: fast at first, settling at the end. `t` runs 0 to 1.
#[must_use]
pub fn ease(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_matches_the_scripts_first_cells() {
        // The first draws of `lcg(0x9e3779b9)`, see rng: shades 4, 1, 0 and
        // jitters 77, 96, 16.
        let field = cells(3, 2);
        assert_eq!(field.len(), 6);
        let first: Vec<(u32, u32, f64, u32)> = field
            .iter()
            .take(3)
            .map(|c| (c.column, c.row, c.shade, c.at_ms))
            .collect();
        assert_eq!(
            first,
            [(0, 0, 0.1, 77), (1, 0, 0.03, 96), (2, 0, 0.015, 16)]
        );
        let second_row = field.get(3).unwrap();
        assert_eq!((second_row.column, second_row.row), (0, 1));
        assert_eq!(second_row.at_ms, RAIN_STEP_MS + 131);
    }

    #[test]
    fn the_same_size_gives_the_same_field() {
        assert_eq!(cells(40, 30), cells(40, 30));
        assert_eq!(cells(0, 10), Vec::new());
    }

    #[test]
    fn every_cell_lands_inside_its_row_window() {
        for cell in cells(50, 40) {
            let row_start = cell.row * RAIN_STEP_MS;
            assert!(cell.at_ms >= row_start && cell.at_ms < row_start + JITTER_MS);
            assert!(SHADES.contains(&cell.shade));
        }
    }

    #[test]
    fn a_cell_falls_in_and_settles() {
        let cell = RainCell {
            column: 0,
            row: 0,
            shade: 0.1,
            at_ms: 100,
        };
        assert!(cell.alpha(50.0).abs() < f64::EPSILON, "not started");
        assert!(
            (cell.lift(100.0) - 3.0).abs() < f64::EPSILON,
            "starts three cells up"
        );
        let mid = cell.alpha(275.0);
        assert!(mid > 0.0 && mid < 0.1);
        assert!((cell.alpha(450.0) - 0.1).abs() < f64::EPSILON, "landed");
        assert!(cell.lift(1_000.0).abs() < f64::EPSILON);
        assert_eq!(cell.lands_at_ms(), 450);
    }

    #[test]
    fn ease_runs_zero_to_one_and_front_loads() {
        assert!(ease(0.0).abs() < f64::EPSILON);
        assert!((ease(1.0) - 1.0).abs() < f64::EPSILON);
        assert!(ease(0.5) > 0.5);
    }
}
