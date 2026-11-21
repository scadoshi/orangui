//! A year of days as one heatmap every host draws the same: a scroller with the weekday names
//! pinned at its left, the months over the grid, the [`HeatGrid`] under them, a chip naming a
//! day, and the [`HeatReadout`] under it all. Wider than a phone, it scrolls sideways with no
//! scroll bar and opens on its newest weeks. The host hands over the cells and what each one
//! says; the look is the `.heat-*` rules in `assets/components.css`.

use std::rc::Rc;

use dioxus::html::geometry::PixelsVector2D;
use dioxus::prelude::*;

use crate::{
    ChartTip, HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit, HeatReadout, HeatScrollParts,
    HeatWeekdays, heat_span, use_heat_pick, use_open_on_newest,
};

/// Room on the grid's left for the weekday names, in SVG units.
pub const HEAT_LEFT: f64 = 24.0;
/// Room over the grid for the month names, in SVG units.
pub const HEAT_TOP: f64 = 14.0;

/// The [`Heatmap`]'s parts as its classes name them, for the open-on-newest script.
pub const HEATMAP_PARTS: HeatScrollParts = HeatScrollParts {
    scroll: ".heat-scroll",
    row: ".heat-plot",
    axis: ".heat-axis",
    month: ".heat-month",
};

/// Where a chip naming a day sits: its text, and its left and top as percentages of the plot.
#[derive(Debug, Clone, PartialEq)]
struct Chip {
    text: String,
    left: f64,
    top: f64,
}

/// A grid `columns` weeks wide, in SVG units, the weekday room included.
fn plot_width(columns: usize) -> f64 {
    HEAT_LEFT + heat_span(columns)
}

/// The grid's height in SVG units, the month room included.
fn plot_height() -> f64 {
    HEAT_TOP + heat_span(HEAT_ROWS)
}

/// The month names over the grid: a `heat-label heat-month` text per month at its first week's
/// column, which the open-on-newest script reads.
#[component]
fn HeatMonths(months: Vec<(usize, String)>) -> Element {
    rsx! {
        for (column, name) in months {
            text {
                key: "m{column}",
                class: "heat-label heat-month",
                x: "{HEAT_LEFT + heat_span(column)}",
                y: "{HEAT_TOP - 4.0}",
                "{name}"
            }
        }
    }
}

/// The heatmap of `cells`, `columns` weeks wide.
///
/// - `months` and `weekdays` pair a column or row with its name.
/// - `chips` and `tags` say what each cell holds, by its place in `cells`: `chips` in the chip a
///   hovering fine pointer or a tap brings up, `tags` in the readout a tap on a touch or narrow
///   screen fills (the stylesheet picks which shows).
/// - `rest` is the readout's resting tags, `label` the grid's accessible name, and `chip_class`
///   the chip's look, a `.tag` by default.
///
/// A tap that lands on no cell clears the chip and the readout's pick.
#[component]
pub fn Heatmap(
    cells: Vec<HeatCell>,
    columns: usize,
    months: Vec<(usize, String)>,
    weekdays: Vec<(usize, String)>,
    chips: Vec<String>,
    tags: Vec<Vec<String>>,
    rest: Vec<String>,
    label: String,
    #[props(default = "tag".to_string())] chip_class: String,
) -> Element {
    let pick = use_heat_pick();
    let mut chip: Signal<Option<Chip>> = use_signal(|| None);
    let width = plot_width(columns);
    let height = plot_height();
    let chips: Rc<[String]> = chips.into();
    let tags: Rc<[Vec<String>]> = tags.into();
    let place = move |hit: HeatHit, text: &str| Chip {
        text: text.to_owned(),
        left: (hit.x + HEAT_CELL / 2.0) / width * 100.0,
        top: hit.y / height * 100.0,
    };
    let enter_chips = Rc::clone(&chips);
    let on_enter = move |hit: HeatHit| {
        if let Some(text) = enter_chips.get(hit.index) {
            chip.set(Some(place(hit, text)));
        }
    };
    let on_tap = move |hit: HeatHit| {
        if let Some(text) = chips.get(hit.index) {
            chip.set(Some(place(hit, text)));
        }
        if let Some(day) = tags.get(hit.index) {
            pick.show(day.clone());
        }
    };
    use_open_on_newest(HEATMAP_PARTS);
    rsx! {
        div {
            class: "chart-scroll scroll-fade-x heat-scroll",
            onmounted: move |event| async move {
                // The open-on-newest script settles a cut month a beat later.
                let _ = event
                    .data()
                    .scroll(PixelsVector2D::new(width, 0.0), ScrollBehavior::Instant)
                    .await;
            },
            div {
                class: "heat-plot",
                style: "width:{width}px;",
                onmouseleave: move |_| chip.set(None),
                // The weekday names again, pinned at the scroller's left over the grid's own, so
                // they stay in view as the weeks scroll under them.
                div {
                    class: "heat-axis",
                    style: "width:{HEAT_LEFT}px;height:{height}px;margin-bottom:-{height}px;",
                    "aria-hidden": "true",
                    svg {
                        width: "{HEAT_LEFT}",
                        height: "{height}",
                        view_box: "0 0 {HEAT_LEFT} {height}",
                        HeatWeekdays { labels: weekdays.clone(), top: HEAT_TOP }
                    }
                }
                svg {
                    class: "heat-grid",
                    style: "width:{width}px;",
                    view_box: "0 0 {width} {height}",
                    role: "img",
                    "aria-label": "{label}",
                    rect {
                        class: "heat-backdrop",
                        x: "{HEAT_LEFT}",
                        y: "0",
                        width: "{width - HEAT_LEFT}",
                        height: "{height}",
                        onclick: move |_| {
                            chip.set(None);
                            pick.clear();
                        },
                    }
                    HeatMonths { months }
                    HeatWeekdays { labels: weekdays, top: HEAT_TOP }
                    HeatGrid {
                        cells,
                        left: HEAT_LEFT,
                        top: HEAT_TOP,
                        on_enter,
                        on_tap,
                    }
                }
                if let Some(Chip { text, left, top }) = chip() {
                    ChartTip { text, left, top, class: "{chip_class} heat-float" }
                }
            }
        }
        HeatReadout { rest, pick }
    }
}

/// The heatmap while its days load: empty space in the shape of a grid `columns` weeks wide, at
/// its width on a phone and scaled with it where a host scales the grid, so nothing under it
/// moves when the grid lands.
#[component]
pub fn HeatGhost(columns: usize) -> Element {
    let width = plot_width(columns);
    let height = plot_height();
    rsx! {
        div { class: "chart-scroll heat-scroll",
            div { class: "heat-plot", style: "width:{width}px;aspect-ratio:{width} / {height};" }
        }
    }
}
