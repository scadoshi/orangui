# orangui

Tampermonkey scripts that make my work browser look like [zwipe](https://github.com/scadoshi/zwipe). Same JetBrains Mono, same rounded outlined panels, same 31 themes in light and dark, and the same theme picker with the wipe across the screen when you change themes.

Right now that means Teams and Outlook on the web. Both run on Microsoft's Fluent UI, which reads its colors from a few hundred CSS variables, so one script covers both by pointing every one of those variables at the current zwipe theme.

## Install

1. Install [Tampermonkey](https://www.tampermonkey.net/).
2. Open [orangui-fluent.user.js](https://raw.githubusercontent.com/scadoshi/orangui/main/orangui-fluent.user.js) and Tampermonkey offers to install it. It checks this repo for updates on its own.
3. Install JetBrains Mono on the machine (`brew install --cask font-jetbrains-mono`). The script asks for it by name and falls back to the system monospace.

## The theme picker

Alt+Shift+T opens it from anywhere in Teams or Outlook. There's also a small ◐ in the bottom-right corner that stays faded until you hover it. Drag it wherever it's least in the way and it remembers the spot per site, or hide it from the Tampermonkey menu and stick with the shortcut. The picker itself drags by its header.

Clicking a theme previews it. Save keeps it, Back or Esc puts the old one back. The pick is stored in Tampermonkey, so Teams and Outlook stay on the same theme, and other open tabs follow along.

The Tampermonkey menu also has a switch to turn the whole thing off when a screen share needs stock Teams.

## Themes

The palettes are zwipe's, copied out of `zwipe-components/assets/themes.css` by a script so they never drift:

```
node scripts/sync-themes.mjs
```

It expects zwipe checked out next to this repo, or takes the path to `themes.css` as an argument.

## A note on how it's built

This is for me. It's based on zwipe's look and its theme system, and most of it is AI-generated, more than I'd accept in anything other people depend on. If it breaks after a Teams update, that's the deal. Rules and layout are in `context/`, starting at `context/README.md`.
