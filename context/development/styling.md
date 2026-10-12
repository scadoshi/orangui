# Styling

How the stylesheets are written, and the two gates `cargo test` holds them to.

## The cascade

A host inlines the sheets in this order: `THEMES_CSS`, then `COMPONENTS_CSS`, then `APP_CSS` (apps) or `SITE_CSS` (websites), then its own stylesheet. Component rules resolve theme variables, and the host's rules override shared defaults at equal specificity. A component rule never carries a color literal: it names a theme variable, so it resolves to whichever theme the host has active.

Class names are the public surface as much as the Rust names. A host styles against them, so renaming one is a breaking change. Every section of `components.css` opens with a one-line comment naming the component it styles, in the form `/* ---- Panel (orangui::Panel) ---- */`.

## Colors: three tiers, one job each

| Tier | Variables | Job | Decorative use |
|---|---|---|---|
| Semantic accents | `--accent-primary`, `--accent-secondary`, `--accent-tertiary` | title and link, selected state, eyebrow | no |
| Status | `--color-success`, `--color-warning`, `--color-error` and their `--border-*` twins | outcomes: saved, over budget, failed | never |
| Palette | `--palette-1` to `--palette-6` | variety with no meaning: tags, chart series, chip tints, logo stripes | only here |

No variable crosses tiers. A status variable means an outcome and ships with its context (a toast, a check, an error box); a palette slot means nothing. A slot may hold the same hex as a status color, since in half the themes the only green and yellow are the success and warning colors. The reservation is on the variable, not the color.

Slot order is fixed across themes, so a given element has the same temperature everywhere and neighbors in a row alternate warm and cool:

| Slot | Role | Gruvbox dark |
|---|---|---|
| 1 | cool, the theme's blue | `#83a598` |
| 2 | warm, the theme's orange or red-orange | `#fe8019` |
| 3 | the theme's green | `#b8bb26` |
| 4 | the theme's violet or pink | `#d3869b` |
| 5 | the theme's yellow | `#fabd2f` |
| 6 | the theme's cyan or aqua | `#8ec07c` |

Where a theme has fewer than six hues, slots repeat the theme's own published shades. Nothing is invented. Values come from published palettes, with the source in a comment above each theme block. The four accessibility themes (protanopia, deuteranopia, tritanopia, achromatopsia) are built from the deficiency outward instead.

## Themes

A theme is a `.theme-{name}-dark` block and a `.theme-{name}-light` block in `assets/themes.css`, and its name is in `ALLOWED_THEMES` in `src/theme.rs`. The `:root` block repeats gruvbox dark as the default. Each block defines every variable the components reference: backgrounds, text, borders, the three accents, the three status colors with their borders, `--color-money`, the six palette slots and the shadows.

## The gates

- `tests/themes_palette.rs`: every theme block carries `--palette-1` to `--palette-6` and `--color-money`, and each slot reads on the block's `--bg-primary` at 3:1 or better. Pairs closer than 15 in OKLab are printed, never failed, since published colors stay as published. The accessibility themes are the exception: their six are built, so every pair must stay 8 apart under the deficiency they are built for.
- `tests/status_colors.rs`: every status variable reference in the crate's own sheets sits under a selector in the test's allowlist, each of which marks an outcome. Decoration that wants a color takes a palette slot. A host holds its own stylesheet to the same rule with `status_uses`.
- `src/theme.rs`: the theme list and the stylesheet name the same themes, alphabetically, each with both blocks.
