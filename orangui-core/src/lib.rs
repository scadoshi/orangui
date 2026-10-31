//! orangui-core: the arithmetic orangui's components compute with, for any client.
//!
//! orangui draws its pieces with Dioxus; this crate holds what they compute
//! and nothing that renders, so a terminal client or anything else without
//! Dioxus or a browser draws the same numbers, frames and themes. It depends
//! on `serde` alone and builds for native targets and `wasm32-unknown-unknown`.
//!
//! - [`number`]: thousands separators and the compact `28.0k` form.
//! - [`chart_math`]: smooth chart paths, hover-chip anchors and heatmap peaks.
//! - [`count_up`]: the rolling digits of a figure on its way to its value.
//! - [`decode`]: block-glyph art resolving out of noise in two waves, and leaving the same way.
//! - [`rng`]: the xorshift and LCG generators the motion draws from.
//! - [`grid_rain`]: the backdrop's shaded cells and how they rain in.
//! - [`splash`]: the opening's timeline.
//! - [`heat`]: the heatmap grid's measurements.
//! - [`diagram`]: the diagram boxes' measurements.
//! - [`toast`]: how long a toast stays up.
//! - [`theme`]: the theme list, its picker groups and the stored pick.
//!
//! Every frame function is deterministic: the same inputs give the same
//! output, so a client keeps time itself and asks for frame `n`.

pub mod chart_math;
pub mod count_up;
pub mod decode;
pub mod diagram;
pub mod grid_rain;
pub mod heat;
pub mod number;
pub mod rng;
pub mod splash;
pub mod theme;
pub mod toast;
