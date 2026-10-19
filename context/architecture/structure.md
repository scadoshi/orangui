# Structure

One crate, flat. Every module is one component or one hook family, and `lib.rs` is the whole public surface.

```
src/
├── lib.rs               # the exports, the CSS and JS string constants, the kit docs
├── theme.rs             # ThemeConfig, ALLOWED_THEMES, COLORBLIND_THEMES, display_theme_name
├── palette_guard.rs     # status_uses: finds status-color references in a stylesheet, for the gates
│
│   # for every host
├── action_bar.rs        # ActionBar: the button row along a screen's bottom
├── banner.rs            # Banner, BannerStatus
├── button.rs            # Button, ButtonVariant
├── chip.rs              # Chip
├── nav_bar.rs           # NavBar and BRAND_RESET_JS
├── nav_dropdown.rs      # NavDropdown
├── page_meta.rs         # PageMeta, SiteMeta: title, description, Open Graph and Twitter tags
├── panel.rs             # Panel: eyebrow, title, rule, body, actions
├── password_input.rs    # PasswordInput with its Show/Hide toggle
├── theme_picker.rs      # ThemePicker, DarkModeButton, ModeIcon
├── theme_wipe.rs        # use_theme_wipe, use_theme_follow: a theme change sweeps across the page
├── diagram.rs           # DiagramNode, DiagramArrow, DiagramDefs: boxes and arrows in an inline svg
├── entrance.rs          # CountUp, Decode, Figure, Replay: the figures that resolve on arrival
├── chart_math.rs        # curve, area, tip_anchor, peak_indices
├── chart_tip.rs         # ChartTip: the chip over a chart's point
├── heat_grid.rs         # HeatGrid: a year of days, a column per week
│
│   # the app kit, styled by APP_CSS
├── bottom_sheet.rs      # BottomSheet
├── alert_dialog.rs      # AlertDialog, DialogAction, ActionButton, DialogSpec, and the app-root DialogHost that draws them
├── page_header.rs       # PageHeader
├── theme_sheet.rs       # ThemeSheet: every theme as a row, previewed live
├── hint.rs              # InfoButton, HintLine, HintBullets, HintBullet, HintKey
├── overlay_stack.rs     # OverlayBackStack and the use_overlay_back hooks
├── os_back.rs           # use_os_back: the iOS edge swipe and Android back, through the overlay stack to the host's go_back
├── toast_timing.rs      # TOAST_QUICK, TOAST_NORMAL, TOAST_LONG
├── toast_stack.rs       # ToastStack, behind the toast feature
│
│   # the site kit, with the gallery base in SITE_CSS
├── theme_store.rs       # use_persisted_theme: the pick kept in localStorage
├── stats_strip.rs       # StatsStrip: three boxed figures with their source chips
└── gallery.rs           # GalleryFrame, GalleryFooter

assets/
├── themes.css           # 43 themes, a dark and a light block each, gruvbox dark in :root, and the sink and grid mixed from them
├── components.css       # the rules for the components above
├── app.css              # the app shell: screen, page header, bottom sheet, alert dialog, settings lists, hints
├── site.css             # the sites: scroll reveal, gallery, footer
├── toast.css            # the toast stack
├── reveal.js            # elements marked data-reveal fade up as they scroll in
├── nav-glide.js         # nav items slide to their new places when a neighbor changes width
└── scroll-fade.js       # edge fades for browsers without scroll timelines

tests/
├── themes_palette.rs    # the palette gate on themes.css
└── status_colors.rs     # status colors only where they mean an outcome
```

## What is deliberately not here

Anything that knows a host. The crate names no product, no domain model and no route. A Magic card row, a deck chart, a changelog, a product's architecture diagram: those live in the host that owns them, built from the pieces here. If a change needs to know about the host, it belongs in the host.

## Dependencies

- `dioxus` with no platform feature, so any Dioxus target can consume the crate and cargo unifies the host's feature with it.
- `serde` for `ThemeConfig`.
- On the web, `gloo-timers`, `web-sys` and `serde_json` for the entrance timer and local storage; elsewhere `tokio`'s timer.
- On iOS, `objc2` and `tokio`'s channel for the edge-swipe recognizer; on Android, `jni` and `ndk-context` to finish the Activity. Those two are the only target-OS dependencies, for `use_os_back`.
- `dioxus-primitives`, pinned to one commit, only behind the `toast` feature.
