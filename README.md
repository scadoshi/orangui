# orangui

Tampermonkey scripts that make my work browser look like [zwipe](https://github.com/scadoshi/zwipe). Same JetBrains Mono, same rounded outlined panels, same 31 themes in light and dark, and the same theme picker with the wipe across the screen when you change themes.

Three scripts, sharing one theme between them:

- [orangui-teams.user.js](https://raw.githubusercontent.com/scadoshi/orangui/main/orangui-teams.user.js) for Teams on the web. Chat bubbles get zwipe's outline, the message list sits on zwipe's grid, people's pictures are squircles, and Teams' purple goes to the theme accent.
- [orangui-outlook.user.js](https://raw.githubusercontent.com/scadoshi/orangui/main/orangui-outlook.user.js) for Outlook on the web, and the Outlook calendar Teams shows inside itself.
- [orangui-halo.user.js](https://raw.githubusercontent.com/scadoshi/orangui/main/orangui-halo.user.js) for Halo (`*.haloservicedesk.com`). Halo has no color variables, so this one reads every color in Halo's stylesheets and swaps it for the nearest theme color as the page loads, ticket notes included. It works from Halo's light or dark theme.

Teams and Outlook both run on Microsoft's Fluent UI, which reads its colors from a few hundred CSS variables, so those two share a map that points every one at the current theme.

Coming from the old combined "orangui: Teams + Outlook" script: delete it in Tampermonkey and install the Teams and Outlook ones. Its update link is gone.

## Install

1. Install [Tampermonkey](https://www.tampermonkey.net/).
2. Open either link above and Tampermonkey offers to install it. Each checks this repo for updates on its own.
3. Install JetBrains Mono on the machine (`brew install --cask font-jetbrains-mono`). The scripts ask for it by name and fall back to the system monospace.
4. Turn Dark Reader off for Teams, Outlook and Halo if it's on. It rewrites the same colors and the two fight; the picker says so when it spots it.

## The theme picker

Alt+Shift+T opens it from anywhere. There's also a small ◐ in the bottom-right corner that stays faded until you hover it. Drag it wherever it's least in the way and it remembers the spot per site, or hide it from the Tampermonkey menu and stick with the shortcut. The picker itself moves by dragging its title.

↑ and ↓ walk the themes and preview each one, ← and → (or the ☾/☀ button) flip dark and light, Enter keeps it, Esc puts the old one back. The pick is stored in Tampermonkey, so Teams, Outlook and Halo all stay on the same theme, and other open tabs follow along.

The Tampermonkey menu also has a switch to turn the whole thing off when a screen share needs stock colors.

## Working on it

The palettes are zwipe's, and the picker and color code are shared, so both get copied into each script rather than typed twice:

```
node scripts/sync.mjs
```

That pulls zwipe's `themes.css` (it expects zwipe checked out next to this repo, or takes the path as an argument) and copies the shared files in `src/` into every `*.user.js` that asks for them.

## A note on how it's built

This is for me. It's based on zwipe's look and its theme system, and most of it is AI-generated, more than I'd accept in anything other people depend on. If it breaks after a Teams or Halo update, that's the deal. Rules and layout are in `context/`, starting at `context/README.md`.
