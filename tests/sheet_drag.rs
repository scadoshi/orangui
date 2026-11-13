//! A pulled sheet springs back once let go. The pull rides a class and a custom property, not an
//! inline `transform`: Dioxus keeps any inline property a new `style` value leaves out, so a
//! sheet whose style also carries its layer would keep the pull's transform after release.

#[test]
fn the_pull_writes_only_its_distance_inline() {
    let style = orangui::SheetDrag::style_for(40.0);
    assert_eq!(style, "--sheet-drag: 40px;");
    assert!(!style.contains("transform"), "{style}");
    assert!(!style.contains("transition"), "{style}");
}

#[test]
fn the_dragging_class_moves_the_sheet_by_the_distance_with_no_easing() {
    let css = orangui::COMPONENTS_CSS;
    let rule = ".bottom-sheet.sheet-dragging {";
    let at = css.find(rule).unwrap_or_else(|| panic!("no rule {rule}"));
    let body = css
        .get(at..)
        .and_then(|s| s.split('}').next())
        .unwrap_or_default();
    assert!(
        body.contains("transform: translateY(var(--sheet-drag, 0px)) !important;"),
        "{body}"
    );
    assert!(body.contains("transition: none !important;"), "{body}");
}

#[test]
fn a_drawer_pulls_to_the_right_on_its_own_class() {
    assert_eq!(orangui::Pull::Down.class(), " sheet-dragging");
    assert_eq!(orangui::Pull::Right.class(), " sheet-dragging-right");
    let css = orangui::COMPONENTS_CSS;
    let rule = ".bottom-sheet.sheet-dragging-right {";
    let at = css.find(rule).unwrap_or_else(|| panic!("no rule {rule}"));
    let body = css
        .get(at..)
        .and_then(|s| s.split('}').next())
        .unwrap_or_default();
    assert!(
        body.contains("transform: translateX(var(--sheet-drag, 0px)) !important;"),
        "{body}"
    );
    assert!(body.contains("transition: none !important;"), "{body}");
}
