//! The hover gate: a hover look applies only where the device hovers. Every `:hover` rule in a
//! kit sheet sits inside `@media (hover: hover)`, so a tap on a touch screen does not leave the
//! button it touched lit until the next tap lands somewhere else.

const SHEETS: [(&str, &str); 7] = [
    ("components.css", include_str!("../assets/components.css")),
    ("screen.css", include_str!("../assets/screen.css")),
    ("app.css", include_str!("../assets/app.css")),
    ("dialog.css", include_str!("../assets/dialog.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("toast.css", include_str!("../assets/toast.css")),
    ("nav_fold.css", include_str!("../assets/nav_fold.css")),
];

/// Hover rules that may stand outside the gate, each with the block it must sit in and why.
const EXEMPT: [(&str, &str, &str); 2] = [
    (
        ".release-card:hover",
        "prefers-reduced-motion",
        "it only cancels the gated lift for reduced motion and lights nothing",
    ),
    (
        ".banner:hover .banner-progress",
        "",
        "it pauses the countdown rather than lighting anything; gating it changes what a tap does on touch",
    ),
];

/// `sheet` with its comments blanked out, line breaks kept so line numbers still match.
fn without_comments(sheet: &str) -> String {
    let mut out = String::with_capacity(sheet.len());
    let mut rest = sheet;
    while let Some(start) = rest.find("/*") {
        let (before, after) = rest.split_at(start);
        out.push_str(before);
        let end = after
            .find("*/")
            .map_or(after.len(), |e| e.saturating_add(2));
        let (comment, tail) = after.split_at(end);
        out.extend(comment.chars().filter(|c| *c == '\n'));
        rest = tail;
    }
    out.push_str(rest);
    out
}

/// Every block's prelude in `sheet` with its line and the preludes of the blocks around it.
fn blocks(sheet: &str) -> Vec<(usize, String, Vec<String>)> {
    let mut found = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut prelude = String::new();
    let mut line = 1usize;
    for ch in without_comments(sheet).chars() {
        match ch {
            '{' => {
                let selector = prelude.split_whitespace().collect::<Vec<_>>().join(" ");
                found.push((line, selector.clone(), open.clone()));
                open.push(selector);
                prelude.clear();
            }
            '}' => {
                open.pop();
                prelude.clear();
            }
            ';' => prelude.clear(),
            '\n' => {
                line = line.saturating_add(1);
                prelude.push(' ');
            }
            other => prelude.push(other),
        }
    }
    found
}

#[test]
fn every_hover_look_waits_for_a_device_that_hovers() {
    for (name, sheet) in SHEETS {
        for (line, selector, around) in blocks(sheet) {
            if !selector.contains(":hover") || selector.starts_with('@') {
                continue;
            }
            let gated = around
                .iter()
                .any(|p| p.starts_with("@media") && p.contains("(hover: hover)"));
            let exempt = EXEMPT.iter().any(|(s, inside, _)| {
                *s == selector && (inside.is_empty() || around.iter().any(|p| p.contains(inside)))
            });
            assert!(
                gated || exempt,
                "{name}:{line}: `{selector}` is outside @media (hover: hover)"
            );
        }
    }
}
