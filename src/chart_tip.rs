//! The chip that names a chart's point, hung just above it. Styled by
//! `assets/components.css`.

use dioxus::prelude::*;

use crate::chart_math;

/// A chip at `left`% and `top`% of the nearest positioned ancestor (the
/// chart's plot wrapper, which the host makes `position: relative`), hanging
/// above the point. Near either side it hangs from its edge instead of its
/// middle, so it never leaves the chart. It fades in on arrival and glides
/// between points, and takes no pointer events, so it never steals the hover
/// from the point under it.
///
/// `class` is the chip's look, a `.tag` by default; pass a tone such as
/// `"tag tag-p1"` or `"stat-chip"`.
#[component]
pub fn ChartTip(
    text: String,
    left: f64,
    top: f64,
    #[props(default = "tag".to_string())] class: String,
) -> Element {
    rsx! {
        span {
            class: "chart-tip {class} {chart_math::tip_anchor(left)}",
            style: "left: {left}%; top: {top}%;",
            "{text}"
        }
    }
}
