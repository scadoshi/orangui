# orangui

A Dioxus UI kit: themes, panels, nav, charts and the entrance effects shared by my apps and sites. One crate renders and styles the pieces, so the surfaces that use it never drift. The name is orangutan, a call sign of mine, plus GUI.

## Use

```toml
[dependencies]
orangui = { git = "https://github.com/scadoshi/orangui" }
```

Add `features = ["toast"]` for the toast stack. The crate depends on base `dioxus` with no platform feature, so the host picks web, mobile, desktop or fullstack and cargo unifies. The version stays at 0.0.0: hosts pin a commit through their lockfile and move it on purpose, so nothing reads the number yet.

Styling is inlined by the host, in this order: `THEMES_CSS`, then `COMPONENTS_CSS`, then `APP_CSS` (apps) or `SITE_CSS` (websites). That order makes component rules resolve theme variables. It says nothing about where the host's own sheet lands, and the kit does not depend on it: Dioxus appends head elements in hook order, so a host that mounts its `document::Link` after the kit's `document::Style` puts its sheet last, and a prerendered page may put it first. Structure sits in plain class rules and every value a host tunes is a soft default in a zero-specificity `:where()` rule, so a host rule of class specificity or higher overrides a default in either order. A universal or element reset (`* { margin: 0; padding: 0 }`) would zero those defaults, so a host puts it in a cascade layer, which loses to every unlayered rule: `@layer reset { * { margin: 0; padding: 0; } }`.

## What is in it

For every host: `ActionBar`, `Banner`, `Button`, `Chip`, `NavBar`, `NavDropdown`, `PageMeta`, `Panel`, `PasswordInput`, `ThemePicker`; the diagram pieces (`DiagramNode`, `DiagramArrow`, `DiagramDefs`); the entrance figures (`CountUp`, `Decode`, `Figure`, `Replay`); `HeatGrid` for a year of days; the chart helpers `curve`, `area`, `tip_anchor`, `peak_indices`; and `ChartTip`, the chip over a chart's point, with the `.chart-line` draw.

The app kit: `BottomSheet`, `AlertDialog` with its root host (`DialogAction`, `ActionButton`, `DialogSpec`, `use_hosted_dialog`, `DialogHost`, `use_dialog_host`, `DialogHostView`), `PageHeader`, `ThemeSheet`, the hint pieces (`InfoButton`, `HintLine`, `HintBullets`, `HintBullet`, `HintKey`), the OS back stack (`OverlayBackStack`, `use_overlay_back_stack`, `use_overlay_back`, `use_overlay_back_action`), the OS back gesture `use_os_back` (the iOS edge swipe and Android back, native code behind target-OS gates) and, behind the `toast` feature, `ToastStack`.

The site kit: `use_persisted_theme`, `StatsStrip`, `GalleryFrame`, `GalleryFooter`, the theme wipe (`use_theme_wipe`, `use_theme_follow`) and the `REVEAL_JS`, `NAV_GLIDE_JS` and `SCROLL_FADE_JS` scripts.

The theme model: `ThemeConfig`, `ALLOWED_THEMES`, `COLORBLIND_THEMES` and `display_theme_name`, beside the 31 palettes in `assets/themes.css`, each with a dark and a light block.

## Rules the tests hold

- Every theme block carries `--palette-1` to `--palette-6` and `--color-money`, each readable on the block's background (`tests/themes_palette.rs`).
- Status colors (`--color-success`, `--color-warning`, `--color-error` and their borders) appear only where they mean an outcome; decoration draws from the palette slots (`tests/status_colors.rs`). A host can hold its own stylesheet to the same rule with `status_uses`.
- Every value a host tunes sits in a `:where()` rule, and no plain kit rule sets the same property for the same selector (`tests/soft_defaults.rs`).
- The theme list and the stylesheet name the same themes, alphabetically.

## Hosts

[zwipe](https://github.com/scadoshi/zwipe) (the app and zwipe.net), [portfolio](https://github.com/scadoshi/portfolio) (scottyfermo.com) and [cairn](https://github.com/scadoshi/cairn) (a counter app). Each pins a commit in its lockfile and moves it on purpose with `cargo update -p orangui`.
