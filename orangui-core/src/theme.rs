//! A theme: a name from [`ALLOWED_THEMES`] and a light or dark mode, mapped to
//! the class the host puts on its themed element. The palettes themselves are
//! in orangui's `assets/themes.css`, which a test there holds to this list; a host that persists a user's theme elsewhere (a
//! server, a settings store) keeps its own copy of this list and checks it
//! against this one in a test.

use serde::{Deserialize, Serialize};

/// The theme slugs orangui's `assets/themes.css` defines, alphabetical, each with a
/// `-dark` and a `-light` block.
pub const ALLOWED_THEMES: &[&str] = &[
    "achromatopsia",
    "ayu",
    "bluloco",
    "catppuccin",
    "darcula",
    "deuteranopia",
    "docs-rs",
    "dracula",
    "eclipse",
    "ethereal",
    "everforest",
    "flexoki",
    "github",
    "github-high-contrast",
    "gruvbox",
    "hackerman",
    "kanagawa",
    "kanagawa-dragon",
    "material",
    "matte-black",
    "melange",
    "miasma",
    "modus",
    "monokai",
    "monokai-pro",
    "monokai-pro-machine",
    "monokai-pro-octagon",
    "monokai-pro-ristretto",
    "monokai-pro-spectrum",
    "neovim",
    "night-owl",
    "nightfox",
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
    "terafox",
    "tokyo-night",
    "tritanopia",
    "vantablack",
    "vscode",
    "xcode",
    "zenburn",
];

/// A section of the theme pickers: a label and the themes under it,
/// alphabetical. The sections sort by look, warm to cool to neutral, so
/// someone who has never heard of a theme finds one by the color they want;
/// a family whose variants differ in look stays together under its flagship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeGroup {
    /// The section's heading.
    pub label: &'static str,
    /// The slugs under it, alphabetical.
    pub themes: &'static [&'static str],
}

/// Every allowed theme, once, in the order the pickers show them.
pub const THEME_GROUPS: &[ThemeGroup] = &[
    ThemeGroup {
        label: "Warm & earthy",
        themes: &[
            "darcula",
            "flexoki",
            "gruvbox",
            "kanagawa-dragon",
            "melange",
            "miasma",
            "rustbox",
            "zenburn",
        ],
    },
    ThemeGroup {
        label: "Cool & blue",
        themes: &[
            "ayu",
            "bluloco",
            "kanagawa",
            "night-owl",
            "nightfox",
            "nord",
            "one-dark",
            "poimandres",
            "powershell",
            "solarized",
            "tokyo-night",
        ],
    },
    ThemeGroup {
        label: "Green & teal",
        themes: &["everforest", "hackerman", "osaka-jade", "terafox"],
    },
    ThemeGroup {
        label: "Purple & pink",
        themes: &["catppuccin", "dracula", "rose-pine", "synthwave-84"],
    },
    ThemeGroup {
        label: "Black & gray",
        themes: &[
            "docs-rs",
            "eclipse",
            "github",
            "matte-black",
            "neovim",
            "oxocarbon",
            "vantablack",
            "vscode",
        ],
    },
    ThemeGroup {
        label: "Bright & colorful",
        themes: &[
            "ethereal",
            "material",
            "monokai",
            "monokai-pro",
            "monokai-pro-machine",
            "monokai-pro-octagon",
            "monokai-pro-ristretto",
            "monokai-pro-spectrum",
            "xcode",
        ],
    },
    ThemeGroup {
        label: "High contrast & color blind",
        themes: &[
            "achromatopsia",
            "deuteranopia",
            "github-high-contrast",
            "modus",
            "protanopia",
            "tritanopia",
        ],
    },
];

/// Retired slugs, ones [`ALLOWED_THEMES`] does not list, each mapped to the
/// theme its users move to: a renamed theme to its new slug, a removed one to the
/// closest look. A host that stores slugs applies the same map and checks it
/// against this one in a test.
pub const RETIRED_THEMES: &[(&str, &str)] = &[
    ("ristretto", "monokai-pro-ristretto"),
    ("visual-studio", "vscode"),
];

/// The listed slug a stored one stands for: itself when still listed, its
/// replacement when retired, `None` when it names no theme.
#[must_use]
pub fn current_theme(slug: &str) -> Option<&'static str> {
    let slug = RETIRED_THEMES
        .iter()
        .find(|(from, _)| *from == slug)
        .map_or(slug, |(_, to)| *to);
    ALLOWED_THEMES.iter().copied().find(|t| *t == slug)
}

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
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme identifier, one of [`ALLOWED_THEMES`].
    pub name: String,
    /// Whether dark mode is active.
    pub is_dark: bool,
}

impl ThemeConfig {
    /// The class to put on the themed element: `theme-{name}-{dark|light}`.
    #[must_use]
    pub fn css_class(&self) -> String {
        let mode = if self.is_dark { "dark" } else { "light" };
        format!("theme-{}-{}", self.name, mode)
    }

    /// This theme under its listed slug (see [`current_theme`]), or `None`
    /// when the slug names no theme, so a stored pick never puts a class
    /// with no palette on the page.
    #[must_use]
    pub fn resolve(self) -> Option<Self> {
        let name = current_theme(&self.name)?.to_string();
        Some(Self { name, ..self })
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

    #[test]
    fn every_retired_theme_is_unlisted_and_moves_to_a_listed_one() {
        for (from, to) in RETIRED_THEMES {
            assert!(
                !ALLOWED_THEMES.contains(from),
                "{from} is retired but listed"
            );
            assert!(ALLOWED_THEMES.contains(to), "{from} moves to unlisted {to}");
        }
    }

    #[test]
    fn a_stored_slug_resolves_to_a_listed_theme_or_none() {
        assert_eq!(current_theme("flexoki"), Some("flexoki"));
        assert_eq!(current_theme("visual-studio"), Some("vscode"));
        assert_eq!(current_theme("ristretto"), Some("monokai-pro-ristretto"));
        assert_eq!(current_theme("no-such-theme"), None);

        let stored = ThemeConfig {
            name: "visual-studio".to_string(),
            is_dark: false,
        };
        let resolved = stored.resolve().map(|t| t.css_class());
        assert_eq!(resolved.as_deref(), Some("theme-vscode-light"));
    }

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
    fn every_theme_sits_in_exactly_one_group() {
        let mut seen: Vec<&str> = Vec::new();
        for group in THEME_GROUPS {
            for slug in group.themes {
                assert!(ALLOWED_THEMES.contains(slug), "{slug}");
                assert!(!seen.contains(slug), "{slug} is in two groups");
                seen.push(slug);
            }
        }
        assert_eq!(seen.len(), ALLOWED_THEMES.len());
    }

    #[test]
    fn groups_are_alphabetical_inside() {
        for group in THEME_GROUPS {
            let mut sorted = group.themes.to_vec();
            sorted.sort_unstable();
            assert_eq!(sorted, group.themes, "{}", group.label);
        }
    }

    #[test]
    fn the_colorblind_themes_share_the_last_group() {
        for slug in COLORBLIND_THEMES {
            assert!(
                THEME_GROUPS.last().is_some_and(|g| g.themes.contains(slug)),
                "{slug}"
            );
        }
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
