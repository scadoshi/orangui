# Structure

A userscript reads top to bottom in the order it runs. `orangui-fluent.user.js`, by section banner:

- `<themes>`: the generated `THEMES` map, one line per theme and mode, colors in `KEYS` order. `scripts/sync-themes.mjs` owns it.
- Theme names: the slug list, the colorblind group, and the special spellings, matching `display_theme_name` in zwipe-core.
- Stored state: the theme, the on/off switch and the launcher toggle from Tampermonkey storage, plus the launcher's spot for this hostname.
- Theme variables: `themeCss()` turns one theme into `--og-*` variables on `:root`, with zwipe's sink, overlay and shadows derived per mode. This style element is the only thing a theme change rewrites.
- Fluent v9 tokens: the `fluent` object maps every token to a theme variable. `PALETTE` and `STATUS` fill Fluent's named colors (Red, Berry, Teal, and the rest) from the nearest theme color.
- Fluent v8: the older palette Outlook still uses in places.
- zwipe's shapes: `BASE`, the static stylesheet. Tokens first, then component rules copied from zwipe (panel cards, util buttons, inputs, eyebrows), the ASCII spinner, and the theme wipe keyframes.
- `apply()`: swaps the theme, inside a view transition when one is wanted.
- The picker: zwipe's ThemeSheet in a shadow root. `draggable()` serves both the launcher and the sheet header.
- Tampermonkey menu.

The picker only mounts in the top frame. Styles apply in every frame, and storage listeners keep frames and tabs on the same theme.
