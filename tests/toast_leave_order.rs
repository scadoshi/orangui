//! A leaving toast keeps its place in the stack's order for the whole of its leave: its copy sits
//! in the live toasts' own stacking context, beside their container, so a host whose app shell
//! isolates its layers can't lift the copy over every live toast; behind them when a newer toast
//! is still up, in front when none is.

const LEAVE: &str = include_str!("../assets/toast-leave.js");

#[test]
fn the_copy_sits_beside_the_live_toasts_not_on_the_page() {
    assert!(
        LEAVE.contains("container.after(holder);"),
        "beside the container, in its stacking context: {LEAVE}"
    );
    assert!(
        !LEAVE.contains("document.body.appendChild(holder)"),
        "not on the page, over an isolated app shell"
    );
    assert!(
        LEAVE.contains("holder.getBoundingClientRect()"),
        "corrects its place under a transformed parent: {LEAVE}"
    );
}

#[test]
fn the_copy_goes_behind_whenever_a_newer_toast_is_up() {
    assert!(
        LEAVE.contains("record.previousSibling"),
        "read off the list as the toast leaves it, an add in the same batch included: {LEAVE}"
    );
    assert!(LEAVE.contains("front ? '10000' : '9998'"), "{LEAVE}");
}
