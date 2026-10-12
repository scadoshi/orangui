# Host migration

**Status: IN PROGRESS.** This block is the running log; update it as each item lands.

- [x] 2026-10-12 The crate exists: the generic parts of zwipe's `zwipe-components` lifted as is, with the theme model moved in beside the palettes, the Magic sections cut from the stylesheet, and CI green on the first push. No host depends on it yet.
- [x] 2026-10-12 zwipe branch `orangui` (head d9f10b05, pushed, not merged): `zwipe-components` depends on orangui, keeps the Magic pieces, and re-exports the rest, so zwiper and zite kept their import paths. Its stylesheet is now `assets/magic.css`, exposed as `MAGIC_CSS`, which both hosts inline right after `COMPONENTS_CSS`. The hosts hold orangui's `ThemeConfig`; `zwipe_components::theme_from_preferences` converts from core's `UserPreferences`, and two tests pin core's theme list and names to orangui's. All gates green; zite before and after screenshots identical except four pixels of a banner countdown between captures. The app itself was not screenshotted.
- [x] 2026-10-12 portfolio branch `orangui` (head ccfef21, pushed, not merged): depends on orangui, the Zwipe architecture carousel is the site's own copy under `src/components/zwipe_architecture.rs`, and the explicit `getrandom` dependency went with zwipe-core. All five deploy gates and the static build green; home, the Zwipe page and a side quest pixel-identical before and after, bar a playing video.
- [x] 2026-10-12 cairn branch `orangui` (head eafd652, pushed, not merged): depends on orangui with the `toast` feature; the vendored `tao` stays pinned to zwipe. Format, clippy, 114 tests and the hook-order check green with the default desktop feature. The simulator build and screenshots are still to do at a device.
- [ ] The production switch, one host at a time, at a device: merge the branch, deploy, check live. zite and the portfolio are public pages; the app is a store cut. cairn needs its simulator build first.
- [ ] The re-pin scripts move from `cargo update -p zwipe-components` to `cargo update -p orangui`, and zynergy's `zwipe-core` re-pin stays as it is.
- [ ] crates.io: `orangui` 0.1.0. The toast stack blocks it, since `dioxus-primitives` is a git pin and crates.io refuses git dependencies even behind a feature. The split: the repo becomes a workspace with `orangui` (published) and `orangui-toast` (git only, the toast stack and its sheet), the timing constants staying in `orangui`. cairn's and zwipe's branches then take `orangui-toast` by git. Publishing needs the owner's crates.io token.

**One sentence:** every host takes its UI from orangui, and `zwipe-components` shrinks to the Magic pieces that need `zwipe-core`.

## Why the hosts switch one at a time

Each host pins a commit in its lockfile, so a half-done migration breaks nothing: a host on `zwipe-components` keeps working while another is on orangui. The class names and the component names are unchanged, so for a host the switch is the dependency line, the import path and a lockfile. The screenshots are the test: nothing visible should move.

## zwipe

`zwipe-components` keeps what needs `zwipe-core`: `CardDetails`, `CardRow`, `CardRoleChips`, `KeywordChips`, `OracleText`, `FlippableCardImage`, `Changelog`, the deck charts (`DeckCharts`, `ManaCurve`, `ManaFulfillment`, `DrawOdds`, `ChartLabel`) and `ZwipeArchitecture`. Its stylesheet keeps the sections cut from orangui's: OracleText, KeywordChips, CardRow, FlippableCardImage, Changelog, and the mana glyph rule inside a chip. Everything else is deleted and re-exported from orangui, so zwiper and zite import paths can stay `zwipe_components::` through the switch and move to `orangui::` at leisure.

The theme model stays in `zwipe-core` too, because zerver validates a saved theme against `ALLOWED_THEMES` and core cannot depend on a UI crate. `zwipe-components` converts between the two `ThemeConfig` types (the `From<&UserPreferences>` impl lands there), and a test asserts that core's `ALLOWED_THEMES` and `COLORBLIND_THEMES` equal orangui's, so the lists cannot drift.

The stylesheet gates in zwipe-components shrink to its own sheet. zwipe's build copies orangui's sheets the way it copies the components' today.

## portfolio

The portfolio shows the Zwipe architecture carousel on its Zwipe page. It is Zwipe content, so it does not belong in orangui, and after the switch the portfolio no longer depends on `zwipe-components`. The carousel is static: two slides of `DiagramNode` and `DiagramArrow` in a `GalleryFrame`. It moves into the portfolio's own source as a copy, beside the other project diagrams the portfolio already draws itself. zite keeps the `zwipe-components` copy.

## cairn

cairn uses the app kit and the toast stack. It also takes `tao` from zwipe's vendored copy, locked to the same zwipe commit as `zwipe-components`; that dependency stays on zwipe and is unrelated to the UI crate.

## Open questions

- Whether orangui should ship the theme-sync test as a helper (a function a host calls with its own list), so each host writes one line instead of its own test.
- Whether `zwipe-components` keeps its name once it is only the Magic pieces. It could, since it is still zwipe's components.
