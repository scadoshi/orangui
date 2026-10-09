# Changelog

## 2026-10-09: outlook 0.4.1

Outlook's message list sits on zwipe's grid, under and around the rows.

## 2026-10-09: teams 0.3.3

The chat list gets two levels. The open chat is primary: filled with the primary accent, outlined, with a bar at its left edge. Unread chats not yet opened are secondary: outlined in the secondary accent with the name in it, no fill.

## 2026-10-09: teams 0.3.2

Unread chats in the Teams list are outlined and tinted in the accent, with the chat's name in the accent too.

## 2026-10-09: teams 0.3.1, outlook 0.4.0, halo 0.2.1

The picker is bigger: about 60rem wide, the themes in a grid of larger cards, with a bigger title, toggle, swatches and buttons. In Outlook, the open email sits on zwipe's grid as an outlined card, and Outlook's surface variables follow the theme.

## 2026-10-09: teams 0.3.0, outlook 0.3.0, halo 0.2.0

The combined Teams + Outlook script splits into `orangui-teams.user.js` and `orangui-outlook.user.js`, sharing `src/fluent.js`. Teams' message list sits on zwipe's grid, and the purple that still leaked is gone where it can be reached: Teams' own tokens are mapped, and the recolor now catches token rules Fluent swaps in after load. In Halo, ticket notes and emails take the mono font and theme colors.

## 2026-10-09: fluent 0.2.0, halo 0.1.0

Halo gets its own script, `orangui-halo.user.js`, for `*.haloservicedesk.com`. The picker and theme code move to `src/core.js`, which `scripts/sync.mjs` copies into every script, and the core gains a recolor engine that rewrites color literals in stylesheets, inline styles and SVG color attributes, reading Halo's dark-theme rules as dark.

In Teams, chat bubbles get zwipe's outline (yours tinted with the accent), people's pictures are squircles, and Teams' purple goes to the theme accent. The picker walks with ↑ ↓, flips dark and light with ← → or one ☾/☀ button, saves on Enter, and drops its hint line, keeping only a warning when Dark Reader is on.

## fluent 0.1.0 (2026-10-09)

First version. `orangui-fluent.user.js` themes Teams and Outlook from zwipe's 31 palettes, light and dark, with zwipe's radii, outlines, shadows and sunken grid background. The theme picker opens on Alt+Shift+T or from a draggable ◐ launcher, previews each pick with zwipe's wipe, and keeps the choice in Tampermonkey storage shared by both apps.
