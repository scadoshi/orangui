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

`src/fluent.js`, copied into the Teams and Outlook scripts, maps every Fluent v9 token (and Outlook's v8 palette) to a theme variable and adds zwipe's shapes on `fui-*` classes: dialogs, cards, text buttons, inputs, avatars, the spinner. It exports `FLUENT_CSS`.

`orangui-teams.user.js` adds Teams' own tokens (`colorTeamsBrand1*`, `colorBrandFlair*`, Copilot's glow, the bubble variables), the outlined chat bubbles, and zwipe's grid on `[data-tid="message-pane-list-viewport"]`. `orangui-outlook.user.js` is the Fluent look alone. Both run the recolor for saturated colors only, which catches purple wherever it's a literal or a rule Teams swaps in at runtime. Teams' biggest stylesheet comes from its CDN and can't be read, so the purple left in it (Copilot glows, brand gradients) stays.

`orangui-halo.user.js` is mostly CSS on Halo's stable classes (nav, menu, widgets, modals, buttons, tabs, list headers, avatars) and runs the recolor on everything: grays, fonts and radii included. `darkPage: '.theme-dark'` tells it when inline styles were written for Halo's dark theme. Notes and emails render in a same-origin `iframe.halo-html-renderer` the script isn't loaded into, so it writes the theme and the font into each one from the parent, again on every theme change through `onTheme`.

The picker only mounts in the top frame. Styles apply in every frame, and storage listeners keep frames and tabs on the same theme.
