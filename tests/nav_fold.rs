//! The `NavBar`'s fold rules live once, in `NAV_FOLD_CSS`, and `COMPONENTS_CSS`
//! applies them at the kit's own breakpoint, so a host that folds wider adds
//! the same rules under its own query instead of copying them.

use orangui::{COMPONENTS_CSS, NAV_FOLD_CSS};

#[test]
fn the_kit_folds_the_nav_at_60rem_with_the_exported_rules() {
    let wrapped = format!("@media (max-width: 60rem) {{\n{NAV_FOLD_CSS}}}");
    assert!(
        COMPONENTS_CSS.contains(&wrapped),
        "COMPONENTS_CSS applies NAV_FOLD_CSS under max-width 60rem"
    );
    assert_eq!(
        COMPONENTS_CSS.matches("(max-width: 60rem)").count(),
        1,
        "every hamburger-width rule is in NAV_FOLD_CSS"
    );
}

#[test]
fn the_fold_rules_carry_no_width_query_of_their_own() {
    assert!(
        !NAV_FOLD_CSS.contains("@media (max-width") && !NAV_FOLD_CSS.contains("@media (min-width"),
        "a host's query sets the width"
    );
    assert!(NAV_FOLD_CSS.contains(".nav-toggle"), "the toggle shows");
}
