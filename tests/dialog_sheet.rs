//! The alert dialog's rules live in their own sheet, so a site can draw the kit's dialogs without
//! the app shell, while an app keeps them through `APP_CSS`.

/// The app shell's own sheet, without the dialog.
const APP_SHEET: &str = include_str!("../assets/app.css");

#[test]
fn the_dialog_has_a_sheet_of_its_own() {
    for rule in [
        ".alert-dialog-overlay",
        ".alert-dialog {",
        ".alert-dialog-actions",
        "@keyframes alert-dialog-in",
    ] {
        assert!(orangui::DIALOG_CSS.contains(rule), "{rule}");
    }
    assert!(
        !APP_SHEET.contains(".alert-dialog"),
        "the dialog's rules have one home"
    );
}

#[test]
fn an_app_still_gets_the_dialog_from_app_css() {
    assert!(orangui::APP_CSS.contains(APP_SHEET));
    assert!(orangui::APP_CSS.contains(orangui::DIALOG_CSS));
}

#[test]
fn a_danger_action_leading_the_row_stands_apart() {
    let rule = ".alert-dialog-actions > .alert-dialog-action-danger:first-child {\n    margin-inline-end: 0.75rem;\n}";
    assert!(orangui::DIALOG_CSS.contains(rule), "{rule}");
}
