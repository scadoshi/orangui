# CLAUDE.md

Guidance for Claude Code and any other AI assistant working in this repository.

## Project Overview

orangui is a set of Tampermonkey userscripts that restyle the owner's work web apps to look like zwipe: JetBrains Mono, zwipe's radii, borders and shadows, and zwipe's 31 themes with a live theme picker.

- One userscript per app: `orangui-teams.user.js`, `orangui-outlook.user.js`, `orangui-halo.user.js`. Teams and Outlook share the Fluent UI token map in `src/fluent.js`.
- No build step for installing. Each `*.user.js` is self-contained and installs straight from its raw GitHub URL; Tampermonkey updates from `@updateURL`.
- Generated blocks in every script are filled by `scripts/sync.mjs`: `THEMES` from zwipe's `zwipe-components/assets/themes.css`, and each `// <name>` block from `src/<name>.js`. Edit the files in `src/`, never the copies.

## Layout

```
src/core.js               # every script: theme variables, the picker, the recolor engine, the menu, start()
src/fluent.js             # Teams and Outlook: Fluent token map, zwipe shapes on fui-* classes
orangui-teams.user.js     # Teams' own tokens, chat bubbles, the message-list grid
orangui-outlook.user.js   # Outlook's surface variables, the reading-pane grid, message cards
orangui-halo.user.js      # Halo: zwipe shapes on Halo's classes, note iframes, the recolor on everything
scripts/sync.mjs          # fills the themes and src/ blocks in every *.user.js
context/                  # this documentation
```

`architecture/structure.md` walks a userscript section by section; `architecture/decisions.md` says why.

## Rules

- Colors come from the theme. Every color a script writes is a `var(--og-*)` or a `color-mix` of them, never a literal, except the black shadows zwipe itself uses.
- zwipe is the reference. A new style copies the matching rule from `zwipe-components/assets/components.css` or `app.css` rather than inventing a look.
- Selectors target stable class names: Fluent's `fui-*`, Halo's own (`nhd-*`, `.widget`, `.ReactTable`, `.Select__*`). Never hashed atomic classes, which change with every deploy.
- The picker lives in a closed shadow root and builds its DOM with `createElement`. No `innerHTML`: Teams enforces Trusted Types.
- Anything a script stores goes through `GM_getValue`/`GM_setValue` so every site and tab shares it.
- Bump `@version` on every change that should reach installed copies; Tampermonkey only updates on a higher version. A change to `src/core.js` bumps every script, `src/fluent.js` both Fluent ones.

## Common Commands

```bash
node scripts/sync.mjs                 # zwipe's palettes and src/ into every script (zwipe beside this repo)
for f in *.user.js; do node --check "$f"; done
```

Changes ship straight to `main`, look tweaks included: no holding them back for a review round. The scripts update from raw `main`, so that's where the owner tries them.

A saved page ("Web Page, Complete") with its scripts stripped and the userscript plus `GM_*` stubs added renders in headless Chrome for Halo and Teams. Outlook's layout is built at runtime and doesn't survive the save, so Outlook changes get checked live.

## Commit Guidelines

See `development/commit_guidelines.md`. The short version: one-line messages, no emojis, never any AI-agent signature or Co-Authored-By trailer.

## Comments

A comment says what the code does, in the present tense, and only when the code does not say it itself. The full rules are in `development/comment_guidelines.md`.

## Context Directory

```
context/
├── README.md         start-here index and current focus
├── CLAUDE.md         this file
├── architecture/     structure.md (layout), decisions.md (why)
├── development/      commit_guidelines, comment_guidelines
└── progress/         todo.md (next), backlog.md (someday), changelog.md
```
