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
