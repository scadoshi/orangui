//! A disabled chip fades on every theme, light ones included. A chip has the
//! panel's own fill, so muted ink alone leaves it looking like an enabled one.

const COMPONENTS: &str = include_str!("../assets/components.css");

/// Every rule in `css` as its selector and body, comments left in.
fn rules(css: &str) -> impl Iterator<Item = (&str, &str)> {
    css.split('}').filter_map(|chunk| {
        let (selector, body) = chunk.split_once('{')?;
        let selector = selector.rsplit("*/").next().unwrap_or(selector);
        Some((selector.trim(), body))
    })
}

#[test]
fn a_disabled_chip_fades() {
    let (_, body) = rules(COMPONENTS)
        .find(|(selector, _)| *selector == ".chip.disabled")
        .expect("the disabled chip rule");
    assert!(body.contains("opacity: 0.45"), "the chip fades: {body}");
}

#[test]
fn no_light_theme_rule_undoes_the_disabled_chip_fade() {
    let undone: Vec<&str> = rules(COMPONENTS)
        .filter(|(selector, body)| {
            selector.contains("-light")
                && selector.contains(".chip.disabled")
                && body.contains("opacity")
        })
        .map(|(selector, _)| selector)
        .collect();
    assert!(undone.is_empty(), "light themes keep the fade: {undone:?}");
}
