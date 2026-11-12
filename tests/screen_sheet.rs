//! The app screen's frame and page header live in their own sheet, so a site that opens an app
//! screen over its page draws it as the apps do, while an app keeps them at the head of
//! `APP_CSS`, in the same order as before.

/// The app shell's own sheet, without the screen frame.
const APP_SHEET: &str = include_str!("../assets/app.css");

#[test]
fn the_screen_and_page_header_have_a_sheet_of_their_own() {
    for rule in [
        ".screen-content {",
        ".screen:has(> .grid-cells)",
        ".page-header {",
        ".page-header-corner {",
    ] {
        assert!(orangui::SCREEN_CSS.contains(rule), "{rule}");
        assert!(!APP_SHEET.contains(rule), "{rule} has one home");
    }
    assert!(
        !orangui::SCREEN_CSS.contains(".hint-line"),
        "the hints stay in the app shell"
    );
}

#[test]
fn an_app_gets_the_screen_first_then_the_rest_of_the_shell() {
    assert!(orangui::APP_CSS.starts_with(orangui::SCREEN_CSS));
    let screen = orangui::APP_CSS.find(".page-header-corner {").unwrap();
    let hints = orangui::APP_CSS.find(".hint-line").unwrap();
    assert!(
        screen < hints,
        "the screen's rules keep their place before the hints"
    );
}
