//! The nav brand's ASCII mark has one size for every site that draws it: the kit's `.nav-logo`
//! rule, soft so a host can still tune it.

/// The kit's component sheet.
const COMPONENTS: &str = include_str!("../assets/components.css");

/// The body of the first rule whose selector is exactly `selector`.
fn rule<'a>(selector: &str) -> Option<&'a str> {
    let at = COMPONENTS.find(&format!("\n{selector} {{"))?;
    let body = COMPONENTS.get(at..)?;
    let open = body.find('{')?;
    let close = body.find('}')?;
    body.get(open.saturating_add(1)..close)
}

#[test]
fn the_nav_logo_is_one_soft_size_in_the_logo_color() {
    let soft = rule(":where(.nav-logo)").expect("a soft rule for the mark");
    assert!(soft.contains("font-size: 0.4rem"), "{soft}");
    assert!(soft.contains("color: var(--logo)"), "{soft}");
    let plain = rule(".nav-logo").expect("a rule for the mark's shape");
    for shape in [
        "line-height: 0.9",
        "white-space: pre",
        "display: block",
        "letter-spacing: 0",
    ] {
        assert!(plain.contains(shape), "{shape} in {plain}");
    }
}
