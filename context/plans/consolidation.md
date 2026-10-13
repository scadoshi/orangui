# Consolidation

**Status: KIT DONE 2026-10-13; host passes pending.** The owner approved the ten top candidates. The kit side landed in orangui as one commit per group, in the ranked order; the names it chose are in "Kit names" and each host's mechanical pass is in "Host pass", both at the end. The tables below are the audit as approved, kept as the record of each verdict.

- [x] Owner approves or strikes each `move` and `drop` below.
- [ ] Every host on orangui in production and through the class rename (see [`host_migration.md`](host_migration.md) and [`class_names.md`](class_names.md)).
- [x] One orangui commit per candidate group, in the ranked order.
- [ ] Per host: drop the copies, re-pin, the screenshot diff.

## Summary

The five host sheets hold 1,481 rules (one rule per selector and at-rule context): zwiper `main.css` 482, zite `style.css` 393, portfolio `main.css` 268, cairn `main.css` 188, zwipe-components `magic.css` 150. The kit's four sheets hold 265: `components.css` 188, `app.css` 36, `site.css` 9, `toast.css` 32.

- **Host copies of a kit rule: 14.** 7 under the kit's own selector (5 exact, 2 with one declaration changed) and 7 under another name with the kit's body (zite's `.profile-link` is the nav link rule, all 18 declarations).
- **Rules two or more hosts define and the kit does not: 87 selectors.** 41 identical in every host that has them, 24 close (half the declarations or more the same), 22 diverged. 29 of the 87 are zwiper and zite only, the same repository, and mostly belong in `zwipe-components` rather than here. Another 9 groups share a body under different selectors, such as zite's `footer` and the portfolio's `.footer`.
- **Host overrides of a kit rule with a host value: 28** (portfolio 11, zite 9, cairn 6, zwiper 2). Fine as they are, except 4 that two hosts override the same way, which says the kit's default is wrong.

The top ten, ranked by value: duplicates already drifting between hosts first, then what two hosts both need.

1. **Tinted chips and tags** (`.tag`, `.tag-cN`, `.stat-chip-*` tones): move. The portfolio's `tag-c0` and zite's `tag-c1` are both palette 1, the radius is 0.6rem against 0.4rem, the border mix 40% against 35%. All four hosts, about 170 css lines and 40 Rust lines.
2. **Alert dialog and the dialog host**: move. zwiper's and cairn's copies have drifted (title color, padding, rule margins, no close animation in cairn). zwiper and cairn, about 270 css lines and 290 Rust lines.
3. **Entrance keyframes** (`content-enter`, `content-enter-delayed`, `chip-in`): move. Four copies, two without the reduced-motion guard, two delayed timings. All four hosts, about 55 css lines.
4. **Reveal rule** (`.reveal-pending`, `.reveal-in`): move. The kit ships `reveal.js` but no rule; the sites agree, cairn does not. zite, portfolio, cairn, about 30 css lines.
5. **Chart tip and line draw** (`*-tip`, `tip-start`, `tip-end`, `chart-draw`): move. The kit's `tip_anchor` emits the classes and styles none; four tip copies, two `chart-draw` keyframes that disagree. portfolio and cairn, about 150 css lines and 25 Rust lines.
6. **Sink and grid variables** (`--bg-sink`, `--grid-line`, `--grid-size`): move. The kit's sheets read them and define none; all four hosts define the same values. All four hosts, about 20 css lines.
7. **Settings list group** (`profile-sections`, `profile-list`, `card-header`, `card-title`, `settings-row-value`, `pref-note`): move. zwiper and cairn, near-identical with small drift. About 140 css lines.
8. **Host copies of kit rules**: drop. zwiper's password rules and TextInput's password branch, zite's `.info-button`, the portfolio's `:root`, and four more. All four hosts, about 140 css lines and 25 Rust lines.
9. **Site footer** (`footer`, `.footer`, `.fan-content-notice`, `.footer-built-text`): move. The same rule under two names on the two sites. About 40 css lines.
10. **NavBar persistent slot and the store pill** (`.nav-stores-persistent`, `.nav-link-store`, `.store-link`): move. NavBar has the `persistent` slot and no rule for it; both sites style it alike, and the pill has drifted. About 110 css lines.

**Total blast radius** for the top ten: about 1,100 host stylesheet lines and 350 host Rust lines deleted, across all four hosts, against roughly 450 lines added to the kit's sheets and two small components (the dialog and its host).

**How a pass would run.** The same shape as the class rename. One orangui commit per candidate group, in the ranked order above, each adding the rule or component under the names in this document and saying so in the message. Each host branch then drops its copies in one commit per group, re-pins with `cargo update -p orangui`, and builds and lints. The gate is the screenshot diff used for the migration: nothing visible moves, except where a row below picks one side of a drift, and then the diff shows only that. A host whose diff is not clean stays on the previous pin until it is.

The counts come from the hosts' `orangui` worktrees and orangui `main` on 2026-10-12, parsed by a script that splits selector lists, normalizes at-rule contexts and compares declarations; "lines" are the rules' declarations plus their braces, so they are estimates.

## Stylesheets

### Drifting between hosts

| Selector | Where defined | Verdict | Why |
|---|---|---|---|
| `.tag`, `.tag a`, `.tag:nth-child(6n + N)`, `.tag-c0` to `.tag-c6` | zite 14 rules (71 lines), portfolio 8 (41) | move | one tinted tag in `components.css` with `tag-p1` to `tag-p6` named after the slot; the two sites number from 0 and 1 for the same slot, and radius and border mix differ |
| `.stat-chip-bad`, `-format`, `-zone`, `-tag`, `-power`, `-other` (zwiper); `-derived`, `-short`, `-met`, `-trend`, `-goal` (cairn); `.title-tag.tag` (portfolio) | zwiper 6 (30), zite 12 (60, scoped copies), cairn 6 (30), portfolio 1 (5) | move | tone modifiers on the kit's `.stat-chip`: `-muted`, `-success`, `-warning`, `-error`, `-money`, `-p1` to `-p6`; hosts map their domain names in Rust. zite mixes the tints into `--bg-primary` where zwiper mixes into transparent. The status tones need entries in the `status_colors` allowlist, since they mean an outcome |
| `.alert-dialog`, `-overlay`, `-title`, `-description`, `-actions`, `-action`, `-action-danger`, `-cancel`, `.dialog-rule`, `@keyframes alert-animate-in`, `-out` | zwiper `alert-dialog.css` (178), cairn (95) | move | `app.css`, with the dialog component; title is `--accent-tertiary` in zwiper and `--accent-primary` in cairn, padding 2rem against 1.25rem 1.5rem, cairn has no close animation and no focus ring |
| `@keyframes content-enter`, `.content-enter`, `.content-enter-delayed`, `@keyframes chip-in` | zwiper 4 (14), zite 3 (11), portfolio 4 (14), cairn 5 (16) | move | `components.css`, with the reduced-motion guard that zwiper and zite lack; the delayed variant is 0.3s after 0.15s in zwiper and 0.4s after 0.45s in cairn |
| `.reveal-pending`, `.reveal-in`, `@keyframes reveal-in` | zite 2 (9), portfolio 2 (9), cairn 4 (14) | move | the sites agree (14px, 0.55s transition) under different scopes; cairn uses 12px and a 0.45s animation with its own `entrance.js`. The kit ships `reveal.js`, so the rule belongs beside it |
| `.heatmap-tip`, `.commits-tip`, `.bench-tip`, `.heat-tip` with `.tip-start`, `.tip-end`, `@keyframes tip-in` | portfolio 13 (64), cairn 4 (18) | move | one `.chart-tip` in `components.css`; `tip_anchor` already emits `tip-start` and `tip-end`. The entrance is `tip-in` on the portfolio and `content-enter` in cairn |
| `@keyframes chart-draw`, `.commits-line`, `.bench-line`, `.chart-line` | portfolio 12 (46), cairn 4 (21) | move | one `.chart-line` with the draw, keyed to `pathLength="1"`; cairn uses 1000 and a `to`-only keyframe |
| `.collapsible`, `.collapsible.open`, `.collapsible-inner`, `@keyframes collapsible-open-guard` | zwiper 5 (35), zite 3 (27) | move | a generic grid-rows collapse; zite's copy lacks zwiper's overflow guard. `magic.css` repeats the pattern as `.keyword-reveal` and `.card-row-collapse-inner` |
| `.store-link`, `.store-link:hover` | zite, portfolio (about 40) | move | the outlined pill; zite adds `white-space`, `cursor` and a shadow, the portfolio a `:visited` twin |
| `.nav-logo`, `.nav-brand:hover .nav-logo` | zite, portfolio (about 20) | move | 0.35rem against 0.4rem; the rest is the same |
| `.label` | zwiper, cairn (about 15) | move | `app.css`; zwiper adds `font-weight: 400`, nothing else differs |
| `.card-bullets`, `.card-bullets li`, `li::before` | zite 4 (22), portfolio 3 (20) | leave | a `›` list; sizes and spacing differ by page, and `magic.css` has its own as `.changelog-bullets` |
| `.logo` | zwiper, zite, portfolio, cairn (about 50) | leave | the ASCII mark; five declarations shared, the size and margins are each host's |

### Two hosts need it, the kit lacks it

| Selector | Where defined | Verdict | Why |
|---|---|---|---|
| `--bg-sink`, `--grid-line`, `--grid-size`, the light-theme `--bg-sink` | zwiper `.app-shell`, zite `body`, portfolio `.theme-wrapper`, cairn `.screen` (about 5 lines each) | move | `components.css` reads `--bg-sink` and `app.css` reads both; every host defines the same values on its theme root. The kit defines them on the `theme-*` class |
| `.profile-sections`, `.profile-list` (zwiper's `.info-list`), `.card-header`, `.card-title`, `.settings-row-value`, `.pref-note` | zwiper 11 (89), cairn 6 (53) | move | `app.css` beside `.settings-row`; widths differ by 4rem against 2rem, `.pref-note` by size |
| `footer`, `.footer`, `.footer-copy`, `.fan-content-notice`, `.footer-built-text` | zite 2 (19), portfolio 3 (21) | move | `site.css` as `.site-footer` and `.site-footer-note`; the same declarations under two names, margin-top differs |
| `.nav-stores-persistent`, its 60rem query, `.nav-panel .nav-link-store` | zite 5, portfolio 5 (about 65) | move | NavBar's `persistent` slot, renamed `nav-persistent`; identical on both sites |
| `*`, `input, textarea, select, button`, `*::-webkit-scrollbar`, `.screen`, `.text-center` | zwiper 8, cairn 8 (about 85) | move | the app shell base in `app.css`; `.screen` differs only by the sink variables cairn sets on it |
| `::selection`, `.ext`, `a:hover .ext`, the reduced-motion `.ext`, `.sr-only` | zite 5 (26), portfolio 5 (26) | move | `site.css`; identical |
| `.hero-head`, `.hero-head .logo`, its 62rem query | zite 3, portfolio 3 (about 30) | move | `site.css`: a logo beside a StatsStrip, stacked under 62rem; the portfolio aligns left |
| `.label-hint-anchor`, `.label-hint-anchor .info-button`, cairn's `.label-with-hint` | zwiper 3, zite 2, cairn 2 (about 40) | move | `app.css`: a label with a `?` hanging off it; zwiper nudges the button 0.125rem up, cairn centers a flex row |
| `.chip-row` | zwiper, zite, cairn | leave | gaps and margins differ per screen |
| `.info-row`, `-label`, `-value`, `-tags`, `-stacked` | zwiper, zite | leave | zwipe only; zwiper's `.info-row` is the kit's `.settings-row` (see the drop table), the rest can go to `magic.css` |
| `.privacy-*`, `.outcome-success`, `.card-group`/`.sd-group`, `.deck-featured-*`/`.sd-cz-*` | zwiper, zite | leave | zwipe only; candidates for `magic.css`, not the kit |
| `.card-detail-name`, `.card-detail-oracle`, `.card-row-rule-muted` | zite, `magic.css` | leave | Magic pieces, already diverged on purpose |
| `.spinner`, `@keyframes spin` | zwiper, zite | leave | two different spinners |
| `@font-face` (JetBrains Mono) | zite 4, portfolio 4, cairn and zwiper inline from Rust | leave | every host ships the font, but the sites use fontsource subsets and the apps the full font for the logo's block glyphs; moving it means the kit ships binaries |

### Host copies of a kit rule

| Selector | Where defined | Verdict | Why |
|---|---|---|---|
| `.password-input-wrapper`, `.password-input-wrapper .input-password`, `.password-toggle-btn`, `:hover` | zwiper (24) | drop | the kit's rules; only the toggle's `translateY(calc(-50% - 0.5rem))` stays, as an override |
| `.info-button` | zite (17) | drop | 13 of 15 declarations are the kit's; the other two go to the kit (next table) |
| `:root` | portfolio (19) | drop | all 19 variables are the kit's gruvbox dark, `--text-primary` included (`#ebdbb2`, published gruvbox fg1) |
| `.hero-rule` | portfolio (5) | drop | the kit's `.panel-rule`, and the hero is a Panel |
| `.profile-link`, `.profile-links` | zite (about 25) | drop | the kit's nav link pill and `.panel-actions`; zite can emit the kit classes |
| `.info-row`, `.info-row-label` | zwiper (about 15) | drop | the kit's `.settings-row` and `.settings-row-label`, same body |
| `.home-hero .stats-strip`, `.stat`, `.stat-num`, `.stat-label` | zwiper (43) | drop | restates the kit's StatsStrip; only the sizes stay |
| reduced-motion `.heat-cell`, `.heat-halo` | cairn (4) | drop | the kit has the same guard |

### Overrides two hosts make alike

28 overrides, 24 of them each host's own business. These 4 say the kit's default should change.

| Selector | Where defined | Verdict | Why |
|---|---|---|---|
| `.nav-wrapper { --nav-max-width: none }` | zite, portfolio | move | both hosts that use NavBar turn the cap off |
| `.stats-strip .stat` padding, `.stat-num` 1.5rem, `.stat-label` 0.75rem and 0.06em | zite, portfolio, zwiper | move | the kit's figure has no padding or size; every host sets them, the sites to the same values |
| `.gallery-nav` background and opacity | zite, portfolio | move | both give it `--bg-primary`; the opacity drifts (0.9 against 0.75), so the kit picks one |
| `.info-button` `cursor: pointer`, `font-family` | zite, cairn | move | a button that shows no pointer is a kit bug |

## Components

**Portfolio's project card and Panel.** `ProjectCard` is already a Panel with eyebrow, status and actions; what is left is its body (`card-summary`, `card-bullets`, `card-impact`, `Measured`), which is the portfolio's data. Leave.

**Portfolio's heatmap, cairn's heatmap and HeatGrid.** Both already draw through `HeatGrid`, `heat_span`, `peak_indices` and `tip_anchor`. What remains alike is the layout from dates to columns and month labels (the portfolio's `days_from_civil` and `layout`, cairn's `heat::heat_grid` on chrono with a week-start preference, both with a 3-column label gap) and the tap chip. The layout needs a date model, so it stays in the hosts; the chip moves with the chart tip below.

**Cairn's line chart, the portfolio's commits and benchmark charts, and the chart helpers.** All three use the kit's `curve`; the portfolio adds `area`. The duplicates are the tip (`struct Tip { text, left, top }` is word for word in the portfolio's `chart.rs` and cairn's `heatmap.rs`), its four stylesheet copies, and the line-draw keyframe. Move a `ChartTip` component that takes the text and position and emits `span.chart-tip` with `tip_anchor`, plus the `.chart-line` draw rule. The axis helpers (the portfolio's `ceiling` and `ticks`, cairn's label stride) differ in purpose and stay.

**zwiper's alert dialog and cairn's.** cairn's says "zwiper's look" and is a rewrite: zwiper wraps `dioxus-primitives` (133 lines) with a backdrop and `use_overlay_back_action`, cairn draws a `DialogSpec` through an app-root `DialogHost` (153 lines) so the dim escapes the scroll container, which zwiper solves the same way for hints with `HintHost`. The stylesheets have drifted (see above). Worth a shared version: cairn's host model has no primitives dependency and fits the kit's app kit, taking the title, body, an optional confirm and an optional link as props. zwiper keeps its hint topics and support content and draws them through it.

**zite's banner stack.** zite wraps the kit's `Banner` in `div.banner-stack` by hand, as `banner.rs` documents. A `BannerStack` wrapper would save one line. Leave.

**The sites' hero rules.** `hero-head` and the logo beside the StatsStrip are the same layout on both sites and move to `site.css` (above). `hero-rule` is the kit's `panel-rule` on the portfolio and an animated copy on zite, so the portfolio drops it and zite keeps its own. `hero-chip` and `hero-card` are zite only. Leave those.

**Tags and `hl-*` highlights.** The tags move as one tinted family (above). `hl-p2`, `hl-p3`, `hl-p5` and `hl-tertiary` exist only on the portfolio's home page. Leave.

**Status pills.** The kit's `status-tag` with `status-done` and `status-doing` serves Panel and Banner, and the portfolio uses it through `BannerStatus`. zwiper's `stat-chip-bad` and cairn's `stat-chip-met` have the status pills' exact bodies on a different chip; they fold into the stat-chip tones. zite's `.status-message` is a form result box. Leave that.

**zwiper's stat-chip modifiers.** Six tones over the kit's `.stat-chip`, repeated by zite under two scopes with a different mix. The domain names (`format`, `zone`, `power`) are zwipe's; the tones are generic. Move the tones, keep the names as aliases in the hosts or map them in Rust.

**zwiper's screen header and support button, and PageHeader.** `ScreenHeader` is 26 lines that put zwiper's `SupportButton` in PageHeader's `leading` slot; cairn does the same with `AboutButton`. Both buttons use the documented `page-header-support` hook and open host content. Leave.

**Form inputs.** zwiper's `TextInput` carries a copy of PasswordInput's markup in its password branch; it should call `PasswordInput` with `class: "input input-error"` (drop, about 25 lines). The rest of `TextInput` (label, `?` hint, error line) is a labeled field zwiper and cairn both build by hand; the `.label` and label-hint rules move (above). A kit `Field` component is not worth it yet: the hint each host opens is its own.

**Toasts in cairn and ToastStack.** cairn already renders the kit's `ToastStack` with `TOAST_CSS` and the timing constants, and its sheet has no toast rules. Nothing left.

**Meta handling.** zite and the portfolio each wrap `PageMeta` in a 20-line component that bakes in their `SiteMeta`. The kit could read `SiteMeta` from context so pages call `PageMeta` directly. Nice to have, two files.

**Footers.** zite's `Footer` and the portfolio's are different text in the same frame; the frame's rules move to `site.css` (above). A `SiteFooter` component taking children would add little over the class. Leave the components.

**The reveal and content-enter animations.** The kit owns `reveal.js` and the entrance figures but no rule for either; the rules move (above). cairn keeps `entrance.js`, which reveals and rolls figures inside the page because iOS holds the app's messages until a scroll settles; it should switch to the kit's rule and class names once those exist.

**The back handler: done.** zwiper's and cairn's `BackHandlerLayout` (194 and 174 lines, cairn's "ported from zwiper") differed in the route type, the Android event name, the Objective-C class name and whether failures were logged. The owner approved the native code, and the bridge is now `use_os_back` in `src/os_back.rs`, logging failures as zwiper did. Each host keeps a layout of a few lines that passes its Android event name, its WKWebView from `dioxus/mobile` and its router's `go_back`, and keeps the `MainActivity` patch in its build scripts. The hosts take it on their `orangui` branches.

## Kit names

What each group landed as in orangui, under the [`class_names.md`](class_names.md) convention: a family prefix named after the component or role, modifiers after the prefix.

| Group | Old host names | Kit name | Where |
|---|---|---|---|
| 1 | `.tag` (zite, portfolio) | `.tag` | `components.css` |
| 1 | portfolio `tag-c0` to `tag-c4`, zite `tag-c1` to `tag-c6`, zite's `.tag:nth-child(6n + N)` cycle | `tag-p1` to `tag-p6`, the number is the palette slot | `components.css` |
| 1 | portfolio `.title-tag.tag` gray | `tag-muted` | `components.css` |
| 1 | `.stat-chip` (was in `app.css`) | `.stat-chip`, now in `components.css` so the sites get it too | `components.css` |
| 1 | zwiper `stat-chip-bad`, `-format`, `-zone`, `-tag`, `-power`, `-other`; zite `stat-chip-price`; cairn `stat-chip-derived`, `-short`, `-met`, `-trend`, `-goal` | `stat-chip-` and `tag-` tones: `-p1` to `-p6`, `-muted`, `-money`, `-success`, `-warning`, `-error` | `components.css` |
| 1 | zite's tints mixed into `--bg-primary` | `--tone-base` set on an ancestor (transparent by default); `--tone` is the color, `--tone-edge` the outline mix | `components.css` |
| 2 | zwiper `AlertDialogRoot` and its parts (dioxus-primitives), cairn `DialogHost`, `DialogHostView`, `DialogSpec`, `use_hosted_dialog`, `ConfirmDialog` | `AlertDialog`, `DialogAction` (`Dismiss`, `Confirm`, `Link`), `DialogSpec`, `use_hosted_dialog`, `DialogHost`, `use_dialog_host`, `DialogHostView` | `src/alert_dialog.rs` |
| 2 | `.alert-dialog*` classes | unchanged: `alert-dialog-overlay`, `alert-dialog`, `-title`, `-description`, `-actions`, `-cancel`, `-action`, `-action-danger`; new `alert-dialog-long` marks a long body for the edge fade | `app.css` |
| 2 | `.dialog-rule` | `alert-dialog-rule` | `app.css` |
| 2 | `@keyframes alert-animate-in`, `-out` | `alert-dialog-in`, `alert-dialog-out` | `app.css` |
| 3 | `content-enter`, `content-enter-delayed`, `chip-in` keyframes; zite and portfolio `.tag-swap`, zwiper `.ub-exception-chip` animations | `.content-enter`, `.content-enter-delayed`, `.chip-in` (0.3s ease-out, the delayed one 0.15s later, none under reduced motion) | `components.css` |
| 4 | `.reveal-pending`, `.reveal-in` | unchanged, styled by the kit (14px, 0.55s transition), plus the paused `.heat-cell`, `.heat-halo`, `.chart-line` until the reveal starts | `site.css` |
| 5 | portfolio `.heatmap-tip`, `.commits-tip`, `.bench-tip`; cairn `.heat-tip`; `@keyframes tip-in` | `ChartTip` component emitting `span.chart-tip` with `tip-start`/`tip-end`; `@keyframes chart-tip-in` | `src/chart_tip.rs`, `components.css` |
| 5 | portfolio `.commits-line`, `.bench-line` draw; cairn `.chart-line` draw; `@keyframes chart-draw` | `.chart-line` on a path with `pathLength="1"`; `@keyframes chart-draw` | `components.css` |
| 6 | `--bg-sink`, `--grid-line`, `--grid-size` on each host's theme root | defined in `themes.css` on `:root` and every element carrying a theme class, the lighter sink on `-light` | `themes.css` |
| 7 | `profile-sections` | `settings-sections` | `app.css` |
| 7 | `profile-list`, zwiper `info-list` | `settings-list` | `app.css` |
| 7 | `card-header` | `settings-list-header` | `app.css` |
| 7 | `card-title` | `settings-list-title` | `app.css` |
| 7 | `settings-row-value`, `pref-note` | unchanged | `app.css` |
| 8 | `.info-button` (was in `app.css`) | `.info-button` in `components.css`, with `cursor: pointer` and the font | `components.css` |
| 8 | zwiper's toggle `translateY(calc(-50% - 0.5rem))` | the password wrapper carries the field's 1rem bottom margin, so the toggle centers at -50% | `app.css` |
| 8 | the sites' `--nav-max-width: none`, StatsStrip padding and sizes, `.gallery-nav` background | kit defaults: `--nav-max-width` falls back to `none`; `.stat` padding `0.7rem 0.35rem 0.6rem`, `.stat-num` `clamp(0.9rem, 3.8vw, 1.5rem)`, `.stat-label` `clamp(0.6rem, 2.4vw, 0.75rem)` at `0.06em`; `.gallery-nav` on `--bg-primary` at 0.9 opacity, 1 on hover | `components.css`, `site.css` |
| 9 | zite `footer`, portfolio `.footer` | `.site-footer` (`margin-top: auto`) | `site.css` |
| 9 | zite `.fan-content-notice`, portfolio `.footer-built-text` | `.site-footer-note` | `site.css` |
| 10 | `.nav-stores-persistent` | `div.nav-persistent`, now emitted by NavBar around the `persistent` slot, with `data-nav-glide` | `components.css`, `src/nav_bar.rs` |
| 10 | `.store-link` | `.nav-cta` | `components.css` |
| 10 | `li.nav-link-store` | `li.nav-cta-item` | `components.css` |

No `SiteFooter` component: the class is enough, as the plan judged.

## Host pass

Per host, what to delete and what to rename, in group order. Re-pin with `cargo update -p orangui` first; renames are whole-token substitutions over the host's stylesheet and Rust sources. "Visible" marks the places where the kit picked the other side of a drift, so the screenshot diff will show them; the line after says how to keep the old look if the owner wants it.

### zwiper (zwipe `zwiper/`)

1. Tags. Rename in Rust: `stat-chip-bad` to `stat-chip-warning`; `stat-chip-format` and `stat-chip-zone` to `stat-chip-p3`; `stat-chip-tag` to `stat-chip-muted`; `stat-chip-power` and `stat-chip-other` to `stat-chip-p5`. Delete the six `.stat-chip-*` rules from `assets/main.css`.
2. Dialog. Delete `assets/alert-dialog.css`, the `ALERT_DIALOG_CSS` asset and its `document::Link` in `src/bin/zwipe.rs`, and `src/lib/inbound/components/alert_dialog/`. At the app root: `let dialogs = use_dialog_host(); use_context_provider(|| dialogs);` and mount `DialogHostView {}` beside the router (where `HintHost {}` sits). Each dialog built from `AlertDialogRoot`/`Content`/`Title`/`Description`/`Actions`/`Cancel`/`Action` becomes `AlertDialog { open, title, actions: vec![DialogAction::Dismiss("Cancel".into()), DialogAction::Confirm { label, danger: true, on_confirm }], {body} }`; a buy link is `DialogAction::Link { label, href }`; a hint is `actions: vec![DialogAction::Dismiss("Got it".into())]`. `AlertDialog` registers the overlay back itself, so drop the dialog's own `use_overlay_back`/`use_overlay_back_action`. The body no longer carries `alert-dialog-description` (the kit wraps it). Rename `dialog-rule` to `alert-dialog-rule` if any remains. Mark the card-rules body with `alert-dialog-long` so it keeps the edge fade (the kit's `:has()` covers `.hint-line` and `.hint-bullets` already). In `tests/status_colors.rs`, remove `.alert-dialog-action-danger` and the `alert-dialog.css` include. Drop `dioxus-primitives` if nothing else uses it. Visible: a keyboard focus ring on the dialog buttons (accent-secondary outline) where zwiper had none; the dialog now fades out on close.
3. Entrance. Delete `@keyframes content-enter`, `.content-enter`, `.content-enter-delayed`, `@keyframes chip-in`, the `.ub-exception-chip` animation and its reduced-motion block. In Rust, add `chip-in` to the element that carries `ub-exception-chip`.
6. Sink and grid. Delete the `.app-shell` block that sets `--grid-line`, `--grid-size`, `--bg-sink`, and `.app-shell[class$="-light"]`.
7. Settings list. Rename `profile-sections` to `settings-sections`; `profile-list` and `info-list` to `settings-list`; `card-header` to `settings-list-header`; `card-title` to `settings-list-title` (the compound `.card-title.update-required-title` becomes `.settings-list-title.update-required-title`). Delete the rules for `.info-list`, `.profile-sections`, `.profile-list`, `.card-header`, `.card-title`, `.settings-row-value`, `.pref-note`. Visible: the column is `calc(100% - 2rem)` wide against zwiper's 4rem; to keep it, add `.settings-sections { width: calc(100% - 4rem); }`. A long unbroken value now wraps at need (`overflow-wrap: anywhere`) rather than at any character (`word-break: break-all`).
8. Drops. Delete `.password-input-wrapper`, `.password-input-wrapper .input-password`, `.password-toggle-btn` (including the translate override) and `.password-toggle-btn:hover`. In `TextInput`, the password branch calls `PasswordInput` with `class: "input input-error"` instead of repeating its markup. Rename `info-row` to `settings-row` and `info-row-label` to `settings-row-label` (zwiper only; zite's `info-row-*` stay) and delete those two rules. Replace the `.home-hero .stats-strip`, `.stat`, `.stat-num`, `.stat-label` rules with only the sizes: `.home-hero .stat { padding: 0.6rem 0.25rem 0.5rem; }`, `.home-hero .stat-num { font-size: 1.25rem; }`, `.home-hero .stat-label { font-size: 0.65rem; letter-spacing: 0.04em; text-align: center; }`.

### zite (zwipe `zite/`)

1. Tags. Rename `tag-c1` to `tag-c6` to `tag-p1` to `tag-p6` (`src/pages/guides/mod.rs`, `CLASSES`). Delete `.tag`, the six `.tag:nth-child(6n + N)`, the six `.tag.tag-cN`, `a.tag`, `.stats-figures .stats-source .tag`, and `.tag a, .stat-chip a` from the `.tag a, .stat-chip a, .chip a` group (keep `.chip a`). Keep `a.tag:hover`. Delete every scoped `.sd-header-meta .stat-chip*` / `.info-row-tags .stat-chip*` rule (seven pairs) and add `.sd-header-meta, .info-row-tags { --tone-base: var(--bg-primary); }`. In `src/pages/shared_deck.rs`: the bare `stat-chip` (card count) becomes `stat-chip stat-chip-p5`; `stat-chip-format` and `stat-chip-zone` to `stat-chip-p3`; `stat-chip-power` and `stat-chip-other` to `stat-chip-p5`; `stat-chip-tag` to `stat-chip-muted`; `stat-chip-price` to `stat-chip-money`. Visible: tag radius 0.6rem against 0.4rem. A bare `.tag` now reads in `--accent-primary` (the source row already did); any bare tag that relied on the position cycle takes an explicit `tag-pN`.
3. Entrance. Delete `@keyframes content-enter`, `.content-enter`, `@keyframes chip-in`, `.stats-figures .stats-source .tag-swap` and its reduced-motion block. In `src/components/stats_strip.rs`, `tag tag-swap` becomes `tag chip-in`. Visible: the swap runs 0.3s against 0.4s.
4. Reveal. Delete `body .reveal-pending` and `body .reveal-pending.reveal-in`.
6. Sink and grid. Delete `--grid-line`, `--grid-size`, `--bg-sink` from the `body` rule (keep `--grid-line-major` and the rest) and delete `body[class$="-light"]`.
8. Drops. Delete `.info-button` (keep `.info-button:hover`). Rename `profile-links` to `panel-actions` and `profile-link` to `panel-action` and delete both rules; check the about page against the screenshot, since `.panel-action` is the kit's pill. Delete `.nav-wrapper { --nav-max-width: none; }`, the `.stats-strip .stat`, `.stat-num`, `.stat-label` overrides (the kit has the same clamps now), and `background-color` and `opacity` from `.gallery-nav` (keep its transition, `z-index` and hover).
9. Footer. The `Footer` component's `footer` gets `class: "site-footer content-enter-delayed"`; `fan-content-notice` becomes `site-footer-note`. Replace the `footer` rule with `.site-footer { margin-top: 3.75rem; }` and delete `.fan-content-notice`.
10. Nav. In `Nav`, the `persistent` slot passes the two anchors directly (NavBar now wraps them in `div.nav-persistent` with `data-nav-glide`); rename `store-link` to `nav-cta` and `nav-link-store` to `nav-cta-item`. Delete `.nav-stores-persistent`, `.store-link, .nav-links a.store-link`, its hover block and the `@media (max-width: 60rem)` block with the three persistent rules.

### portfolio

1. Tags. Rename `tag-cN` to `tag-p(N+1)`: `tag tag-c0` to `tag tag-p1` (fleet, home, the three tips before step 5), `measured.rs` `tag-c{i % 5}` to `tag-p{i % 5 + 1}`. In `src/pages/detail.rs`, `tag title-tag tag-c{i % 4}` becomes `tag title-tag tag-muted`. Delete `.tag`, `.tag-c0` to `.tag-c4`, `.title-tag.tag`, `.tag a`. Visible: outlines mix 35% against 40%.
3. Entrance. Delete `@keyframes content-enter`, `.content-enter`, `@keyframes chip-in`, `.stats-source .tag-swap` and its reduced-motion block, and `.content-enter` from the reduced-motion list near the reveal rule. In `src/pages/home.rs`, `tag tag-c0 tag-swap` becomes `tag tag-p1 chip-in`. Visible: the swap runs 0.3s against 0.4s.
4. Reveal. Delete `.theme-wrapper .reveal-pending` and `.theme-wrapper .reveal-pending.reveal-in`. In the paused list, drop `.heat-cell`, `.commits-line` and `.bench-line` (the kit pauses `.heat-cell` and `.chart-line`).
5. Charts. The three tip spans become `ChartTip { text: tip.text, left: tip.left, top: tip.top, class: "tag tag-p1" }` (heatmap, commits, benchmark); delete `.heatmap-tip, .commits-tip, .bench-tip`, their `.tip-start` and `.tip-end` rules, `@keyframes tip-in` and its reduced-motion block. The `Tip` struct in `chart.rs` can stay or go. Add `chart-line` to the `commits-line` and `bench-line` paths (they already carry `path_length: "1"`); delete `@keyframes chart-draw`, and the `stroke-dasharray` and `animation` declarations from `.heat-grid .commits-line`, `.commits-chart .commits-line` and `.bench-chart .bench-line`, plus those selectors in the reduced-motion blocks. Keep `.bench-line { stroke: currentColor; }` as the host's color. Visible: line caps are round.
6. Sink and grid. Delete `--grid-line`, `--grid-size`, `--bg-sink` from `.theme-wrapper` (keep `--grid-line-major`) and delete `.theme-wrapper[class$="-light"]`.
8. Drops. Delete `:root` (the kit's gruvbox dark). `hr.hero-rule` in `src/pages/home.rs` becomes `hr.panel-rule`; delete `.hero-rule`. Delete `.nav-wrapper { --nav-max-width: none; }`, the `.stats-strip .stat`, `.stat-num`, `.stat-label` overrides, and `background` and `opacity` from `.gallery-nav` (keep its transition and the hover's `--bg-sink`). Visible: `.stat-num` shrinks on a phone (the kit's clamp tops out at the same 1.5rem); the gallery buttons rest at 0.9 opacity against 0.75.
9. Footer. In `src/components/footer.rs`, `footer.footer` becomes `footer.site-footer` and `footer-built-text` becomes `site-footer-note`; delete `.footer` and `.footer-built-text`. Keep `.footer-copy`, its links and `.footer-built-repo`.
10. Nav. In `src/components/navbar.rs`, the `persistent` slot passes the anchors directly; rename `store-link` to `nav-cta` and `nav-link-store` to `nav-cta-item`. Delete `.nav-stores-persistent`, the `.store-link` group with its `:visited` twins, its hover block and the `@media (max-width: 60rem)` block with the three persistent rules.

### cairn

1. Tags. Rename in Rust: `stat-chip-derived` to `stat-chip-muted`, `stat-chip-short` to `stat-chip-error`, `stat-chip-met` to `stat-chip-success`, `stat-chip-trend` to `stat-chip-p2`, `stat-chip-goal` to `stat-chip-p4`. Delete those five rules, and their entries in `tests/status_colors.rs` if listed.
2. Dialog. Delete `src/lib/inbound/ui/components/dialog_host.rs`. At the root (`ui/mod.rs`), `components::dialog_host::DialogHost(use_signal(|| None))` becomes `use_dialog_host()` and `DialogHostView {}` comes from orangui. `ConfirmDialog` becomes `AlertDialog { open, title, actions: vec![DialogAction::Dismiss("Cancel".into()), DialogAction::Confirm { label: confirm_label, danger: true, on_confirm }], p { "{body}" } }` and drops its own `use_overlay_back(open)`. `HintDialog` and the about dialog use `use_hosted_dialog(open, || DialogSpec { title, body, actions })` or `AlertDialog`, with `DialogAction::Dismiss("Got it".into())` and, for the about link, `DialogAction::Link { label, href }`. Bodies drop `alert-dialog-description` (keep `hint-body`). Delete the whole "alert dialog" block in `assets/main.css` (`.alert-dialog-overlay` through `.alert-dialog-action-danger`, with `@keyframes alert-animate-in`) and `.alert-dialog-action-danger` from `tests/status_colors.rs`. Visible: the title is `--accent-tertiary` against `--accent-primary`, the padding 2rem against 1.25rem 1.5rem, the actions sit 0.5rem down against 0.25rem, and the dialog fades out on close. To keep cairn's look: `.alert-dialog { --dialog-pad-x: 1.5rem; padding: 1.25rem var(--dialog-pad-x); } .alert-dialog-title { color: var(--accent-primary); }`.
3. Entrance. Delete `@keyframes content-enter`, `.content-enter`, `.content-enter-delayed`, and both from the reduced-motion block. Visible: the delayed entrance runs 0.3s after 0.15s against 0.4s after 0.45s; to keep it, `.content-enter-delayed { animation-duration: 0.4s; animation-delay: 0.45s; }`.
4. Reveal. Nothing: cairn loads `APP_CSS`, not `SITE_CSS`, and keeps `entrance.js` with its own `.reveal-pending` and `.reveal-in`.
5. Charts. The tip span `stat-chip heat-tip {tip_anchor(t.left)}` becomes `ChartTip { text: t.text, left: t.left, top: t.top, class: "stat-chip" }`; delete `.heat-tip`, `.heat-tip.tip-start`, `.heat-tip.tip-end` and `.heat-tip` in the reduced-motion block. In `line_chart.rs`, `path_length: "1000"` becomes `"1"` on both paths; delete `stroke-dasharray`, `stroke-dashoffset` and `animation` from `.chart-line`, `@keyframes chart-draw`, and the `.chart-line, .chart-overlay` reduced-motion block. Keep `.chart-line { stroke-width: 1.6; }` and `.chart-overlay`'s stroke, duration and delay. Visible: the line draws over 1.1s against 0.9s.
6. Sink and grid. Delete `--grid-line`, `--grid-size`, `--bg-sink` from `.screen` and delete `.screen[class$="-light"]`.
7. Settings list. Rename `profile-sections` to `settings-sections`, `profile-list` to `settings-list`, `card-header` to `settings-list-header` (`.card-header.home-hero-head` becomes `.settings-list-header.home-hero-head`; `card-header-flow` stays a host modifier), `card-title` to `settings-list-title`. Delete `.profile-sections`, `.profile-list`, `.card-header`, `.card-title`, `.profile-list > .action-bar`, `.settings-row-value`, `.pref-note`. Visible: `.pref-note` is 0.75rem with a 0.6rem bottom margin against 0.85rem and none; to keep it, `.pref-note { font-size: 0.85rem; margin: 0; }`.
8. Drops. Delete the `.info-button` override (font and cursor are the kit's now) and `.heat-cell`, `.heat-halo` from the reduced-motion block.
