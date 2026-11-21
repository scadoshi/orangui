//! The shared heatmap: one scroller every host draws the same, with its pinned weekdays, months,
//! backdrop and readout, the loading ghost in its shape, and no horizontal scroll bar.

#![expect(
    clippy::string_slice,
    reason = "a test: a missing rule or render fails it"
)]

use dioxus::prelude::*;
use orangui::{
    HEAT_LEFT, HEAT_ROWS, HEAT_TOP, HEATMAP_PARTS, HeatCell, HeatGhost, Heatmap, heat_span,
    open_on_newest_js,
};

const CSS: &str = include_str!("../assets/components.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn cell(column: usize, row: usize, level: u8) -> HeatCell {
    HeatCell {
        column,
        row,
        level,
        peak: false,
        key: format!("{column}-{row}"),
    }
}

fn year() -> Element {
    rsx! {
        Heatmap {
            cells: vec![cell(0, 1, 2), cell(1, 3, 0)],
            columns: 2,
            months: vec![(0, "Jun".to_owned())],
            weekdays: vec![(1, "Mon".to_owned()), (3, "Wed".to_owned())],
            chips: vec!["2 on Jun 2".to_owned(), "0 on Jun 10".to_owned()],
            tags: vec![vec!["2 logged".to_owned()], vec!["0 logged".to_owned()]],
            rest: vec!["2 logged".to_owned(), "1 of 2 days".to_owned()],
            label: "A year of days".to_owned(),
        }
    }
}

/// The body of the first rule in the kit's sheet whose selector line is exactly `selector`.
fn rule(selector: &str) -> &'static str {
    let start = CSS.find(&format!("\n{selector} {{")).unwrap();
    let body = &CSS[start..];
    &body[..body.find('}').unwrap()]
}

#[test]
fn the_heatmap_draws_every_part_under_the_heat_classes() {
    let html = render(year);
    for part in [
        "class=\"chart-scroll scroll-fade-x heat-scroll\"",
        "class=\"heat-plot\"",
        "class=\"heat-axis\"",
        "class=\"heat-backdrop\"",
        "class=\"heat-label heat-month\"",
        "class=\"heat-readout\"",
        "aria-label=\"A year of days\"",
    ] {
        assert!(html.contains(part), "{part} in {html}");
    }
    let width = HEAT_LEFT + heat_span(2);
    assert!(html.contains(&format!("width:{width}px;")), "{html}");
}

#[test]
fn the_months_sit_over_their_first_week() {
    let html = render(year);
    let x = HEAT_LEFT + heat_span(0);
    let y = HEAT_TOP - 4.0;
    assert!(
        html.contains(&format!(
            "class=\"heat-label heat-month\" x=\"{x}\" y=\"{y}\">Jun"
        )),
        "{html}"
    );
}

#[test]
fn the_ghost_holds_the_grids_shape_while_it_loads() {
    fn app() -> Element {
        rsx! {
            HeatGhost { columns: 54 }
        }
    }
    let html = render(app);
    let width = HEAT_LEFT + heat_span(54);
    let height = HEAT_TOP + heat_span(HEAT_ROWS);
    assert!(
        html.contains(&format!(
            "class=\"heat-plot\" style=\"width:{width}px;aspect-ratio:{width} / {height};\""
        )),
        "{html}"
    );
    assert!(
        !html.contains("heat-cell"),
        "no cells while it loads: {html}"
    );
}

#[test]
fn it_opens_on_its_newest_weeks_by_the_heat_classes() {
    let js = open_on_newest_js(HEATMAP_PARTS);
    for part in [
        "querySelectorAll('.heat-scroll')",
        "querySelector('.heat-plot')",
        "querySelector('.heat-axis')",
        "querySelectorAll('.heat-month')",
    ] {
        assert!(js.contains(part), "{part}: {js}");
    }
}

#[test]
fn the_heatmap_scrolls_with_no_scroll_bar() {
    let scroll = rule(".heat-scroll");
    assert!(scroll.contains("scrollbar-width: none;"), "{scroll}");
    assert!(
        rule(".heat-scroll::-webkit-scrollbar").contains("display: none;"),
        "WebKit, the iPhone's engine, hides it too"
    );
}

#[test]
fn the_small_fixes_from_both_hosts_hold() {
    assert!(
        rule(".heat-axis > svg").contains("font-family: inherit;"),
        "the pinned weekdays take the page's font"
    );
    assert!(
        rule(".heat-plot").contains("box-sizing: content-box;"),
        "room added at the far end widens the plot"
    );
    let readout = rule(":where(.heat-scroll + .heat-readout)");
    assert!(
        readout.contains("margin-inline: var(--space-4);")
            && readout.contains("margin-bottom: var(--space-3);"),
        "{readout}"
    );
    assert!(rule(".heat-backdrop").contains("fill: transparent;"));
}
