//! The syntax gate for `assets/themes.css`.
//!
//! Every theme block carries eleven `--syntax-*` slots, the colors its editor
//! scheme gives code, so a host's code blocks look like that theme in an
//! editor. A theme with no editor scheme of its own maps them from its palette.
//! Each value is a hex or one of the block's own palette or text variables.

const CSS: &str = include_str!("../assets/themes.css");

const SLOTS: [&str; 11] = [
    "keyword",
    "type",
    "function",
    "string",
    "constant",
    "comment",
    "macro",
    "attribute",
    "punctuation",
    "parameter",
    "member",
];

/// Each theme block's name and body, `gruvbox-dark` and the text between its
/// braces.
fn blocks() -> Vec<(&'static str, &'static str)> {
    CSS.split(".theme-")
        .skip(1)
        .filter_map(|chunk| {
            let (name, rest) = chunk.split_once(" {")?;
            if !(name.ends_with("-dark") || name.ends_with("-light")) {
                return None;
            }
            let (body, _) = rest.split_once('}')?;
            Some((name, body))
        })
        .collect()
}

fn value(body: &str, slot: &str) -> Option<String> {
    let key = format!("--syntax-{slot}:");
    let line = body.lines().find(|l| l.trim().starts_with(&key))?;
    Some(
        line.trim()
            .trim_start_matches(&key)
            .trim()
            .trim_end_matches(';')
            .to_string(),
    )
}

fn allowed(v: &str) -> bool {
    let hex = v.len() == 7
        && v.starts_with('#')
        && v[1..]
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
    let own = [
        "var(--text-primary)",
        "var(--text-muted)",
        "var(--text-subtle)",
    ]
    .contains(&v)
        || (1..=6).any(|n| v == format!("var(--palette-{n})"));
    hex || own
}

#[test]
fn every_theme_block_has_the_eleven_syntax_slots() {
    let blocks = blocks();
    assert_eq!(blocks.len(), 74, "37 themes, dark and light");
    for (name, body) in blocks {
        for slot in SLOTS {
            let v = value(body, slot).unwrap_or_default();
            assert!(!v.is_empty(), "{name} has no --syntax-{slot}");
            assert!(allowed(&v), "{name} --syntax-{slot}: {v}");
        }
    }
}
