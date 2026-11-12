//! Status colors mean an outcome. Decoration draws from the palette slots. Every
//! status reference in the crate's own stylesheets must be one of these.

const ALLOWED: [&str; 14] = [
    ".alert-dialog-action-danger",
    ".input-error, .input-error:focus",
    ".message-error",
    ".btn-danger, .btn-action-danger",
    // A destructive button's hover and focus: the same red, as a tint.
    r#":is(.btn-danger, .btn-action-danger, .alert-dialog-action-danger):not(:disabled, [aria-disabled="true"]):hover"#,
    r#":is(.btn-danger, .btn-action-danger, .alert-dialog-action-danger):not(:disabled, [aria-disabled="true"]):focus-visible"#,
    ".tag-error, .stat-chip-error",
    ".tag-success, .stat-chip-success",
    ".tag-warning, .stat-chip-warning",
    ".status-doing",
    ".status-done",
    ".toast-container [data-type=\"error\"]",
    ".toast-container [data-type=\"success\"]",
    ".toast-container [data-type=\"warning\"]",
];

#[test]
fn status_colors_only_where_they_mean_something() {
    let sheets = [
        include_str!("../assets/components.css"),
        include_str!("../assets/toast.css"),
        include_str!("../assets/app.css"),
        include_str!("../assets/dialog.css"),
    ];
    let offenders: Vec<String> = sheets
        .iter()
        .flat_map(|css| orangui::status_uses(css))
        .filter(|u| !ALLOWED.contains(&u.selector.as_str()))
        .map(|u| format!("{} uses {}", u.selector, u.variable))
        .collect();
    assert!(
        offenders.is_empty(),
        "status colors used as decoration (move to a --palette-N slot, or add the selector to ALLOWED if it marks an outcome):\n  {}",
        offenders.join("\n  ")
    );
}
