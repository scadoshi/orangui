//! A heat grid's pinned weekday strip: the weekday names down the grid's left edge, which a
//! host draws in a strip pinned over the scroller's left and again in the grid's own SVG, and
//! the script that opens a grid wider than its box on its newest weeks. The host keeps its own
//! strip, scroller and classes; these are the parts every host draws the same.

use dioxus::prelude::*;

use crate::{HEAT_CELL, heat_span};

/// The weekday names down a heat grid's left edge: a `heat-label` text per labelled row, its
/// baseline on the row's cell, the rows starting `top` below the SVG's top. `labels` pairs each
/// labelled row (0 for the first day of the week) with its name.
#[component]
pub fn HeatWeekdays(labels: Vec<(usize, String)>, top: f64) -> Element {
    rsx! {
        for (row, name) in labels {
            text {
                key: "{row}",
                class: "heat-label",
                x: "0",
                y: "{top + heat_span(row) + HEAT_CELL - 2.0}",
                "{name}"
            }
        }
    }
}

/// The parts of a pinned-strip heat grid the open-on-newest script reads, as CSS selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeatScrollParts {
    /// The scroller the grid sits in.
    pub scroll: &'static str,
    /// The row inside the scroller that takes the room added at the far end.
    pub row: &'static str,
    /// The pinned strip.
    pub axis: &'static str,
    /// A month name over the grid.
    pub month: &'static str,
}

/// The open-on-newest script for `parts` (see [`use_open_on_newest`]). It opens every scroller
/// wider than its box on its newest weeks. A month name the pinned strip cuts through moves on
/// under the strip, with that much room added at the far end, so the newest week stays in view
/// and no name shows by half at the edge. It waits a beat for the box to be laid out.
#[must_use]
pub fn open_on_newest_js(parts: HeatScrollParts) -> String {
    let HeatScrollParts {
        scroll,
        row,
        axis,
        month,
    } = parts;
    format!(
        r"
setTimeout(() => {{
    for (const box of document.querySelectorAll('{scroll}')) {{
        const row = box.querySelector('{row}');
        const axis = box.querySelector('{axis}');
        if (!row || !axis) continue;
        row.style.paddingRight = '';
        box.scrollLeft = box.scrollWidth;
        if (box.scrollWidth <= box.clientWidth) continue;
        const edge = axis.getBoundingClientRect().right;
        const cut = Array.from(box.querySelectorAll('{month}'))
            .map((label) => label.getBoundingClientRect())
            .find((r) => r.left < edge && r.right > edge);
        if (!cut) continue;
        row.style.paddingRight = Math.ceil(cut.right - edge) + 2 + 'px';
        box.scrollLeft = box.scrollWidth;
    }}
}}, 60);
"
    )
}

/// Opens every grid matching `parts` on its newest weeks once the page is up, with a month name
/// the pinned strip would cut moved on under it.
pub fn use_open_on_newest(parts: HeatScrollParts) {
    use_effect(move || {
        document::eval(&open_on_newest_js(parts));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weekdays_draw_on_their_rows() {
        fn app() -> Element {
            rsx! {
                svg {
                    HeatWeekdays {
                        labels: vec![(1, "Mon".to_owned()), (3, "Wed".to_owned())],
                        top: 14.0,
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        assert_eq!(
            dioxus_ssr::render(&dom),
            "<svg><text class=\"heat-label\" x=\"0\" y=\"36\">Mon</text><text class=\"heat-label\" x=\"0\" y=\"62\">Wed</text></svg>"
        );
    }

    #[test]
    fn the_script_is_pinned_for_one_set_of_parts() {
        let js = open_on_newest_js(HeatScrollParts {
            scroll: ".heat-scroll",
            row: ".heat-plot",
            axis: ".heat-axis",
            month: ".heat-month",
        });
        let want = "
setTimeout(() => {
    for (const box of document.querySelectorAll('.heat-scroll')) {
        const row = box.querySelector('.heat-plot');
        const axis = box.querySelector('.heat-axis');
        if (!row || !axis) continue;
        row.style.paddingRight = '';
        box.scrollLeft = box.scrollWidth;
        if (box.scrollWidth <= box.clientWidth) continue;
        const edge = axis.getBoundingClientRect().right;
        const cut = Array.from(box.querySelectorAll('.heat-month'))
            .map((label) => label.getBoundingClientRect())
            .find((r) => r.left < edge && r.right > edge);
        if (!cut) continue;
        row.style.paddingRight = Math.ceil(cut.right - edge) + 2 + 'px';
        box.scrollLeft = box.scrollWidth;
    }
}, 60);
";
        assert_eq!(js, want);
    }

    #[test]
    fn the_script_names_each_part() {
        let js = open_on_newest_js(HeatScrollParts {
            scroll: ".a-scroll",
            row: ".a-row",
            axis: ".a-axis",
            month: ".a-month",
        });
        for want in [
            "document.querySelectorAll('.a-scroll')",
            "box.querySelector('.a-row')",
            "box.querySelector('.a-axis')",
            "box.querySelectorAll('.a-month')",
        ] {
            assert!(js.contains(want), "{want} in {js}");
        }
    }
}
