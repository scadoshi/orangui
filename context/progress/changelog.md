# Changelog

## 2026-10-09: fluent 0.2.0, halo 0.1.0

Halo gets its own script, `orangui-halo.user.js`, for `*.haloservicedesk.com`. The picker and theme code move to `src/core.js`, which `scripts/sync.mjs` copies into every script, and the core gains a recolor engine that rewrites color literals in stylesheets, inline styles and SVG color attributes, reading Halo's dark-theme rules as dark.

In Teams, chat bubbles get zwipe's outline (yours tinted with the accent), people's pictures are squircles, and Teams' purple goes to the theme accent. The picker walks with ↑ ↓, flips dark and light with ← → or one ☾/☀ button, saves on Enter, and drops its hint line, keeping only a warning when Dark Reader is on.

## fluent 0.1.0 (2026-10-09)

First version. `orangui-fluent.user.js` themes Teams and Outlook from zwipe's 31 palettes, light and dark, with zwipe's radii, outlines, shadows and sunken grid background. The theme picker opens on Alt+Shift+T or from a draggable ◐ launcher, previews each pick with zwipe's wipe, and keeps the choice in Tampermonkey storage shared by both apps.
