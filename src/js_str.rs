//! Host strings written into generated scripts as JavaScript string literals.

use std::fmt::Write;

/// `s` as a double-quoted JavaScript string literal, safe to splice into a script wherever an
/// expression goes. Quotes, backslashes, line breaks and other control characters are escaped,
/// so a selector like `[aria-label='x']` stays one string.
pub(crate) fn js_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len().saturating_add(2));
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Control characters, and the two line terminators JavaScript reads inside a string.
            c if c.is_control() || c == '\u{2028}' || c == '\u{2029}' => {
                let _: std::fmt::Result = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_selector_is_quoted_as_is() {
        assert_eq!(js_str(".bottom-sheet.show"), r#"".bottom-sheet.show""#);
    }

    #[test]
    fn quotes_backslashes_and_line_breaks_stay_inside_the_literal() {
        assert_eq!(js_str("[aria-label='x']"), r#""[aria-label='x']""#);
        assert_eq!(js_str(r#"[data-k="a\b"]"#), r#""[data-k=\"a\\b\"]""#);
        assert_eq!(js_str("a\nb\u{2028}c\u{1}"), r#""a\nb\u2028c\u0001""#);
    }
}
