//! A full-bleed box (a panel's rule, a dialog's rule, a sheet's sticky bar, a nav's row) runs out
//! through its container's side padding to the border and no further: its negative side margin
//! names the container's padding token, so the two move together. No rule pulls out by a bare
//! spacing step, except the boxes that tuck into a corner rather than span a container.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: walks CSS text by byte offsets it finds itself"
)]

/// Every sheet orangui ships.
const SHEETS: [(&str, &str); 8] = [
    ("app.css", include_str!("../assets/app.css")),
    ("components.css", include_str!("../assets/components.css")),
    ("dialog.css", include_str!("../assets/dialog.css")),
    ("nav_fold.css", include_str!("../assets/nav_fold.css")),
    ("screen.css", include_str!("../assets/screen.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("themes.css", include_str!("../assets/themes.css")),
    ("toast.css", include_str!("../assets/toast.css")),
];

const COMPONENTS: &str = include_str!("../assets/components.css");

/// The containers' side padding tokens a full-bleed box may pull out by.
const PADDING: [&str; 3] = [
    "calc(-1 * var(--panel-pad))",
    "calc(-1 * var(--dialog-pad-x))",
    "calc(-1 * var(--nav-pad))",
];

/// Boxes with a negative side margin that tuck into a corner instead of spanning a container.
const TUCKED: [&str; 2] = [
    // A release card's caret, nudged toward the head's edge.
    ".release-card-head .collapse-caret",
    // The toast's close button, pulled into the toast's corner.
    ".toast-container [role=\"alertdialog\"] button",
];

/// `css` without its comments, the same length so offsets still line up.
fn without_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("*/")
            .map_or(rest.len(), |e| start + e + 2);
        out.extend(
            rest[start..end]
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' }),
        );
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Every style rule in `css` as its selector and its declarations, at-rules looked inside.
fn rules(css: &str) -> Vec<(String, String)> {
    let css = without_comments(css);
    let mut out = Vec::new();
    let mut heads: Vec<String> = Vec::new();
    let mut start = 0;
    for (at, c) in css.char_indices() {
        match c {
            '{' => {
                heads.push(css[start..at].trim().to_owned());
                start = at + 1;
            }
            '}' => {
                if let Some(head) = heads.pop()
                    && !head.starts_with('@')
                {
                    out.push((
                        head.split_whitespace().collect::<Vec<_>>().join(" "),
                        css[start..at].to_owned(),
                    ));
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    out
}

/// The value `body` gives `property`, if it sets it.
fn value(body: &str, property: &str) -> Option<String> {
    body.split(';').find_map(|decl| {
        let (name, value) = decl.split_once(':')?;
        (name.trim() == property).then(|| value.split_whitespace().collect::<Vec<_>>().join(" "))
    })
}

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule(css: &str, selector: &str) -> String {
    rules(css)
        .into_iter()
        .find(|(head, _)| head == selector)
        .map_or_else(|| panic!("a rule for {selector}"), |(_, body)| body)
}

/// The space-separated parts of a shorthand, a parenthesized group kept whole.
fn parts(shorthand: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0_u32;
    let mut part = String::new();
    for c in shorthand.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if c == ' ' && depth == 0 {
            if !part.is_empty() {
                out.push(std::mem::take(&mut part));
            }
        } else {
            part.push(c);
        }
    }
    if !part.is_empty() {
        out.push(part);
    }
    out
}

/// The left and right sides a shorthand gives, by CSS's one-to-four value rule.
fn sides(shorthand: &str) -> Vec<String> {
    let parts = parts(shorthand);
    match parts.as_slice() {
        [all] => vec![all.clone()],
        [_, x] | [_, x, _] => vec![x.clone()],
        [_, right, _, left] => vec![right.clone(), left.clone()],
        _ => Vec::new(),
    }
}

/// Every side margin `body` sets: the shorthand's sides, the longhands and `margin-inline`.
fn side_margins(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(margin) = value(body, "margin") {
        out.extend(sides(&margin));
    }
    if let Some(inline) = value(body, "margin-inline") {
        out.extend(parts(&inline));
    }
    for property in ["margin-left", "margin-right"] {
        out.extend(value(body, property));
    }
    out
}

#[test]
fn a_full_bleed_box_pulls_out_by_its_container_s_padding() {
    let mut off = Vec::new();
    for (file, css) in SHEETS {
        for (selector, body) in rules(css) {
            if TUCKED.contains(&selector.as_str()) {
                continue;
            }
            for side in side_margins(&body) {
                if side.starts_with("calc(-1 *") && !PADDING.contains(&side.as_str()) {
                    off.push(format!("{file}: {selector} {{ margin: {side} }}"));
                }
            }
        }
    }
    assert!(
        off.is_empty(),
        "full-bleed margins off the padding:\n{}",
        off.join("\n")
    );
}

#[test]
fn a_panel_s_rule_runs_to_the_panel_s_border() {
    assert_eq!(
        value(&rule(COMPONENTS, ".panel-card"), "padding").as_deref(),
        Some("var(--panel-pad)")
    );
    assert_eq!(
        value(&rule(COMPONENTS, ".panel-rule"), "margin").as_deref(),
        Some("var(--space-4) calc(-1 * var(--panel-pad))")
    );
}

#[test]
fn the_theme_search_bar_spans_the_sheet_body_s_padding() {
    assert_eq!(
        value(&rule(COMPONENTS, ".sheet-content"), "padding").as_deref(),
        Some("var(--panel-pad)")
    );
    let bar = rule(COMPONENTS, ".theme-search-bar");
    assert_eq!(
        value(&bar, "margin").as_deref(),
        Some("calc(-1 * var(--panel-pad)) calc(-1 * var(--panel-pad)) 0")
    );
    assert_eq!(
        value(&bar, "padding").as_deref(),
        Some("var(--panel-pad) var(--panel-pad) var(--space-3)")
    );
}
