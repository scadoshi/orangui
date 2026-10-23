//! The soft-default gate, per `context/development/styling.md`.
//!
//! A value a host tunes sits in a zero-specificity `:where()` rule, so a host rule of class
//! specificity or higher wins over it whether the host's sheet loads before or after the kit's.
//! The same property in a plain kit rule for the same selector would win at equal specificity
//! whenever the kit's sheet comes last, so no plain rule may set a property a `:where()` rule
//! sets for its selector. `TUNABLE` lists the values hosts tune today, so moving one back into a
//! plain rule fails here too.

const SHEETS: [(&str, &str); 4] = [
    ("components.css", include_str!("../assets/components.css")),
    ("app.css", include_str!("../assets/app.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("toast.css", include_str!("../assets/toast.css")),
];

const TUNABLE: [(&str, &[&str]); 19] = [
    (
        ".label",
        &["color", "font-size", "letter-spacing", "margin-bottom"],
    ),
    (
        ".tag",
        &["border-radius", "padding", "font-size", "letter-spacing"],
    ),
    (".stat-chip", &["padding", "border-radius", "font-size"]),
    (".nav-wrapper nav", &["max-width"]),
    (
        ".nav-brand",
        &["font-weight", "font-size", "letter-spacing"],
    ),
    (".nav-dropdown-content", &["right"]),
    (
        ".content-enter",
        &["animation-duration", "animation-timing-function"],
    ),
    (
        ".content-enter-delayed",
        &["animation-duration", "animation-delay"],
    ),
    (
        ".chip-in",
        &["animation-duration", "animation-timing-function"],
    ),
    (
        ".chart-line",
        &[
            "color",
            "stroke-width",
            "animation-duration",
            "animation-delay",
        ],
    ),
    (".stats-strip .stat", &["padding"]),
    (".stats-strip .stat-num", &["font-size"]),
    (".stats-strip .stat-label", &["font-size", "letter-spacing"]),
    (".info-button", &["cursor"]),
    (".alert-dialog", &["--dialog-pad-x", "padding"]),
    (
        ".alert-dialog-title",
        &["color", "font-size", "letter-spacing"],
    ),
    (".settings-sections", &["width"]),
    (".pref-note", &["font-size", "margin"]),
    (".site-footer", &["margin-top"]),
];

struct Rule {
    soft: bool,
    selectors: Vec<String>,
    props: Vec<String>,
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some((before, after)) = rest.split_once("/*") {
        out.push_str(before);
        rest = after.split_once("*/").map_or("", |(_, tail)| tail);
    }
    out.push_str(rest);
    out
}

/// The byte index of the `}` that closes the `{` at `open`.
fn closing(css: &str, open: usize) -> usize {
    let mut depth = 0;
    for (i, c) in css.char_indices().skip_while(|(i, _)| *i < open) {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    css.len()
}

fn normalize(selector: &str) -> String {
    selector.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every style rule in `css`, descending into `@media` and `@supports` and skipping keyframes.
fn rules(css: &str, out: &mut Vec<Rule>) {
    let mut rest = css;
    while let Some(open) = rest.find('{') {
        let close = closing(rest, open);
        let prelude = rest.get(..open).unwrap_or("").trim();
        let body = rest.get(open + 1..close).unwrap_or("");
        if prelude.starts_with("@media") || prelude.starts_with("@supports") {
            rules(body, out);
        } else if !prelude.starts_with('@') {
            let inner = prelude
                .strip_prefix(":where(")
                .and_then(|s| s.strip_suffix(')'));
            out.push(Rule {
                soft: inner.is_some(),
                selectors: inner.unwrap_or(prelude).split(',').map(normalize).collect(),
                props: body
                    .split(';')
                    .filter_map(|d| d.split_once(':'))
                    .map(|(p, _)| p.trim().to_string())
                    .collect(),
            });
        }
        rest = rest.get(close + 1..).unwrap_or("");
    }
}

fn all_rules() -> Vec<(&'static str, Rule)> {
    let mut out = Vec::new();
    for (name, css) in SHEETS {
        let mut found = Vec::new();
        rules(&strip_comments(css), &mut found);
        out.extend(found.into_iter().map(|r| (name, r)));
    }
    out
}

#[test]
fn tunable_values_sit_in_where_rules() {
    let rules = all_rules();
    let missing: Vec<String> = TUNABLE
        .iter()
        .flat_map(|(selector, props)| props.iter().map(move |p| (*selector, *p)))
        .filter(|(selector, prop)| {
            !rules.iter().any(|(_, r)| {
                r.soft
                    && r.selectors.iter().any(|s| s == selector)
                    && r.props.iter().any(|p| p == prop)
            })
        })
        .map(|(selector, prop)| format!("{selector} {{ {prop} }}"))
        .collect();
    assert!(
        missing.is_empty(),
        "tunable values not set in a :where() rule:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn plain_rules_leave_soft_values_alone() {
    let rules = all_rules();
    let soft: Vec<(&str, &str)> = rules
        .iter()
        .filter(|(_, r)| r.soft)
        .flat_map(|(_, r)| {
            r.selectors
                .iter()
                .flat_map(|s| r.props.iter().map(move |p| (s.as_str(), p.as_str())))
        })
        .collect();
    let mut offenders = Vec::new();
    for (sheet, rule) in rules.iter().filter(|(_, r)| !r.soft) {
        for selector in &rule.selectors {
            for prop in &rule.props {
                if soft.contains(&(selector.as_str(), prop.as_str())) {
                    offenders.push(format!("{sheet}: {selector} {{ {prop} }}"));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "plain rules set a soft default's property (move it into the :where() rule):\n  {}",
        offenders.join("\n  ")
    );
}
