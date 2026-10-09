# Context: Start Here

Orientation for AI assistants and returning contributors. Each subdirectory owns one concern, and the outline mirrors zwipe's and cairn's so navigation carries over between repos.

Read [`CLAUDE.md`](CLAUDE.md) first for the rules.

## Directory map

| Directory | What's in it |
|-----------|--------------|
| [`architecture/`](architecture/) | `structure.md` (what each part of a userscript does), `decisions.md` (the numbered why) |
| [`development/`](development/) | Commit and comment standards |
| [`progress/`](progress/) | `todo.md` (next), `backlog.md` (someday), `changelog.md` |

## Current focus

**2026-10-09: first version.** `orangui-fluent.user.js` themes Teams and Outlook from zwipe's palettes, with the picker on Alt+Shift+T and a draggable ◐ launcher. It grew out of a Gruvbox-only Teams + Outlook script; the token map is the same idea, pointed at theme variables instead of fixed hex.

It has been checked against a mock Fluent page in headless Chrome, not yet on live Teams or Outlook. That's the next thing: install it on the work machine and fix whatever Fluent does differently in production.

See [`progress/todo.md`](progress/todo.md) for the ordered list.
