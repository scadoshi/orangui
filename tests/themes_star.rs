//! Every theme has a star color, `--color-star`, for a starred thing: its star and its marker.
//! It defaults to the theme's own warning gold, at no specificity, so a theme block may name its
//! own and keep it.

/// The kit's theme sheet.
const THEMES: &str = include_str!("../assets/themes.css");

#[test]
fn every_theme_stars_in_its_warning_gold_unless_it_names_its_own() {
    let at = THEMES
        .find("--color-star: var(--color-warning);")
        .expect("the star's default");
    let head = THEMES.get(..at).expect("the sheet before it");
    let selector = head
        .get(head.rfind("}\n").expect("the rule before")..)
        .expect("the selector");
    assert!(
        selector.contains(":where(:root, [class*=\"theme-\"][class*=\"-dark\"], [class*=\"theme-\"][class*=\"-light\"])"),
        "the default reaches every theme at no specificity, so a theme's own wins: {selector}"
    );
    assert!(
        THEMES.contains("--ink-color-star: var(--ink-color-warning, var(--color-warning));"),
        "its ink follows the warning's"
    );
    assert_eq!(
        THEMES.matches("--color-star:").count(),
        1,
        "no theme names its own yet, so every theme takes the default"
    );
}
