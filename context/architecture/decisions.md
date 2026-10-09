# Decisions

Numbered so other docs can cite them.

## 1. Override Fluent's tokens, not its components

Teams and Outlook both draw everything from Fluent's CSS variables, and a custom property declared `!important` in a stylesheet beats the provider's own declaration. Overriding a few hundred tokens rethemes almost the whole app without knowing its DOM. Component rules are kept for what tokens can't express: zwipe's outlines, the eyebrow, the spinner.

## 2. Tokens point at theme variables

Every token is `var(--og-*)` or a `color-mix` of them, so a theme change rewrites one small style element of about 25 variables instead of regenerating the token sheet. It's also what makes the live preview cheap enough to run on every click.

## 3. zwipe's palettes, copied by a script

The themes are zwipe's `themes.css` read into a JS map, not hand-ported. zwipe is where palettes get tuned, and a sync script means orangui follows without anyone retyping hex. Only the 22 colors a userscript needs are copied.

## 4. Self-contained userscripts, no build

Each script installs from its raw URL and updates itself through Tampermonkey. A bundler or `@require` would make installing on a locked-down work machine harder for no gain at this size.

## 5. The picker is out of the way by default

The launcher is a 1.6rem ◐ at 30% opacity that drags anywhere and remembers its spot per site, and it can be hidden entirely. Alt+Shift+T always works. Work apps are busy enough already, and the theme gets picked once a week at most.

## 6. Shadow root for the picker

Fluent's resets and Teams' own CSS can't reach inside a closed shadow root, and the theme variables still inherit into it. The picker therefore looks the same on every site without fighting each one.

## 7. Rounded, like zwipe, not square

The script this grew from zeroed every radius and drew `[ ]` text buttons for a terminal look. orangui follows zwipe instead: 0.4rem inputs, 0.5rem buttons, 0.6rem chips, 1rem panels, and outlines that take the accent on hover. The ASCII spinner stayed because zwipe would have it.

## 8. Shared code is copied, not required

The picker, the theme code and the recolor live once in `src/core.js`, and `scripts/sync.mjs` copies them into each script between markers. Tampermonkey's `@require` would avoid the copy, but every script would then depend on a second URL being reachable from a work network, and the files would stop being readable on their own.

## 9. Recolor the literals where there are no tokens

Halo writes its colors as literals across five stylesheets and as inline styles in hundreds of places, so there is nothing to override the way Fluent's tokens are. The recolor reads each literal by its role (background, text, border), its hue and its gray level, and rewrites it to a theme variable in place. It's a cruder map than Fluent's, but it covers parts of Halo nobody has looked at yet, and a theme change still only touches the variables.

## 10. Read dark rules as dark

The gray ladder runs the other way on a dark page: `#353535` is the page in Halo's dark theme, but the same gray would be a heavy border in its light one. A rule whose selector or media query mentions dark is read as dark, and inline styles are read by whether the page currently has `.theme-dark`. Without this, every gray in Halo's dark theme lands on the same border color.
