//! The contribution heatmap's measurements: a cell per day, a column per
//! week, in SVG units, and how long a picked day's readout holds.

/// A cell's side, in SVG units.
pub const CELL: f64 = 11.0;
/// The gap from one cell to the next, in SVG units.
pub const GAP: f64 = 2.0;
/// From one cell's edge to the next cell's.
pub const STEP: f64 = CELL + GAP;
/// Rows in the grid, one per weekday.
pub const ROWS: usize = 7;
/// How far apart the columns arrive in the sweep, in milliseconds.
pub const SWEEP_STEP_MS: usize = 12;
/// How far a peak's halo reaches past its cell on each side, in SVG units.
pub const HALO_REACH: f64 = 2.5;
/// How long a picked cell's tags hold before the resting tags come back, in milliseconds.
pub const PICK_MS: u32 = 4000;

/// How far `n` columns or rows reach, in SVG units: a grid `columns` wide is
/// `span(columns)` past its origin, and `span(ROWS)` tall.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a grid is a few hundred columns, far below 2^53, so the float is exact; std has no From<usize> for f64"
)]
pub fn span(n: usize) -> f64 {
    n as f64 * STEP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_counts_cells_and_gaps() {
        assert!((span(0)).abs() < f64::EPSILON);
        assert!((span(1) - 13.0).abs() < f64::EPSILON);
        assert!((span(ROWS) - 91.0).abs() < f64::EPSILON);
    }
}
