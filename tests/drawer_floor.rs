//! The wide-window drawer stops at a floor the host can raise, such as the
//! top of a footer scrolled into view, and its dock rides inside it.

const COMPONENTS: &str = include_str!("../assets/components.css");

/// The body of the first rule in `css` whose selector is exactly `selector`.
fn body<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let at = css.find(&format!("{selector} {{"))?;
    let rest = css.get(at..)?;
    let open = rest.find('{')?;
    let close = rest.find('}')?;
    rest.get(open.checked_add(1)?..close)
}

#[test]
fn the_drawer_floor_is_the_hosts_to_raise() {
    let drawer = body(COMPONENTS, ".sheet-form-drawer > .bottom-sheet").expect("the drawer rule");
    let bottom = drawer
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("bottom:"))
        .expect("the drawer sets its bottom");
    assert!(
        bottom.contains("var(--sheet-drawer-floor, 0px)"),
        "the drawer's bottom rises with the host's floor: {bottom}"
    );

    let dock =
        body(COMPONENTS, ".sheet-form-drawer > .bottom-sheet .sheet-dock").expect("the dock rule");
    assert!(
        dock.contains("position: absolute"),
        "the dock sits inside the drawer, so it rises with it"
    );
}
