//! A theme: a name from [`ALLOWED_THEMES`] and a light or dark mode, mapped to
//! the class the host puts on its themed element. The palettes themselves are
//! in `assets/themes.css`; a host that persists a user's theme elsewhere (a
//! server, a settings store) keeps its own copy of this list and checks it
//! against this one in a test.

use serde::{Deserialize, Serialize};

/// The theme slugs `assets/themes.css` defines, alphabetical, each with a
/// `-dark` and a `-light` block.
pub const ALLOWED_THEMES: &[&str] = &[
    "achromatopsia",
    "ayu",
    "catppuccin",
    "darcula",
    "deuteranopia",
    "docs-rs",
    "dracula",
    "ethereal",
    "everforest",
    "github",
    "github-high-contrast",
    "gruvbox",
    "hackerman",
    "kanagawa",
    "kanagawa-dragon",
    "matte-black",
    "miasma",
    "monokai",
    "monokai-pro",
    "monokai-pro-machine",
    "monokai-pro-octagon",
    "monokai-pro-ristretto",
    "monokai-pro-spectrum",
    "night-owl",
    "nord",
    "one-dark",
    "osaka-jade",
    "oxocarbon",
    "poimandres",
    "powershell",
    "protanopia",
    "rose-pine",
    "rustbox",
    "solarized",
    "synthwave-84",
    "tokyo-night",
    "tritanopia",
    "vantablack",
    "vscode",
    "zenburn",
];

/// The themes with palettes adjusted for color-vision deficiency. Every theme
/// picker groups these in their own section at the bottom, so the list lives
/// here once.
pub const COLORBLIND_THEMES: &[&str] =
    &["protanopia", "deuteranopia", "tritanopia", "achromatopsia"];

/// Human-readable label for a theme slug: title-cased words, with the
/// accents and brand casings that title-casing cannot produce special-cased.
#[must_use]
pub fn display_theme_name(slug: &str) -> String {
    match slug {
        "rose-pine" => return "Rosé Pine".to_string(),
        "vscode" => return "VS Code".to_string(),
        "github" => return "GitHub".to_string(),
        "github-high-contrast" => return "GitHub High Contrast".to_string(),
        "synthwave-84" => return "Synthwave '84".to_string(),
        "powershell" => return "PowerShell".to_string(),
        "docs-rs" => return "docs.rs".to_string(),
        _ => {}
    }
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The theme a host displays. Serializable so a host can keep the last pick
/// in local storage and theme its first paint before anything else loads.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme identifier, one of [`ALLOWED_THEMES`].
    pub name: String,
    /// Whether dark mode is active.
    pub is_dark: bool,
}

impl ThemeConfig {
    /// The class to put on the themed element: `theme-{name}-{dark|light}`.
    pub fn css_class(&self) -> String {
        let mode = if self.is_dark { "dark" } else { "light" };
        format!("theme-{}-{}", self.name, mode)
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "gruvbox".to_string(),
            is_dark: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSS: &str = include_str!("../assets/themes.css");

    #[test]
    fn theme_names_title_case_and_keep_brand_spellings() {
        assert_eq!(display_theme_name("tokyo-night"), "Tokyo Night");
        assert_eq!(display_theme_name("gruvbox"), "Gruvbox");
        assert_eq!(display_theme_name("rose-pine"), "Rosé Pine");
        assert_eq!(display_theme_name("vscode"), "VS Code");
        assert_eq!(display_theme_name("docs-rs"), "docs.rs");
        assert_eq!(
            display_theme_name("github-high-contrast"),
            "GitHub High Contrast"
        );
        assert_eq!(
            display_theme_name("monokai-pro-ristretto"),
            "Monokai Pro Ristretto"
        );
        assert_eq!(
            display_theme_name("monokai-pro-octagon"),
            "Monokai Pro Octagon"
        );
    }

    #[test]
    fn every_colorblind_theme_is_allowed() {
        for slug in COLORBLIND_THEMES {
            assert!(ALLOWED_THEMES.contains(slug), "{slug}");
        }
    }

    #[test]
    fn every_allowed_theme_has_a_non_empty_label() {
        for slug in ALLOWED_THEMES {
            assert!(!display_theme_name(slug).is_empty(), "{slug}");
        }
    }

    #[test]
    fn allowed_themes_are_alphabetical() {
        let mut sorted = ALLOWED_THEMES.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, ALLOWED_THEMES);
    }

    #[test]
    fn every_allowed_theme_has_both_blocks_in_the_stylesheet() {
        for slug in ALLOWED_THEMES {
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
                ALLOWED_THEMES.contains(&slug),
                "{slug} is in themes.css but not ALLOWED_THEMES"
            );
        }
    }

    #[test]
    fn default_is_gruvbox_dark() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.name, "gruvbox");
        assert!(theme.is_dark);
        assert_eq!(theme.css_class(), "theme-gruvbox-dark");
    }

    #[test]
    fn light_mode_class() {
        let theme = ThemeConfig {
            name: "nord".to_string(),
            is_dark: false,
        };
        assert_eq!(theme.css_class(), "theme-nord-light");
    }
}
