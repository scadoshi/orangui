//! A sheet with a dock ends its content with room to scroll the last row
//! clear of the dock, and that room sits inside the scroller, on the content
//! column, so every engine counts it in the scroll height however the
//! content grows.

const COMPONENTS: &str = include_str!("../assets/components.css");

/// The body of the first rule in `css` whose selector is exactly `selector`.
fn body<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let at = css.find(&format!("\n{selector} {{"))?;
    let rest = css.get(at..)?;
    let open = rest.find('{')?;
    let close = rest.find('}')?;
    rest.get(open.checked_add(1)?..close)
}

#[test]
fn the_dock_room_is_inside_the_scroller() {
    let column = body(
        COMPONENTS,
        ".bottom-sheet:has(.sheet-dock) > .sheet-content > .flex-col",
    )
    .expect("the content column's room");
    for part in [
        "5rem",
        "env(safe-area-inset-bottom, 0px)",
        "var(--keyboard-inset, 0px)",
    ] {
        assert!(column.contains(part), "the room counts {part}: {column}");
    }

    let scroller = body(
        COMPONENTS,
        ".bottom-sheet:has(.sheet-dock) > .sheet-content",
    )
    .expect("the scroller's rule");
    assert!(
        scroller.contains("padding-bottom: 0;"),
        "the scroller adds no room of its own: {scroller}"
    );
}
