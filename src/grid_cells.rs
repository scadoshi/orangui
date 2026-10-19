//! The backdrop as shaded cells: every square of the grid in its own shade,
//! with no lines drawn, on a fixed layer under everything on screen. The
//! layer paints the sink; the cells are drawn on one canvas over it by
//! `assets/grid-cells.js`, brightest at the top and fading out down the
//! viewport. They rain in row by row as soon as the page paints and then
//! hold. The shades are seeded, so it is the same field on every load.
//! Styled by `assets/components.css`.
//!
//! A host mounts one under its shell:
//!
//! ```ignore
//! GridCells {}
//! ```
//!
//! A theme change redraws the field in the new colors without raining it in
//! again. An app host also drops its screen's fill (`assets/app.css` clears
//! the screen column) so the layer shows through. [`crate::BottomSheet`]
//! carries its own still copy under its header, content and action bar.

use dioxus::prelude::*;

/// The drawing, one canvas per layer. Inlined once; it guards itself.
pub const GRID_CELLS_JS: &str = include_str!("../assets/grid-cells.js");

/// The backdrop: a fixed layer of shaded cells under everything on screen.
/// `still` is the copy that is already in place and never moves: the app
/// screens' backdrop, and the bottom sheet's. `delay_ms` holds the rain, for
/// a site whose content fades in over the layer: with the cells landing
/// under a half-transparent panel they read as drawn over it, so the sites
/// wait out their `content-enter` and the cells arrive once the panels are
/// solid.
#[component]
pub fn GridCells(#[props(default)] still: bool, #[props(default)] delay_ms: u32) -> Element {
    rsx! {
        document::Script { {GRID_CELLS_JS} }
        if still {
            div { class: "grid-cells", "data-still": "true", aria_hidden: "true",
                canvas { class: "grid-cells-canvas" }
            }
        } else {
            div { class: "grid-cells", "data-delay": "{delay_ms}", aria_hidden: "true",
                canvas { class: "grid-cells-canvas" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drawing_finds_its_canvases_and_runs_once() {
        assert!(GRID_CELLS_JS.contains(".grid-cells-canvas"));
        assert!(GRID_CELLS_JS.contains("__oranguiGridCells"));
    }
}
