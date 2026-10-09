# Structure

Every userscript has the same three parts, in the order they run: the generated `THEMES` block, the generated core, then the site's own code, which ends by calling `start()`.

## src/core.js, by section banner

- Theme names: the slug list, the colorblind group, and the special spellings, matching `display_theme_name` in zwipe-core.
- Stored state: the theme, the on/off switch and the launcher toggle from Tampermonkey storage, plus the launcher's spot for this hostname.
- Theme variables: `themeCss()` turns one theme into `--og-*` variables on `:root`, with zwipe's sink, overlay and shadows derived per mode. This style element is the only thing a theme change rewrites. `squircle()` and `CORE_CSS` (selection, caret, scrollbars, the wipe) live here too.
- `apply()`: swaps the theme, inside a view transition when one is wanted.
- The picker: zwipe's ThemeSheet in a shadow root, with the keyboard map. `draggable()` serves both the launcher and the sheet's title bar.
- The recolor: `recolor()` sends one color literal to a theme expression by its role (background, text, border, shadow), its hue, and its gray level. `darkGray()` reads a gray written for a dark page. `scan()` walks every readable stylesheet, and a `MutationObserver` catches inline styles and SVG color attributes. `recolorPage(options)` sets what it touches and returns the on/off switch.
- `start()`: installs the styles, applies the theme, hooks the storage listeners, mounts the picker and the menu.

## The site scripts

`orangui-fluent.user.js` maps every Fluent v9 token (and Outlook's v8 palette) to a theme variable, then adds zwipe's shapes on `fui-*` classes: dialogs, cards, text buttons, inputs, chat bubbles, avatars, the spinner. It runs the recolor for saturated colors only, which catches Teams' purple wherever it's a literal.

`orangui-halo.user.js` is mostly CSS on Halo's stable classes (nav, menu, widgets, modals, buttons, tabs, list headers, avatars) and runs the recolor on everything: grays, fonts and radii included. `darkPage: '.theme-dark'` tells it when inline styles were written for Halo's dark theme.

The picker only mounts in the top frame. Styles apply in every frame, and storage listeners keep frames and tabs on the same theme.
