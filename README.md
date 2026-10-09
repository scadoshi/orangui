# orangui

Tampermonkey scripts that make my work browser look like [zwipe](https://github.com/scadoshi/zwipe). Same JetBrains Mono, same rounded outlined panels, same 31 themes in light and dark, and the same theme picker with the wipe across the screen when you change themes.

Two scripts so far, sharing one theme between them:

- [orangui-fluent.user.js](https://raw.githubusercontent.com/scadoshi/orangui/main/orangui-fluent.user.js) for Teams and Outlook on the web. Both run on Microsoft's Fluent UI, which reads its colors from a few hundred CSS variables, so the script points every one at the current theme. Chat bubbles get zwipe's outline and people's pictures are squircles.
- [orangui-halo.user.js](https://raw.githubusercontent.com/scadoshi/orangui/main/orangui-halo.user.js) for Halo (`*.haloservicedesk.com`). Halo has no color variables, so this one reads every color in Halo's stylesheets and swaps it for the nearest theme color as the page loads. It works from Halo's light or dark theme.

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

That pulls zwipe's `themes.css` (it expects zwipe checked out next to this repo, or takes the path as an argument) and copies `src/core.js` into every `*.user.js`.

## A note on how it's built

This is for me. It's based on zwipe's look and its theme system, and most of it is AI-generated, more than I'd accept in anything other people depend on. If it breaks after a Teams or Halo update, that's the deal. Rules and layout are in `context/`, starting at `context/README.md`.
