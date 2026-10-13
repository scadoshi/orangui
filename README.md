# orangui

A Dioxus UI kit: themes, panels, nav, charts and the entrance effects shared by my apps and sites. One crate renders and styles the pieces, so the surfaces that use it never drift. The name is orangutan, a call sign of mine, plus GUI.

## Use

```toml
[dependencies]
orangui = { git = "https://github.com/scadoshi/orangui" }
```

Add `features = ["toast"]` for the toast stack. The crate depends on base `dioxus` with no platform feature, so the host picks web, mobile, desktop or fullstack and cargo unifies. The version stays at 0.0.0: hosts pin a commit through their lockfile and move it on purpose, so nothing reads the number yet.

Styling is inlined by the host, in this order: `THEMES_CSS`, then `COMPONENTS_CSS`, then `APP_CSS` (apps) or `SITE_CSS` (websites), then the host's own stylesheet. Component rules resolve theme variables, and the host's rules override shared defaults at equal specificity.

## What is in it

For every host: `ActionBar`, `Banner`, `Button`, `Chip`, `NavBar`, `NavDropdown`, `PageMeta`, `Panel`, `PasswordInput`, `ThemePicker`; the diagram pieces (`DiagramNode`, `DiagramArrow`, `DiagramDefs`); the entrance figures (`CountUp`, `Decode`, `Figure`, `Replay`); `HeatGrid` for a year of days; and the chart helpers `curve`, `area`, `tip_anchor`, `peak_indices`.

The app kit: `BottomSheet`, `PageHeader`, `ThemeSheet`, the hint pieces (`InfoButton`, `HintLine`, `HintBullets`, `HintBullet`, `HintKey`), the OS back stack (`OverlayBackStack`, `use_overlay_back_stack`, `use_overlay_back`, `use_overlay_back_action`), the OS back gesture `use_os_back` (the iOS edge swipe and Android back, native code behind target-OS gates) and, behind the `toast` feature, `ToastStack`.

The site kit: `use_persisted_theme`, `StatsStrip`, `GalleryFrame`, `GalleryFooter`, the theme wipe (`use_theme_wipe`, `use_theme_follow`) and the `REVEAL_JS`, `NAV_GLIDE_JS` and `SCROLL_FADE_JS` scripts.

The theme model: `ThemeConfig`, `ALLOWED_THEMES`, `COLORBLIND_THEMES` and `display_theme_name`, beside the 31 palettes in `assets/themes.css`, each with a dark and a light block.

## Rules the tests hold

- Every theme block carries `--palette-1` to `--palette-6` and `--color-money`, each readable on the block's background (`tests/themes_palette.rs`).
- Status colors (`--color-success`, `--color-warning`, `--color-error` and their borders) appear only where they mean an outcome; decoration draws from the palette slots (`tests/status_colors.rs`). A host can hold its own stylesheet to the same rule with `status_uses`.
- The theme list and the stylesheet name the same themes, alphabetically.

## Hosts

[zwipe](https://github.com/scadoshi/zwipe) (the app and zwipe.net), [portfolio](https://github.com/scadoshi/portfolio) (scottyfermo.com) and [cairn](https://github.com/scadoshi/cairn) (a counter app). Each pins a commit in its lockfile and moves it on purpose with `cargo update -p orangui`.
