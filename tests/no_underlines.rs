//! The link gate: no link is underlined. A text link reads as one by the accent color alone, at
//! rest, under a mouse and on keyboard focus, so no kit sheet sets an underline on anything.

const SHEETS: [(&str, &str); 5] = [
    ("components.css", include_str!("../assets/components.css")),
    ("app.css", include_str!("../assets/app.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("toast.css", include_str!("../assets/toast.css")),
    ("nav_fold.css", include_str!("../assets/nav_fold.css")),
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

#[test]
fn no_kit_sheet_underlines_a_link() {
    for (name, sheet) in SHEETS {
        for (number, line) in without_comments(sheet).lines().enumerate() {
            assert!(
                !line.contains("underline"),
                "{name}:{}: {}",
                number.saturating_add(1),
                line.trim()
            );
        }
    }
}
