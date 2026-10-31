//! The list gate: `assets/themes.css` defines exactly the themes
//! [`orangui::ALLOWED_THEMES`] lists, a dark and a light block each, and no
//! palette for a retired one.

const CSS: &str = include_str!("../assets/themes.css");

#[test]
fn every_retired_theme_has_no_palette() {
    for (from, _) in orangui::RETIRED_THEMES {
        assert!(
            !CSS.contains(&format!(".theme-{from}-dark")),
            "{from} is retired but still has a palette"
        );
    }
}

#[test]
fn every_allowed_theme_has_both_blocks_in_the_stylesheet() {
    for slug in orangui::ALLOWED_THEMES {
        for mode in ["dark", "light"] {
            let class = format!(".theme-{slug}-{mode} {{");
            assert!(CSS.contains(&class), "{class} missing from themes.css");
        }
    }
}

#[test]
fn every_stylesheet_theme_is_allowed() {
    for line in CSS.lines() {
        let Some(rest) = line.trim().strip_prefix(".theme-") else {
            continue;
        };
        let Some(slug) = rest
            .strip_suffix("-dark {")
            .or_else(|| rest.strip_suffix("-light {"))
        else {
            continue;
        };
        assert!(
            orangui::ALLOWED_THEMES.contains(&slug),
            "{slug} is in themes.css but not ALLOWED_THEMES"
        );
    }
}
