# Context: Start Here

Orientation for AI assistants and returning contributors. This `context/` tree is the project's living documentation; each subdirectory owns one concern. The outline mirrors zwipe's and cairn's so navigation carries over between repos.

Read [`CLAUDE.md`](CLAUDE.md) first for the rules.

## Directory map

| Directory | What's in it |
|-----------|--------------|
| [`architecture/`](architecture/) | `structure.md`: the tree, the three kits, what is deliberately not here, the dependencies |
| [`development/`](development/) | How to write code here: commit and comment guidelines, `styling.md` (the cascade, the color contract, the gates), the Dioxus 0.7 cheatsheet |
| [`plans/`](plans/) | `host_migration.md`: each host's move onto the crate, with a running log |
| [`progress/`](progress/) | `todo.md`: what is next, in order |

## Where things stand

The running log, newest first. Update this when something ships.

## 2026-10-13: every host is on orangui

The portfolio (3cdb06f), cairn (650e008) and zwipe (791f3920) merged their `orangui` branches and take their UI from this crate; the re-pin scripts run `cargo update -p orangui`. zwipe's Magic crate, `zwipe-components` in the entries below, is now `components`. [`plans/host_migration.md`](plans/host_migration.md) has the log.

## 2026-10-12: the class rename pass shipped

The names [`plans/class_names.md`](plans/class_names.md) approved are in the sheets and the components: `action-bar`, `btn-action` with `ButtonVariant::Action`, `sheet-*` for BottomSheet's children, `settings-row`, `banner`, `theme-picker`, `stats-figures`; `tag-row` is dropped and the two Zwipe architecture rules left `site.css`. Each host runs the substitution from the plan's table before it re-pins. Later the same day the two architecture rules came back to `site.css` as `diagram-carousel` and `diagram-slide`, since a diagram carousel in the gallery frame is generic and only the name was Zwipe's. [`plans/consolidation.md`](plans/consolidation.md) audited what the hosts still duplicate, of the kit or of each other; the owner approved the top ten, the kit side is done (tinted tags, `AlertDialog`, the entrance and reveal rules, `ChartTip`, the sink and grid variables, the settings list, the site footer and the nav CTAs), and its "Host pass" section lists each host's deletions and renames.

## 2026-10-12: the crate exists

orangui is the generic part of zwipe's `zwipe-components`, lifted as is: 26 modules, the 39 themes, the component sheet with its Magic sections cut, the app and site sheets, the three scripts and the two stylesheet gates. The theme model (`ThemeConfig`, the theme list, the display names) moved in from `zwipe-core` so nothing here depends on zwipe. Component names, class names and the storage key a host passes are unchanged, so for a host the switch is a dependency line and an import path.

[`plans/class_names.md`](plans/class_names.md) audits every class the kit defines, with a verdict per name, for a rename pass after the hosts switch.

No host depends on it yet. The old `orangui` repository, the Tampermonkey scripts that give Teams and Outlook zwipe's look, is now [`tamper-orangui`](https://github.com/scadoshi/tamper-orangui).

**Next:** each host on a branch, then the production switch one host at a time, at a device. [`plans/host_migration.md`](plans/host_migration.md) has the order and the pieces each host keeps.
