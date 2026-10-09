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

**2026-10-09: Halo joins.** `orangui-halo.user.js` themes the Halo agent app with the same picker and stored theme as Teams and Outlook. Halo has no color variables, so the shared core gained a recolor engine that rewrites color literals in place; Teams uses it too, for the purple Fluent's tokens didn't reach. The picker walks with the arrow keys and has one dark/light button.

Both scripts have been checked in headless Chrome: Teams against a mock Fluent page, Halo against a saved ticket page from the owner's instance (Halo's dark theme). Neither has run on the live sites yet, and that's next.

See [`progress/todo.md`](progress/todo.md) for the ordered list.
