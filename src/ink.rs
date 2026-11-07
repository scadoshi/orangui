//! The readable text color for a theme variable.
//!
//! Every theme block carries an `--ink-<tone>` beside each chip tone: the tone
//! itself where it reads at 4.5:1 on its tint and the backgrounds, else the
//! nearest color that does (`context/development/styling.md`). Text drawn in a
//! tone from an inline style goes through [`ink_color`] so it gets the same.

/// The CSS value that draws text in `variable`'s ink, falling back to
/// `variable` itself where the theme has no ink for it.
///
/// `variable` is a custom property name with its leading dashes, such as
/// `--palette-2`.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     orangui::ink_color("--palette-2"),
///     "var(--ink-palette-2, var(--palette-2))"
/// );
/// ```
#[must_use]
pub fn ink_color(variable: &str) -> String {
    let name = variable.trim_start_matches("--");
    format!("var(--ink-{name}, var(--{name}))")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tone_reads_its_ink_then_itself() {
        assert_eq!(
            ink_color("--color-warning"),
            "var(--ink-color-warning, var(--color-warning))"
        );
    }

    #[test]
    fn a_name_without_dashes_reads_the_same() {
        assert_eq!(ink_color("palette-1"), ink_color("--palette-1"));
    }
}
