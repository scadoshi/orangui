# CLAUDE.md

Guidance for Claude Code and any other AI assistant working in this repository. Everything here is enforced somewhere: by CI, by a test, or by a host that broke once.

## Project Overview

orangui is a Dioxus UI kit: themes, panels, nav, charts and the entrance effects that my apps and sites share. One crate renders and styles the pieces so the hosts never drift. The name is orangutan, a call sign of mine, plus GUI.

- One Rust crate, flat: every module is one component or one hook family, and `src/lib.rs` is the whole public surface. `architecture/structure.md` walks it.
- Hosts: [zwipe](https://github.com/scadoshi/zwipe) (the app and zwipe.net), [portfolio](https://github.com/scadoshi/portfolio) and [cairn](https://github.com/scadoshi/cairn). Each takes the crate as a git dependency and pins a commit in its lockfile. `plans/host_migration.md` is where each one stands.
- Styling ships as string constants a host inlines: `THEMES_CSS`, `COMPONENTS_CSS`, then `APP_CSS` or `SITE_CSS`. The host's own sheet may land before or after them, and the kit works either way. `development/styling.md` has the cascade and the color contract.

## Purity Rules

The crate knows no host. These are non-negotiable:

- **No platform feature.** `dioxus` with its defaults only. A host enables web, mobile, desktop or fullstack and cargo unifies. Platform-bound code takes `#[cfg(target_arch = "wasm32")]` with a branch for everywhere else, as `theme_store` and the entrance timer do.
- **One native exception.** The OS back gesture (`os_back.rs`) branches on `#[cfg(target_os = "ios")]` and `#[cfg(target_os = "android")]`, does nothing elsewhere, and takes its native crates (`objc2`, `jni`, `ndk-context`) under the matching target tables in `Cargo.toml`. That is the only target-OS code allowed, and it still pulls no Dioxus platform feature: what only a host has, such as the WKWebView from `dioxus/mobile` or the patched Android activity, comes in as an argument.
- **No host knowledge.** Nothing in here names a product, a domain model, a route or a URL. A component that needs to know about the host takes it as a prop or a callback. If a piece only makes sense for one host, it belongs in that host, built from the pieces here.
- **No color literals in component rules.** A rule names a theme variable. Status colors mean an outcome; decoration takes a `--palette-N` slot. The two stylesheet gates in `tests/` hold this.
- **Defaults are soft; structure is not; load order is not relied on.** A value a host tunes goes in a `:where(...)` rule, so a host rule of class specificity or higher wins whether the host's sheet loads before or after the kit's; display, position and the keyframes stay in the class rule. A host's universal or element reset that touches a soft property goes in `@layer reset { ... }`. `tests/soft_defaults.rs` holds the kit side, and `development/styling.md` has the rule.
- **Class names are public.** A host styles against them, so renaming one is a breaking change and gets called out in the commit.
- **The theme list lives here** (`src/theme.rs`) beside the palettes it names. A host that stores a user's theme elsewhere keeps its own list and checks it against this one in a test.

## Common Commands

```bash
cargo test --all-features                                          # unit tests plus the stylesheet gates
cargo +nightly fmt                                                 # rustfmt.toml uses an unstable option
cargo clippy --all-targets --all-features -- -D warnings           # what CI runs
cargo clippy --target wasm32-unknown-unknown --all-features -- -D warnings   # and for the browser
```

## Linting

The 22 clippy lints zwipe uses, in `Cargo.toml` `[lints.clippy]`, with the panic family (`unwrap`, `expect`, `panic`, indexing, slicing) warned outside tests. `clippy.toml` sets the thresholds and allows the panic family inside tests. CI treats warnings as errors, on the native target and on wasm.

## Commit Guidelines

See `development/commit_guidelines.md`. The short version: one-line messages, no emojis, never any AI-agent signature or Co-Authored-By trailer. Never push without being asked.

## Comments

A comment says what the code does, in the present tense, and only when the code does not say it itself. No history, no "without this", no restating the name. The full rules with examples are in `development/comment_guidelines.md`.

## Markdown Conventions

- **Do not hard-wrap prose.** One paragraph is one line; one bullet is one line. Let the editor or viewer soft-wrap it.
- Code fences, tables and indented blocks keep their own formatting.
- No em dashes. American English.

## Context Directory

```
context/
├── README.md               # start-here orientation and where the migration stands
├── CLAUDE.md               # this file
├── architecture/           # structure.md: the tree, the kits, what is deliberately not here
├── development/            # commit and comment guidelines, styling (the cascade and color contract), the Dioxus 0.7 cheatsheet
├── plans/                  # host_migration.md: each host's move onto the crate
└── progress/               # todo.md: what is next, in order
```
