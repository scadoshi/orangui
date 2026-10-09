# CLAUDE.md

Guidance for Claude Code and any other AI assistant working in this repository.

## Project Overview

orangui is a set of Tampermonkey userscripts that restyle the owner's work web apps to look like zwipe: JetBrains Mono, zwipe's radii, borders and shadows, and zwipe's 31 themes with a live theme picker.

- One userscript per family of sites. `orangui-fluent.user.js` covers Teams and Outlook, which both run Fluent UI.
- No build step. Each `*.user.js` is self-contained and installs straight from its raw GitHub URL; Tampermonkey updates from `@updateURL`.
- The only generated code is the `THEMES` block, written by `scripts/sync-themes.mjs` from zwipe's `zwipe-components/assets/themes.css`. Never edit it by hand.

## Layout

```
orangui-fluent.user.js    # Teams + Outlook: Fluent token overrides, zwipe shapes, theme picker
scripts/sync-themes.mjs   # copies zwipe's palettes into every *.user.js
context/                  # this documentation
```

`architecture/structure.md` walks a userscript section by section; `architecture/decisions.md` says why.

## Rules

- Colors come from the theme. Inside a userscript, every color is a `var(--og-*)` or a `color-mix` of them, never a literal, except the black shadows zwipe itself uses.
- zwipe is the reference. A new style copies the matching rule from `zwipe-components/assets/components.css` or `app.css` rather than inventing a look.
- Selectors target Fluent's stable `fui-*` class names, never the hashed atomic classes, which change with every Teams deploy.
- The picker lives in a closed shadow root and builds its DOM with `createElement`. No `innerHTML`: Teams enforces Trusted Types.
- Anything a script stores goes through `GM_getValue`/`GM_setValue` so every site and tab shares it.
- Bump `@version` on every change that should reach installed copies; Tampermonkey only updates on a higher version.

## Common Commands

```bash
node scripts/sync-themes.mjs          # pull zwipe's palettes (zwipe checked out beside this repo)
node --check orangui-fluent.user.js   # syntax check
```

To try a change without pushing, paste the file into a new Tampermonkey script and disable the installed one.

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
