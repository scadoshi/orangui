//! The backdrop as shaded cells: every square of the grid in its own shade,
//! with no lines drawn, on a fixed layer under everything on screen. The
//! layer paints the sink; the cells sit in a field over it that is brightest
//! at the top and fades out down the viewport. They rain in row by row as
//! soon as the page paints and then hold; a third of them breathe a few
//! percent over half a minute. The shades are seeded, so it is the same field
//! on every load. Styled by `assets/components.css`.
//!
//! A host mounts one under its shell, keyed on the theme class so a theme or
//! mode change rains the field in again:
//!
//! ```ignore
//! GridCells { key: "{css_class}" }
//! ```
//!
//! An app host also drops its screen's fill (`assets/app.css` clears the
//! screen column) so the layer shows through. [`crate::BottomSheet`] carries
//! its own still copy under its header, content and action bar.

use dioxus::prelude::*;

/// The virtual grid, in cells: wide and tall enough to cover a viewport at
/// the stylesheet's cell size. The layer clips whatever falls outside.
const COLUMNS: u32 = 80;
const ROWS: u32 = 44;
/// How far apart the rows arrive, so the field rains down from the top.
const RAIN_STEP_MS: u32 = 18;

struct Cell {
    column: u32,
    row: u32,
    shade: u32,
    rain_ms: u32,
    breathe_ms: Option<u32>,
}

/// The field, from a small linear congruential generator with a fixed
/// seed: each cell's shade, when it lands, and whether it breathes.
fn cells() -> Vec<Cell> {
    let mut state: u32 = 0x9e37_79b9;
    let mut next = move || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        state >> 8
    };
    let mut out = Vec::with_capacity((ROWS * COLUMNS) as usize);
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let roll = next();
            out.push(Cell {
                column,
                row,
                shade: roll % 5 + 1,
                rain_ms: row * RAIN_STEP_MS + (roll >> 4) % 160,
                breathe_ms: (roll % 3 == 0).then_some((roll >> 6) % 20_000),
            });
        }
    }
    out
}

/// The backdrop: a fixed layer of shaded cells under everything on screen.
/// `still` is the bottom sheet's copy: already in place, a step dimmer, and
/// it never animates.
#[component]
pub fn GridCells(#[props(default)] still: bool) -> Element {
    rsx! {
        div {
            class: if still { "grid-cells grid-cells-still" } else { "grid-cells" },
            aria_hidden: "true",
            div { class: "grid-cells-field",
                for cell in cells() {
                    if let Some(breathe) = cell.breathe_ms {
                        span {
                            class: "grid-cell grid-cell-{cell.shade} grid-cell-breathes",
                            style: "--col: {cell.column}; --row: {cell.row}; animation-delay: {cell.rain_ms}ms, {breathe}ms",
                        }
                    } else {
                        span {
                            class: "grid-cell grid-cell-{cell.shade}",
                            style: "--col: {cell.column}; --row: {cell.row}; animation-delay: {cell.rain_ms}ms",
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_covers_the_grid_and_is_the_same_every_time() {
        let a = cells();
        let b = cells();
        assert_eq!(a.len(), (ROWS * COLUMNS) as usize);
        assert!(a.iter().all(|c| (1..=5).contains(&c.shade)));
        assert!(
            a.iter()
                .zip(&b)
                .all(|(x, y)| x.shade == y.shade && x.rain_ms == y.rain_ms)
        );
    }

    #[test]
    fn lower_rows_land_later() {
        let field = cells();
        let first = field.iter().find(|c| c.row == 0).map_or(0, |c| c.rain_ms);
        let last = field
            .iter()
            .find(|c| c.row == ROWS - 1)
            .map_or(0, |c| c.rain_ms);
        assert!(last > first);
    }
}
