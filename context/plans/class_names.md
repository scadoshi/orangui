# Class names

**Status: APPROVED 2026-10-12.** Every verdict approved, with two changes: the Util button is named after the bar it sits in, `btn-action` with `ButtonVariant::Action`, and `stepper-btn` stays in the kit as a generic rule, so only the two architecture rules move. An audit of every CSS class the kit defines or emits, with a verdict per class, for the owner to approve name by name. Nothing is renamed yet. The rename pass runs after every host is on orangui, as one orangui commit plus a mechanical substitution per host.

- [ ] Owner approves or strikes each `rename`, `drop` and `move` below.
- [ ] Every host on orangui in production (see [`host_migration.md`](host_migration.md)).
- [ ] One orangui commit renames the classes in the sheets and the components, and says so in the message since class names are public.
- [ ] Per host: the substitution, a re-pin, the screenshot diff.

## Counts

132 classes: 126 defined in the four sheets (88 in `components.css`, 24 in `app.css`, 9 in `site.css`, 5 in `toast.css`) and 6 that a component or script emits for the hosts to style. Verdicts: 117 keep, 12 rename, 1 drop, 2 move. 17 of the 126 defined classes are host hooks, defined in a sheet and emitted by no component; none is dead, every one is emitted by at least one host.

The counts come from the sheets and the hosts' `orangui` worktrees on 2026-10-12. "Host uses" is css lines / Rust string literals per host, with `zwipe-components` counted apart from zwiper and zite since both inline it.

## The convention

- A class is named after the component it belongs to or the role it serves, in kebab-case.
- Each component family shares one prefix, the component's name: `panel-`, `nav-`, `nav-dropdown-`, `banner-`, `stat-`, `sheet-`, `hint-`, `toast-`, `diagram-`, `heat-`, `gallery-`.
- The family root is the bare name when nothing collides (`chip`, `btn`, `stat`, `diagram`, `banner`) and the component's full name otherwise (`panel-card`, `bottom-sheet`, `stats-strip`).
- Modifiers follow the prefix: `btn-xs`, `btn-danger`, `btn-action`, never `util-btn`.
- State classes that mean the same thing everywhere stay bare: `active`, `selected`, `disabled`, `show`, `snap`, `expanded`, `centered`.

## Blast radius

The 13 renames and drops touch 26 host stylesheet lines (zwiper 12, zite 6, portfolio 3, cairn 5) and 53 Rust lines (zwiper 32, cairn 21). Nothing in `zwipe-components` moves. The three moves are rule relocations, not renames: the host already emits the class, the rule changes sheets.

Every rename is prefix-safe, so a substitution of the whole token in each host's stylesheet and sources is enough, longest name first: `profile-row` also carries cairn's own `profile-row-value` along as `settings-row-value`, and `util-btn` carries `util-btn-danger`, which is the intended result. No proposed name exists in any host sheet today.

`ButtonVariant::Util` becomes `ButtonVariant::Action` and its class `btn-action`, the button named after the bar it sits in, and the hosts' Rust changes with it (zwiper 1 line, cairn 1 line).

## Rollout

1. After the last host switches production to orangui, one orangui commit renames the classes in `assets/*.css`, the components and `lib.rs`'s docs, with the old and new names listed in the message.
2. Each host: the substitution over its stylesheet and sources, `cargo update -p orangui`, build and lint.
3. The gate is the screenshot diff used for the migration: nothing visible moves. A host whose diff is not clean stays on the previous pin until it is.
4. The two moved rules (`arch-carousel`, `arch-slide`) land in the host sheets in the same host commit, before the orangui rules are deleted in a later orangui commit.

## What was judged

- `util-bar`, `util-btn`, `util-btn-danger`: "util" names nothing. The bar is the ActionBar; the button is a `btn` modifier.
- `btn-xs`: keep, a `btn` modifier in the family's form.
- `page-header-corner`, `page-header-support`: keep. The corner button is also BottomSheet's close button, but a page-header rule styles it, and `support` is the documented hook for the host's own left-corner button.
- `stat-chip`: misnamed, it is a chip rather than a StatsStrip figure, but zwiper has `stat-chip-bad`, `stat-chip-format`, `stat-chip-zone` and four more on top of it, 54 uses across zwiper, zite and cairn. Not worth a prefix.
- `tag-row`, `stats-source`, `hero-figures`: StatsStrip emits `div.hero-figures > section.stats-strip` and `div.tag-row.stats-source`, and styles none of them. `hero-figures` is zite's page section inside the kit; `tag-row` is a zite utility class. `stats-source` stays.
- `scroll-fade-x`, `scroll-fade-y`, `reveal-pending`, `reveal-in`: keep, named after the script and the role, the same on every host.
- `diagram-*`, `heat-*`: keep, consistent families. The diagram node and arrow rules sit under the heat grid banner in `components.css`, which is a comment fix, not a rename.
- `theme-switcher`: the component is ThemePicker and the sheet's own comment calls it the picker.
- `mode-toggle`, `mode-icon`, `mode-label`: keep, a three-class family for the dark-mode button.
- `nav-dropdown-*`: keep, consistent.
- `panel-card`: the suffix does no work, but zite has its own `.panel` rule, so `panel` is taken and the 15 host lines would buy nothing.
- `modal-*`: BottomSheet's four children carry a family name no component has. `sheet-` is the prefix.
- `profile-row`, `profile-row-label`: settings rows named after zwiper's profile screen; cairn uses them on its config screen.
- `announcement-banner`: the Banner root under zite's old name, with every child already `banner-*`. No host styles it, so the rename is free.
- `message-*`: not in the kit. `.message-empty` and `.message-error` are host classes in zwiper's and the portfolio's own sheets.
- `arch-carousel`, `arch-slide`: zwipe-only rules living in the kit's sheets (the site sheet's comment names ZwipeArchitecture). They move to the hosts that emit them. `stepper-btn` stays: a stepper is generic even though only zwiper uses it today.
- `hint-key`: keep, but its rule sits under the StatsStrip banner; it belongs with the hints.

## The table

Grouped by sheet and section as the sheets are written. Classes with verdict keep and no host use are listed in one line under their section.

#### components.css: Chip (orangui::Chip)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `chip` | chip | zwiper 5/69, zite 11/14, zwipe-components 2/4, portfolio 7/2, cairn 1/0 | keep |  | root named after the component |
| `selected` | chip, theme_sheet | zwiper 7/47, zite 3/8, zwipe-components 0/2, portfolio 2/1 | keep |  |  |
| `disabled` | button, chip, password_input, theme_sheet | zwiper 0/1, zite 2/0 | keep |  |  |

#### components.css: Button (orangui::Button)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `btn` | button | zwiper 4/1 | keep |  |  |
| `btn-xs` | button | zwiper 4/0 | keep |  |  |
| `util-btn` | button | zwiper 3/1, cairn 0/1 | rename | `btn-action` | `btn-` is the family prefix, and the button is named after the bar it sits in; `ButtonVariant::Util` becomes `Action` |
| `btn-danger` | button | zwiper 1/0 | keep |  |  |
| `util-btn-danger` | button | zwiper 1/0 | rename | `btn-action-danger` | follows `btn-action`, matches `btn-danger` |

#### components.css: ActionBar (orangui::ActionBar)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `util-bar` | action_bar | zwiper 1/1, cairn 3/0 | rename | `action-bar` | the component is ActionBar; "util" names nothing |

#### components.css: NavBar (orangui::NavBar)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `nav-wrapper` | nav_bar | zite 1/1, portfolio 1/0 | keep |  |  |
| `nav-panel` | nav_bar, nav_dropdown | zite 1/0, portfolio 1/0 | keep |  |  |
| `nav-brand` | hosts | zite 2/1, portfolio 3/1 | keep |  | host hook: the host emits it inside NavBar |
| `nav-links` | nav_bar | zite 4/0, portfolio 5/0 | keep |  |  |
| `nav-link` | hosts | portfolio 0/1 | keep |  | host hook: opt-in pill look for button items |
| `active` | theme_picker, theme_sheet | zwiper 1/2, zite 3/2, zwipe-components 1/3, portfolio 0/2, cairn 0/6 | keep |  |  |
| `theme-switcher` | theme_picker | portfolio 1/0 | rename | `theme-picker` | the component is ThemePicker and the sheet calls it the picker |
| `mode-toggle` | theme_picker | none | keep |  | the DarkModeButton; `mode-` is its own three-class family |
| `nav-dropdown-content` | nav_dropdown | portfolio 2/0 | keep |  |  |
| `nav-dropdown-item` | nav_dropdown, theme_picker | portfolio 0/2 | keep |  |  |

Keep, no host use: `nav-panel-inner`, `nav-toggle`, `nav-toggle-bar`, `nav-toggle-open`, `nav-panel-open`, `nav-dropdown-trigger`, `nav-dropdown`, `nav-dropdown-open`, `nav-dropdown-rows`, `nav-dropdown-label`.

#### components.css: NavDropdown (orangui::NavDropdown) + ThemePicker (orangui::ThemePicker)

Keep, no host use: `mode-icon`, `mode-label`, `nav-backdrop`, `nav-dropdown-backdrop`, `theme-swatches`, `theme-dot`.

#### components.css: Banner (orangui::Banner)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `banner-stack` | banner | zite 0/1 | keep |  |  |
| `announcement-banner` | banner | none | rename | `banner` | root of the `banner-*` family; "announcement" is zite's word |
| `status-tag` | banner | zwipe-components 0/2 | keep |  | shared by Banner and Panel, so no `banner-` prefix |
| `status-done` | banner | zwipe-components 0/1 | keep |  |  |
| `status-doing` | banner | zwipe-components 0/1 | keep |  |  |

Keep, no host use: `banner-header`, `banner-category`, `banner-text`, `banner-dismiss`, `banner-leaving`, `banner-progress`.

#### components.css: Panel (orangui::Panel)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `panel-card` | panel | zwiper 2/0, zite 7/0, portfolio 6/0 | keep |  | the root could be `panel`, but zite already has its own `.panel` rule; the suffix costs nothing |
| `panel-eyebrow-row` | panel | zite 1/0, portfolio 1/0 | keep |  |  |
| `panel-title` | panel | zwiper 1/0, zite 3/1, portfolio 1/0 | keep |  |  |
| `panel-rule` | panel | zite 0/1 | keep |  |  |
| `panel-body` | panel | zite 3/0 | keep |  |  |
| `panel-actions` | panel | zite 1/0 | keep |  |  |
| `panel-action` | panel | zite 0/14, portfolio 5/12 | keep |  |  |
| `stepper-btn` | hosts | zwiper 1/14, zwipe-components 0/2 | keep | | generic; stays in the kit though only zwiper uses it today |

Keep, no host use: `panel-eyebrow`, `panel-centered`.

#### components.css: PasswordInput (orangui::PasswordInput)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `password-input-wrapper` | password_input | zwiper 2/1 | keep |  |  |
| `input-password` | password_input | zwiper 1/2 | keep |  | modifier on `.input`, so `input-` comes first on purpose |
| `password-toggle-btn` | password_input | zwiper 2/1 | keep |  |  |

#### components.css: diagram banner

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `diagram` | hosts (documented in lib.rs) | zite 2/1, zwipe-components 0/3, portfolio 4/4 | keep |  | root named after the component |
| `diagram-scroll` | hosts (documented in lib.rs) | zwipe-components 0/1, portfolio 0/2 | keep |  |  |

#### components.css: scroll fade banner

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `scroll-fade-x` | hosts (documented in lib.rs) | zite 0/1, portfolio 0/4 | keep |  | named after the script and its role |
| `scroll-fade-y` | nav_bar | none | keep |  | named after the script and its role |

#### components.css: heat grid banner

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `chart-scroll` | hosts | portfolio 4/4, cairn 1/1 | keep |  | host hook: the wrapper around any wide svg |
| `heat-grid` | hosts | portfolio 9/1, cairn 0/1 | keep |  | host hook: the host owns the svg |
| `heat-label` | hosts | portfolio 0/3, cairn 0/2 | keep |  | host hook: the host draws the labels |
| `heat-cell` | heat_grid | portfolio 3/0, cairn 2/0 | keep |  |  |
| `heat-halo` | heat_grid | cairn 2/0 | keep |  |  |

Keep, no host use: `heat-1`, `heat-2`, `heat-3`, `heat-4`, `heat-peak`.

#### components.css: heat grid banner, the diagram node and arrow rules (misfiled under the heat grid banner)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `diagram-group` | hosts | zwipe-components 0/1, portfolio 0/1 | keep |  | host hook: the dashed frame around nodes |

Keep, no host use: `diagram-node`, `diagram-muted`, `diagram-primary`, `diagram-secondary`, `diagram-tertiary`, `diagram-success`, `diagram-title`, `diagram-sub`, `diagram-arrow`.

#### components.css: StatsStrip (orangui::StatsStrip)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `stats-strip` | stats_strip | zwiper 1/0, zite 4/0, portfolio 3/0 | keep |  |  |
| `stat` | stats_strip | zwiper 1/0, zite 1/0, portfolio 1/0 | keep |  | the StatsStrip figure; `stat-num`, `stat-label` follow |
| `stat-num` | stats_strip | zwiper 1/0, zite 1/0, portfolio 1/0 | keep |  |  |
| `stat-label` | stats_strip | zwiper 1/0, zite 1/0, portfolio 1/0 | keep |  |  |

#### components.css: StatsStrip banner, the HintKey rule (misfiled; it belongs with Hints in app.css or its own section)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `hint-key` | hint | cairn 1/0 | keep |  | sits under the StatsStrip banner; move the rule under Hints when the sheet is next touched |

#### app.css: Screen

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `screen-content` | hosts | zwiper 4/26, cairn 0/4 | keep |  | host hook: the app shell |
| `centered` | page_header, panel | zwiper 0/4, portfolio 0/1 | keep |  | state class shared by Panel and the screen |
| `flex-col` | bottom_sheet | zwiper 0/28, cairn 0/1 | keep |  |  |

#### app.css: Page header (orangui::PageHeader)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `page-header` | page_header | zwiper 0/1, zite 3/2 | keep |  |  |
| `page-header-support` | hosts (documented in page_header.rs) | zwiper 0/1, cairn 1/1 | keep |  | host hook documented in PageHeader; the host emits it |
| `page-header-corner` | bottom_sheet, page_header | cairn 1/0 | keep |  | also BottomSheet's close button; a rename gains nothing |

#### app.css: Hints (orangui::InfoButton, HintLine, HintBullets, HintKey)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `info-button` | hint | zwiper 2/1, zite 3/0, cairn 1/0 | keep |  | named after InfoButton, the hint family's opener |
| `hint-line` | hint | cairn 2/0 | keep |  |  |
| `hint-bullets` | hint | cairn 1/0 | keep |  |  |

#### app.css: Inputs and chips

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `input` | password_input | zwiper 2/45, zite 0/1, portfolio 0/23, cairn 0/2 | keep |  |  |
| `stat-chip` | hosts | zwiper 0/20, zite 3/10, cairn 1/20 | keep |  | misnamed (a chip, not a stat) but zwiper built `stat-chip-*` on it; 54 uses for a prefix |

#### app.css: Bottom sheet (orangui::BottomSheet)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `modal-backdrop` | bottom_sheet | zwiper 0/5 | rename | `sheet-backdrop` | BottomSheet child; `modal-` is a leftover family |
| `show` | bottom_sheet, gallery, toast_stack | zwiper 4/14, zite 0/9, cairn 0/2 | keep |  |  |
| `bottom-sheet` | bottom_sheet | zwiper 3/4 | keep |  | root named after the component; children take `sheet-` |
| `modal-header` | bottom_sheet | zwiper 2/2 | rename | `sheet-header` | BottomSheet child; `modal-` is a leftover family |
| `modal-title` | bottom_sheet | cairn 1/0 | rename | `sheet-title` | BottomSheet child; `modal-` is a leftover family |
| `modal-content` | bottom_sheet | zwiper 3/2, cairn 1/0 | rename | `sheet-content` | BottomSheet child; `modal-` is a leftover family |

Keep, no host use: `bottom-sheet-premount`, `snap`.

#### app.css: Settings rows and the theme sheet (orangui::ThemeSheet)

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `profile-row` | hosts | zwiper 1/11, cairn 0/10 | rename | `settings-row` | a settings row; zwiper's profile screen named it, cairn has no profile |
| `profile-row-label` | hosts | zwiper 0/10, cairn 0/10 | rename | `settings-row-label` | follows `settings-row` |
| `pref-row` | theme_sheet | zwiper 3/0 | keep |  | ThemeSheet row; "pref" is a settings word, not a host word |

Keep, no host use: `pref-row-inner`, `pref-section-label`.

#### site.css: Gallery

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `gallery-body` | gallery | zite 1/0, portfolio 1/0 | keep |  |  |
| `arch-carousel` | hosts | zwipe-components 0/1, portfolio 0/1 | move | (host sheet) | ZwipeArchitecture only; the rule moves to the hosts that draw it |
| `arch-slide` | hosts | zwipe-components 0/1, portfolio 1/1 | move | (host sheet) | ZwipeArchitecture only; the rule moves to the hosts that draw it |
| `gallery-nav` | gallery | zite 2/0, portfolio 2/0 | keep |  |  |
| `gallery-footer` | gallery | portfolio 1/0 | keep |  |  |
| `gallery-caption` | gallery | zite 1/0 | keep |  |  |
| `gallery-counter` | gallery | zite 1/0, portfolio 1/0 | keep |  |  |

Keep, no host use: `gallery-prev`, `gallery-next`.

#### toast.css: Toast stack

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `expanded` | toast_stack | zwiper 1/3, zite 2/4, zwipe-components 2/1 | keep |  |  |

Keep, no host use: `toast-container`, `animating`, `toast-tap`.

#### toast.css: Timed fade (ToastStack's `timed_fade`)

Keep, no host use: `timed-fade`.

#### No sheet: emitted by a component, styled by the hosts

| Class | Emitted by | Host uses (css/rs) | Verdict | Proposed | Why |
|---|---|---|---|---|---|
| `hero-figures` | stats_strip | zwiper 1/0, zite 5/0, portfolio 1/0 | rename | `stats-figures` | StatsStrip's wrapper; "hero" is zite's page section |
| `panel-head` | panel | none | keep |  | Panel emits it; no sheet styles it yet |
| `reveal-in` | reveal.js | zite 1/0, portfolio 9/0, cairn 2/0 | keep |  | the reveal.js contract; hosts style it |
| `reveal-pending` | reveal.js | zite 2/0, portfolio 10/0, cairn 5/0 | keep |  | the reveal.js contract; hosts style it |
| `stats-source` | stats_strip | zwiper 1/0, zite 4/0, portfolio 7/0 | keep |  | StatsStrip's source row; hosts style it |
| `tag-row` | stats_strip | zite 1/0, portfolio 1/2 | drop | (none) | zite utility class on the source row; hosts style `.stats-source` instead |
