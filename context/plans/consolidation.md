# Consolidation

**Status: PROPOSED.** An audit of what the hosts still implement themselves that duplicates the kit or each other, with a verdict per candidate: move into the kit, drop the host copy, or leave in the host. Nothing here is approved yet; the owner approves or strikes each verdict, and the pass runs after the class rename has landed in every host.

- [ ] Owner approves or strikes each `move` and `drop` below.
- [ ] Every host on orangui in production and through the class rename (see [`host_migration.md`](host_migration.md) and [`class_names.md`](class_names.md)).
- [ ] One orangui commit per candidate group, in the ranked order.
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
