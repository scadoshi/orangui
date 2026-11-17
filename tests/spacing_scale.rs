//! Spacing comes from one scale: `--space-1` to `--space-7`, the steps 0.25, 0.5, 0.75, 1, 1.5, 2
//! and 3rem on a 4px base. A padding, margin or gap names a step by its token. The few lengths off
//! the scale are hairlines (0.15rem and under), `em` padding that follows its own text, and the
//! listed sizes that make room for something rather than space things apart.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: slices the stylesheets after matches it found and steps past them"
)]

const SHEETS: [(&str, &str); 8] = [
    ("themes.css", include_str!("../assets/themes.css")),
    ("components.css", include_str!("../assets/components.css")),
    ("app.css", include_str!("../assets/app.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("toast.css", include_str!("../assets/toast.css")),
    ("dialog.css", include_str!("../assets/dialog.css")),
    ("nav_fold.css", include_str!("../assets/nav_fold.css")),
    ("screen.css", include_str!("../assets/screen.css")),
];

/// The steps, by token.
const STEPS: [(&str, &str); 7] = [
    ("--space-1", "0.25rem"),
    ("--space-2", "0.5rem"),
    ("--space-3", "0.75rem"),
    ("--space-4", "1rem"),
    ("--space-5", "1.5rem"),
    ("--space-6", "2rem"),
    ("--space-7", "3rem"),
];

/// The custom properties that hold a spacing step for a rule to use.
const SPACING_TOKENS: [&str; 10] = [
    "--panel-gap",
    "--dialog-pad-x",
    "--dock-inset",
    "--nav-pad",
    "--nav-row-pad",
    "--nav-row-gap",
    "--nav-list-start",
    "--nav-list-end",
    "--nav-trigger-pad",
    "--dock-gap",
];

/// Lengths off the scale on purpose: (sheet, the declaration's property, the length, why).
const ALLOWED: [(&str, &str, &str, &str); 5] = [
    (
        "components.css",
        "padding",
        "2.4rem",
        "the banner's room for its close button",
    ),
    (
        "components.css",
        "padding-right",
        "4rem",
        "the password field's room for its show button",
    ),
    (
        "components.css",
        "padding-bottom",
        "5rem",
        "a docked sheet's room for the dock",
    ),
    (
        "screen.css",
        "padding-bottom",
        "5rem",
        "a screen's room for its dock",
    ),
    (
        "toast.css",
        "margin-bottom",
        "2.75rem",
        "an older toast's overlap with the newer one under it",
    ),
];

/// Every spacing declaration in `sheet`: its property and its value.
fn declarations(sheet: &'static str) -> Vec<(&'static str, &'static str)> {
    sheet
        .split(';')
        .filter_map(|chunk| {
            let line = chunk
                .rsplit(['{', '}', '\n'])
                .next()
                .unwrap_or_default()
                .trim();
            let (property, value) = line.split_once(':')?;
            let property = property.trim();
            let spacing = ["padding", "margin", "gap", "row-gap", "column-gap"]
                .iter()
                .any(|p| property == *p || property.starts_with(&format!("{p}-")))
                || SPACING_TOKENS.contains(&property);
            spacing.then_some((property, value.trim()))
        })
        .collect()
}

/// Every rem length written out in `value`, without its sign.
fn rem_lengths(value: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = value;
    while let Some(end) = rest.find("rem") {
        let start = rest[..end]
            .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
            .map_or(0, |at| at + 1);
        out.push(&rest[start..end + 3]);
        rest = &rest[end + 3..];
    }
    out
}

#[test]
fn the_scale_is_seven_steps_on_a_4px_base() {
    let (_, components) = SHEETS[1];
    let root = components
        .split_once("\n:root {\n")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body);
    for (token, length) in STEPS {
        assert!(
            root.contains(&format!("{token}: {length};")),
            "{token} is {length}: {root:?}"
        );
    }
}

#[test]
fn every_spacing_length_names_a_step_or_is_listed() {
    for (file, sheet) in SHEETS {
        for (property, value) in declarations(sheet) {
            for length in rem_lengths(value) {
                let rem: f64 = length.trim_end_matches("rem").parse().unwrap_or(f64::NAN);
                let hairline = rem <= 0.15;
                let allowed = ALLOWED
                    .iter()
                    .any(|(f, p, l, _)| *f == file && *p == property && *l == length);
                assert!(
                    hairline || allowed,
                    "{file}: `{property}: {value}` writes {length}; name a step (var(--space-N)) or list it with a reason"
                );
            }
        }
    }
}
