//! Nested corners: a rounded box that sits near another's corner takes the outer radius minus
//! the border and the inset between them (never under 0.15rem), so the gap between the two
//! curves stays even around the corner. The outer radius and the inset are tokens, so a host
//! that changes one moves the inner corner with it.

const COMPONENTS: &str = include_str!("../assets/components.css");
const TOAST: &str = include_str!("../assets/toast.css");

/// The body of the rule for `selector` in `sheet`.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let Some(at) = sheet.find(&format!("\n{selector} {{")) else {
        return "";
    };
    sheet
        .get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn dock_buttons_follow_the_dock_corner() {
    let dock = rule(COMPONENTS, ":where(.action-dock)");
    assert!(dock.contains("--dock-radius: 0.9rem;"), "{dock:?}");
    assert!(dock.contains("--dock-inset: 0.3rem;"), "{dock:?}");
    assert!(
        dock.contains("border-radius: var(--dock-radius);"),
        "{dock:?}"
    );
    let button = rule(COMPONENTS, ":where(.action-dock-btn)");
    assert!(
        button.contains("margin: var(--dock-inset) 0;"),
        "{button:?}"
    );
    assert!(
        button.contains(
            "border-radius: max(0.15rem, calc(var(--dock-radius) - 1px - var(--dock-inset)));"
        ),
        "{button:?}"
    );
}

#[test]
fn the_banner_dismiss_follows_the_banner_corner() {
    let banner = rule(COMPONENTS, ".banner");
    assert!(banner.contains("--banner-radius: 0.4rem;"), "{banner:?}");
    assert!(
        banner.contains("border-radius: var(--banner-radius);"),
        "{banner:?}"
    );
    let dismiss = rule(COMPONENTS, ".banner .banner-dismiss");
    assert!(
        dismiss.contains("--banner-dismiss-inset: 0.25rem;"),
        "{dismiss:?}"
    );
    assert!(
        dismiss.contains("top: var(--banner-dismiss-inset);"),
        "{dismiss:?}"
    );
    assert!(
        dismiss.contains(
            "border-radius: max(0.15rem, calc(var(--banner-radius) - 1px - var(--banner-dismiss-inset)));"
        ),
        "{dismiss:?}"
    );
}

#[test]
fn the_toast_close_follows_the_toast_corner() {
    let toast = rule(TOAST, ".toast-container [role=\"alertdialog\"]");
    assert!(toast.contains("--toast-radius: 0.6rem;"), "{toast:?}");
    assert!(toast.contains("--toast-pad: 1rem;"), "{toast:?}");
    assert!(toast.contains("padding: var(--toast-pad);"), "{toast:?}");
    assert!(
        toast.contains("border-radius: var(--toast-radius);"),
        "{toast:?}"
    );
    let close = rule(TOAST, ".toast-container [role=\"alertdialog\"] button");
    assert!(
        close.contains(
            "border-radius: max(0.15rem, calc(var(--toast-radius) - 1px - (var(--toast-pad) - 0.5rem)));"
        ),
        "the close button sits half a rem inside the padding: {close:?}"
    );
}
