//! A release card's header reads the same on every host: its tags sit
//! together at the right end, the group taking the leftover room so two tags
//! read as a pair, and its version keeps the kit's own letter-spacing over a
//! host's `h2` rule.

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
fn the_tags_group_at_the_right_end_of_the_header() {
    let group = body(COMPONENTS, ".release-card-tags").expect("the tags' group rule");
    for part in [
        "display: flex;",
        "margin-left: auto;",
        "gap: var(--space-2);",
    ] {
        assert!(group.contains(part), "the group has {part}: {group}");
    }
    assert!(
        body(COMPONENTS, ".release-card-tag").is_none(),
        "a tag carries no margin of its own, so two never split the row"
    );
}

#[test]
fn the_version_keeps_normal_letter_spacing_over_a_hosts_h2_rule() {
    let version = body(COMPONENTS, ".release-card-version").expect("the version's rule");
    assert!(
        version.contains("letter-spacing: normal;"),
        "the version sets its own letter-spacing: {version}"
    );
}
