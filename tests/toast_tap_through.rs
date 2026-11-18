//! A lone toast lets a tap through to whatever sits under it, such as a dock's buttons, and takes
//! only the taps on its own buttons. A stack of two or more takes a tap anywhere on it, which
//! expands or collapses the stack. A leaving toast's copy takes no tap at all.

const TOAST: &str = include_str!("../assets/toast.css");
const TOAST_STACK: &str = include_str!("../src/toast_stack.rs");

/// The live container while it holds one toast: not a leaving copy, and no second item.
const LONE: &str = ".toast-container:not(.toast-ghost):not(:has(li + li))";

/// The declarations of the first rule whose selector list is exactly `selectors` in `css`.
fn rule<'css>(css: &'css str, selectors: &str) -> &'css str {
    let opening = format!("\n{selectors} {{");
    let (_, rest) = css
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selectors}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selectors}"));
    body
}

#[test]
fn a_lone_toast_lets_taps_through_but_on_its_buttons() {
    let through = rule(TOAST, &format!("{LONE},\n{LONE} *"));
    assert!(
        through.contains("pointer-events: none;"),
        "the lone toast and everything in it pass taps through: {through}"
    );
    let buttons = rule(TOAST, &format!("{LONE} button"));
    assert!(
        buttons.contains("pointer-events: auto;"),
        "its own buttons still take taps: {buttons}"
    );
}

#[test]
fn a_stack_of_two_or_more_still_expands_on_tap() {
    for (at, _) in TOAST.match_indices("pointer-events: none;") {
        // The text between the rule's `{` and whatever ended before it: a rule's `}` or a
        // comment's `*/`.
        let selector = TOAST
            .get(..at)
            .and_then(|head| head.rsplit_once('{'))
            .map_or("", |(before, _)| before);
        let selector = selector.rsplit(['}', '/']).next().map_or("", str::trim);
        assert!(
            selector.starts_with(LONE) || selector == ".toast-live-old",
            "only a lone toast passes taps through, never a stack: {selector}"
        );
    }
    assert!(
        TOAST_STACK.contains("onclick: move |_| tap.toggle()"),
        "a tap on a toast toggles the stack"
    );
}
