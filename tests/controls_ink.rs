//! The control gate for the kit's sheets, per `context/development/styling.md`.
//!
//! A button, link or other control that draws its text in a tone reads the tone's ink,
//! `var(--ink-<tone>, var(--<tone>))`, so it reads at 4.5:1 in every theme. A disabled control is
//! exempt: it is dimmed on purpose.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: a bad slice or overflow fails it"
)]

/// The tones that carry an `--ink-<tone>` in every theme block.
const TONES: [&str; 14] = [
    "accent-primary",
    "accent-secondary",
    "accent-tertiary",
    "palette-1",
    "palette-2",
    "palette-3",
    "palette-4",
    "palette-5",
    "palette-6",
    "text-muted",
    "color-money",
    "color-success",
    "color-warning",
    "color-error",
];

/// What marks a selector as a control: a button, a link, a menu item, a toggle.
const CONTROLS: [&str; 16] = [
    "btn",
    "button",
    "nav-link",
    "nav-links > li > a",
    "nav-dropdown-item",
    "nav-dropdown-trigger",
    "mode-toggle",
    "nav-cta",
    "nav-icon",
    "pref-row",
    "action-dock",
    ".banner a",
    "banner-dismiss",
    "panel-action",
    "alert-dialog-action",
    "chip",
];

/// Each rule's selector and body, innermost blocks only, comments removed.
fn rules(css: &str) -> Vec<(String, String)> {
    let mut clean = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        clean.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    clean.push_str(rest);

    let mut out = Vec::new();
    let mut mark = 0;
    let mut selector = None;
    for (i, ch) in clean.char_indices() {
        match ch {
            '{' => {
                selector = Some(
                    clean[mark..i]
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                mark = i + 1;
            }
            '}' => {
                if let Some(sel) = selector.take() {
                    out.push((sel, clean[mark..i].to_string()));
                }
                mark = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// Whether `selector` styles a disabled control, as opposed to excluding one with `:not(...)`.
fn disabled(selector: &str) -> bool {
    let mut rest = selector;
    let mut outside = String::new();
    while let Some(start) = rest.find(":not(") {
        outside.push_str(&rest[..start]);
        let inner = &rest[start + ":not(".len()..];
        let mut depth = 1_usize;
        let close = inner.char_indices().find_map(|(i, ch)| {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            (depth == 0).then_some(i)
        });
        rest = close.map_or("", |i| &inner[i + 1..]);
    }
    outside.push_str(rest);
    outside.contains("disabled")
}

/// The tones the `color:` declarations in `body` name bare.
fn bare_tones(body: &str) -> Vec<&'static str> {
    body.split(';')
        .filter_map(|decl| {
            let (key, value) = decl.split_once(':')?;
            (key.trim() == "color").then(|| value.trim())
        })
        .filter_map(|value| {
            TONES
                .into_iter()
                .find(|tone| value == format!("var(--{tone})"))
        })
        .collect()
}

#[test]
fn controls_draw_tone_text_in_its_ink() {
    let sheets = [
        ("components.css", include_str!("../assets/components.css")),
        ("app.css", include_str!("../assets/app.css")),
        ("site.css", include_str!("../assets/site.css")),
    ];
    let mut offenders = Vec::new();
    for (name, css) in sheets {
        for (selector, body) in rules(css) {
            let control = CONTROLS.iter().any(|marker| selector.contains(marker));
            if !control || disabled(&selector) {
                continue;
            }
            for tone in bare_tones(&body) {
                offenders.push(format!("{name}: {selector} reads --{tone}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "controls that draw text in a bare tone (read var(--ink-<tone>, var(--<tone>))):\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_gate_sees_a_bare_tone_and_passes_an_ink() {
    let css = "/* c */ .x-btn { color: var(--palette-2); }\n@media (hover: hover) { .y-btn:hover { color: var(--ink-palette-1, var(--palette-1)); border-color: var(--palette-1); } }";
    let found: Vec<_> = rules(css)
        .into_iter()
        .flat_map(|(selector, body)| {
            bare_tones(&body)
                .into_iter()
                .map(move |tone| (selector.clone(), tone))
        })
        .collect();
    assert_eq!(found, [(".x-btn".to_string(), "palette-2")]);
    assert!(disabled(".btn:disabled"));
    assert!(!disabled(".btn:not(:disabled, .chip:not(.disabled)):hover"));
}
