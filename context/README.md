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

## 2026-10-21: Flexoki replaces Visual Studio

`visual-studio` is gone from `ALLOWED_THEMES` and `flexoki` takes its slot, so the set stays at 50. Visual Studio's colors could not be traced to a published file, and its palette mostly repeated VS Code's. Flexoki comes from kepano/flexoki: its CSS palette and mapping table for the base colors and status colors, and its VS Code port for syntax. Synthwave '84's palette now comes from robb0wen's original theme rather than Omarchy's terminal port, and Ristretto's from monokai-pro.nvim. A host that stores a theme slug moves `visual-studio` to `vscode`, the closest match.

## 2026-10-21: the prerendered splash

On the sites the splash's mark waited on the app: the overlay painted from the prerendered HTML, but the mark only mounted once the wasm (about 1.1 MB compressed for zite) had downloaded and started, so a slow load showed an empty field first. `Splash { prerendered: true }` puts the mark in the HTML, and `SPLASH_JS` runs the whole opening from first paint on the timings the overlay carries as data attributes, driving it through `splash-running`, `splash-fading` and `splash-over` on the root. The app only waits for the fade, opens the gate and unmounts the overlay, so the opening takes its 1.2 s however long the app takes. The apps keep the Rust timeline. The portfolio and zite also draw their backdrop still, so the field rains in once, inside the splash.

## 2026-10-20: the launch guard, the two-wave decode and the splash exit

`BOOT_HEAD`, `BOOT_BG` and `BootReveal` (200fd24) keep an app's WebView dark until every stylesheet has loaded, which ended cairn's flash of unstyled text before the splash; both apps passed on a phone in release builds. `Decode` builds its art left to right in two waves (0de3e1d): a fast wave of noise draws the shape out of blank and the decode wave follows it into the glyphs. Under a `Splash` the mark then leaves the same way in reverse before the fade (ede1e2d), which stretched the splash to 1.8 s. `Splash` takes a `pace` (06e3b75), a percent of that timeline with the mark's passes included; the portfolio and zite run it at 67, about 1.2 s, and the apps keep the default. Trials of falling letters and hex rain for the noise were tried on the phone and dropped. Every host re-pinned each step: portfolio, cairn and zwipe.

## 2026-10-13: every host is on orangui

The portfolio (3cdb06f), cairn (650e008) and zwipe (791f3920) merged their `orangui` branches and take their UI from this crate; the re-pin scripts run `cargo update -p orangui`. zwipe's Magic crate, `zwipe-components` in the entries below, is now `components`. [`plans/host_migration.md`](plans/host_migration.md) has the log.

## 2026-10-12: the class rename pass shipped

The names [`plans/class_names.md`](plans/class_names.md) approved are in the sheets and the components: `action-bar`, `btn-action` with `ButtonVariant::Action`, `sheet-*` for BottomSheet's children, `settings-row`, `banner`, `theme-picker`, `stats-figures`; `tag-row` is dropped and the two Zwipe architecture rules left `site.css`. Each host runs the substitution from the plan's table before it re-pins. Later the same day the two architecture rules came back to `site.css` as `diagram-carousel` and `diagram-slide`, since a diagram carousel in the gallery frame is generic and only the name was Zwipe's. [`plans/consolidation.md`](plans/consolidation.md) audited what the hosts still duplicate, of the kit or of each other; the owner approved the top ten, the kit side is done (tinted tags, `AlertDialog`, the entrance and reveal rules, `ChartTip`, the sink and grid variables, the settings list, the site footer and the nav CTAs), and its "Host pass" section lists each host's deletions and renames.

## 2026-10-12: the crate exists

orangui is the generic part of zwipe's `zwipe-components`, lifted as is: 26 modules, the 50 themes, the component sheet with its Magic sections cut, the app and site sheets, the three scripts and the two stylesheet gates. The theme model (`ThemeConfig`, the theme list, the display names) moved in from `zwipe-core` so nothing here depends on zwipe. Component names, class names and the storage key a host passes are unchanged, so for a host the switch is a dependency line and an import path.

[`plans/class_names.md`](plans/class_names.md) audits every class the kit defines, with a verdict per name, for a rename pass after the hosts switch.

No host depends on it yet. The old `orangui` repository, the Tampermonkey scripts that give Teams and Outlook zwipe's look, is now [`tamper-orangui`](https://github.com/scadoshi/tamper-orangui).

**Next:** each host on a branch, then the production switch one host at a time, at a device. [`plans/host_migration.md`](plans/host_migration.md) has the order and the pieces each host keeps.
