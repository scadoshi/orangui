//! Toasts and banners sit in the bottom-left corner of the window, as far from its left edge as
//! from its bottom edge, on every site, with nothing lifting them over a dock or a footer. One
//! inset, `--toast-inset`, places both stacks; the safe area adds to each side on its own.

const TOAST: &str = include_str!("../assets/toast.css");
const COMPONENTS: &str = include_str!("../assets/components.css");

/// The declarations of the rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    let (_, rest) = css
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selector}"));
    body
}

/// The value `property` takes in `declarations`, if it is declared there.
fn value<'css>(declarations: &'css str, property: &str) -> Option<&'css str> {
    let prefix = format!("{property}:");
    declarations
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix.as_str()))
        .map(|v| v.trim().trim_end_matches(';'))
}

const LEFT: &str = "calc(var(--toast-inset, var(--space-4)) + env(safe-area-inset-left, 0px))";
const BOTTOM: &str = "calc(var(--toast-inset, var(--space-4)) + env(safe-area-inset-bottom, 0px))";

#[test]
fn toasts_sit_bottom_left_at_one_inset_from_both_edges() {
    let stack = rule(TOAST, ".toast-container");
    assert_eq!(value(stack, "left"), Some(LEFT));
    assert_eq!(value(stack, "bottom"), Some(BOTTOM));
    assert_eq!(value(stack, "top"), None, "nothing anchors it to the top");
}

#[test]
fn banners_sit_in_the_same_corner_at_the_same_inset() {
    let stack = rule(COMPONENTS, ".banner-stack");
    assert_eq!(value(stack, "left"), Some(LEFT));
    assert_eq!(value(stack, "bottom"), Some(BOTTOM));
}

#[test]
fn the_newest_toast_is_at_the_bottom_and_rises_in() {
    assert_eq!(
        value(rule(TOAST, ".toast-container ol"), "flex-direction"),
        Some("column-reverse")
    );
    assert_eq!(
        value(rule(TOAST, ".toast-container li"), "margin-bottom"),
        Some("-2.75rem"),
        "older toasts tuck above the newest"
    );
    assert!(
        rule(TOAST, ".toast-container [role=\"alertdialog\"]").contains("toast-rise-in"),
        "a toast enters from below, the way the stack grows"
    );
}
