// What the Fluent UI scripts share: every Fluent v9 token and Outlook's v8 palette pointed at a theme variable, and zwipe's shapes on Fluent's components.
// scripts/sync.mjs copies this file into each Fluent userscript between the fluent markers, after the core.

// == Fluent v9 tokens, every one pointed at a theme variable ==

const fluent = {
  // zwipe's radii: 0.4rem inputs, 0.5rem buttons, 0.6rem chips, 1rem panels. Circular stays circular.
  borderRadiusSmall: '0.25rem',
  borderRadiusMedium: '0.5rem',
  borderRadiusLarge: '0.6rem',
  borderRadiusXLarge: '1rem',
  borderRadius2XLarge: '1rem',
  borderRadius3XLarge: '1rem',
  borderRadius4XLarge: '1rem',
  borderRadius5XLarge: '1rem',
  borderRadius6XLarge: '1rem',
  fontFamilyBase: FONT,
  fontFamilyMonospace: FONT,
  fontFamilyNumeric: FONT,

  colorNeutralForeground1: v('text'),
  colorNeutralForeground1Hover: v('text'),
  colorNeutralForeground1Pressed: v('text'),
  colorNeutralForeground1Selected: v('text'),
  colorNeutralForeground2: v('text'),
  colorNeutralForeground2Hover: v('text'),
  colorNeutralForeground2Pressed: v('text'),
  colorNeutralForeground2Selected: v('text'),
  colorNeutralForeground2BrandHover: v('a1'),
  colorNeutralForeground2BrandPressed: v('a1'),
  colorNeutralForeground2BrandSelected: v('a1'),
  colorNeutralForeground3: v('subtle'),
  colorNeutralForeground3Hover: v('text'),
  colorNeutralForeground3Pressed: v('text'),
  colorNeutralForeground3Selected: v('text'),
  colorNeutralForeground3BrandHover: v('a1'),
  colorNeutralForeground3BrandPressed: v('a1'),
  colorNeutralForeground3BrandSelected: v('a1'),
  colorNeutralForeground4: v('muted'),
  colorNeutralForeground5: v('subtle'),
  colorNeutralForeground5Hover: v('text'),
  colorNeutralForeground5Pressed: v('text'),
  colorNeutralForeground5Selected: v('text'),
  colorNeutralForegroundDisabled: v('b1'),
  colorNeutralForegroundInvertedDisabled: clear(v('text'), 40),
  colorBrandForegroundLink: v('a1'),
  colorBrandForegroundLinkHover: v('a1'),
  colorBrandForegroundLinkPressed: v('a1'),
  colorBrandForegroundLinkSelected: v('a1'),
  colorNeutralForeground2Link: v('text'),
  colorNeutralForeground2LinkHover: v('text'),
  colorNeutralForeground2LinkPressed: v('text'),
  colorNeutralForeground2LinkSelected: v('text'),
  colorCompoundBrandForeground1: v('a1'),
  colorCompoundBrandForeground1Hover: v('a1'),
  colorCompoundBrandForeground1Pressed: v('a1'),
  colorBrandForeground1: v('a1'),
  colorBrandForeground2: v('a1'),
  colorBrandForeground2Hover: v('a1'),
  colorBrandForeground2Pressed: v('text'),
  colorNeutralForeground1Static: v('bg'),
  colorNeutralForegroundStaticInverted: v('text'),
  colorNeutralForegroundInverted: v('bg'),
  colorNeutralForegroundInvertedHover: v('bg'),
  colorNeutralForegroundInvertedPressed: v('bg'),
  colorNeutralForegroundInvertedSelected: v('bg'),
  colorNeutralForegroundInverted2: v('bg'),
  colorNeutralForegroundOnBrand: v('bg'),
  colorNeutralForegroundInvertedLink: v('bg'),
  colorNeutralForegroundInvertedLinkHover: v('bg'),
  colorNeutralForegroundInvertedLinkPressed: v('bg'),
  colorNeutralForegroundInvertedLinkSelected: v('bg'),
  colorBrandForegroundInverted: v('a1'),
  colorBrandForegroundInvertedHover: v('a1'),
  colorBrandForegroundInvertedPressed: v('a1'),
  colorBrandForegroundOnLight: v('a1'),
  colorBrandForegroundOnLightHover: v('a1'),
  colorBrandForegroundOnLightPressed: v('a1'),
  colorBrandForegroundOnLightSelected: v('a1'),

  colorNeutralBackground1: v('bg'),
  colorNeutralBackground1Hover: v('b2'),
  colorNeutralBackground1Pressed: v('sink'),
  colorNeutralBackground1Selected: v('b2'),
  colorNeutralBackground2: v('bg'),
  colorNeutralBackground2Hover: v('b2'),
  colorNeutralBackground2Pressed: v('sink'),
  colorNeutralBackground2Selected: v('b2'),
  colorNeutralBackground3: v('sink'),
  colorNeutralBackground3Hover: v('b2'),
  colorNeutralBackground3Pressed: v('sink'),
  colorNeutralBackground3Selected: v('bg'),
  colorNeutralBackground4: v('sink'),
  colorNeutralBackground4Hover: v('bg'),
  colorNeutralBackground4Pressed: v('sink'),
  colorNeutralBackground4Selected: v('bg'),
  colorNeutralBackground5: v('sink'),
  colorNeutralBackground5Hover: v('sink'),
  colorNeutralBackground5Pressed: v('sink'),
  colorNeutralBackground5Selected: v('sink'),
  colorNeutralBackground6: v('b2'),
  colorNeutralBackground7: 'transparent',
  colorNeutralBackground7Hover: v('sink'),
  colorNeutralBackground7Pressed: v('sink'),
  colorNeutralBackground7Selected: 'transparent',
  colorNeutralBackground8: v('bg'),
  colorNeutralBackgroundInverted: v('text'),
  colorNeutralBackgroundInvertedHover: v('text'),
  colorNeutralBackgroundInvertedPressed: v('subtle'),
  colorNeutralBackgroundInvertedSelected: v('text'),
  colorNeutralBackgroundStatic: v('b2'),
  colorNeutralBackgroundAlpha: clear(v('sink'), 50),
  colorNeutralBackgroundAlpha2: clear(v('sink'), 70),
  colorSubtleBackgroundHover: v('b2'),
  colorSubtleBackgroundPressed: v('sink'),
  colorSubtleBackgroundSelected: v('b2'),
  colorSubtleBackgroundLightAlphaHover: clear(v('bg'), 80),
  colorSubtleBackgroundLightAlphaPressed: clear(v('bg'), 50),
  colorSubtleBackgroundInvertedHover: clear(v('sink'), 10),
  colorSubtleBackgroundInvertedPressed: clear(v('sink'), 30),
  colorSubtleBackgroundInvertedSelected: clear(v('sink'), 20),
  colorNeutralBackgroundDisabled: v('sink'),
  colorNeutralBackgroundDisabled2: v('bg'),
  colorNeutralBackgroundInvertedDisabled: clear(v('text'), 10),
  colorNeutralStencil1: v('b1'),
  colorNeutralStencil2: v('b2'),
  colorNeutralStencil1Alpha: clear(v('text'), 10),
  colorNeutralStencil2Alpha: clear(v('text'), 5),
  colorBackgroundOverlay: v('overlay'),
  colorScrollbarOverlay: clear(v('text'), 60),

  // Brand fills read like zwipe's active keyword chip: the accent, with the background as its text.
  colorBrandBackground: v('a1'),
  colorBrandBackgroundHover: mix(v('a1'), 88, v('text')),
  colorBrandBackgroundPressed: mix(v('a1'), 80),
  colorBrandBackgroundSelected: v('a1'),
  colorCompoundBrandBackground: v('a1'),
  colorCompoundBrandBackgroundHover: mix(v('a1'), 88, v('text')),
  colorCompoundBrandBackgroundPressed: mix(v('a1'), 80),
  colorBrandBackgroundStatic: v('a1'),
  colorBrandBackground2: mix(v('a1'), 15),
  colorBrandBackground2Hover: mix(v('a1'), 25),
  colorBrandBackground2Pressed: mix(v('a1'), 10),
  colorBrandBackground3Static: v('a1'),
  colorBrandBackground4Static: v('a1'),
  colorBrandBackgroundInverted: v('text'),
  colorBrandBackgroundInvertedHover: v('text'),
  colorBrandBackgroundInvertedPressed: v('a1'),
  colorBrandBackgroundInvertedSelected: v('subtle'),
  colorNeutralCardBackground: v('bg'),
  colorNeutralCardBackgroundHover: v('bg'),
  colorNeutralCardBackgroundPressed: v('sink'),
  colorNeutralCardBackgroundSelected: v('b2'),
  colorNeutralCardBackgroundDisabled: v('sink'),

  colorNeutralStrokeAccessible: v('muted'),
  colorNeutralStrokeAccessibleHover: v('subtle'),
  colorNeutralStrokeAccessiblePressed: v('muted'),
  colorNeutralStrokeAccessibleSelected: v('a1'),
  colorNeutralStroke1: v('b1'),
  colorNeutralStroke1Hover: v('muted'),
  colorNeutralStroke1Pressed: v('muted'),
  colorNeutralStroke1Selected: v('a1'),
  colorNeutralStroke2: v('b3'),
  colorNeutralStroke3: v('b2'),
  colorNeutralStroke4: v('b2'),
  colorNeutralStroke4Hover: v('bg'),
  colorNeutralStroke4Pressed: v('bg'),
  colorNeutralStroke4Selected: v('b2'),
  colorNeutralStrokeSubtle: v('sink'),
  colorNeutralStrokeOnBrand: v('bg'),
  colorNeutralStrokeOnBrand2: v('text'),
  colorNeutralStrokeOnBrand2Hover: v('text'),
  colorNeutralStrokeOnBrand2Pressed: v('text'),
  colorNeutralStrokeOnBrand2Selected: v('text'),
  colorBrandStroke1: v('a1'),
  colorBrandStroke2: v('a1'),
  colorBrandStroke2Hover: v('a1'),
  colorBrandStroke2Pressed: v('b2'),
  colorBrandStroke2Contrast: v('a1'),
  colorCompoundBrandStroke: v('a1'),
  colorCompoundBrandStrokeHover: v('a1'),
  colorCompoundBrandStrokePressed: v('a1'),
  colorNeutralStrokeDisabled: v('b3'),
  colorNeutralStrokeDisabled2: v('b2'),
  colorNeutralStrokeInvertedDisabled: clear(v('text'), 40),
  colorNeutralStrokeAlpha: clear(v('text'), 10),
  colorNeutralStrokeAlpha2: clear(v('text'), 20),
  colorStrokeFocus1: v('bg'),
  colorStrokeFocus2: v('a2'),

  colorNeutralShadowAmbient: 'rgba(0,0,0,.24)',
  colorNeutralShadowKey: 'rgba(0,0,0,.28)',
  colorNeutralShadowAmbientLighter: 'rgba(0,0,0,.12)',
  colorNeutralShadowKeyLighter: 'rgba(0,0,0,.14)',
  colorNeutralShadowAmbientDarker: 'rgba(0,0,0,.4)',
  colorNeutralShadowKeyDarker: 'rgba(0,0,0,.48)',
  colorBrandShadowAmbient: 'rgba(0,0,0,.3)',
  colorBrandShadowKey: 'rgba(0,0,0,.25)',
  // Flat in place, zwipe's soft drop under anything that floats.
  shadow2: 'none',
  shadow4: 'none',
  shadow8: v('shadow-sm'),
  shadow16: v('shadow-sm'),
  shadow28: v('shadow-md'),
  shadow64: v('shadow-md'),
  shadow2Brand: 'none',
  shadow4Brand: 'none',
  shadow8Brand: v('shadow-sm'),
  shadow16Brand: v('shadow-sm'),
  shadow28Brand: v('shadow-md'),
  shadow64Brand: v('shadow-md'),
};

// Fluent's named palette colors, each sent to the theme color nearest it.
const PALETTE = {
  err: ['Red', 'DarkRed', 'Cranberry'],
  ok: ['Green', 'LightGreen', 'Forest', 'DarkGreen'],
  warn: ['Yellow', 'Marigold', 'Gold', 'Brass'],
  p2: ['DarkOrange', 'Pumpkin', 'Peach', 'Brown'],
  p4: ['Berry', 'Purple', 'Grape', 'Lilac', 'Pink', 'Magenta', 'Plum'],
  p6: ['Seafoam', 'LightTeal', 'Teal'],
  p1: ['Steel', 'Blue', 'RoyalBlue', 'Cornflower', 'Navy', 'Lavender'],
  muted: ['Beige', 'Mink', 'Platinum', 'Anchor'],
};
const STATUS = { ok: 'Success', warn: 'Warning', err: 'Danger' };

const tint = (prefix, c) => {
  fluent[`${prefix}Background1`] = mix(c, 12);
  fluent[`${prefix}Background2`] = mix(c, 25);
  fluent[`${prefix}Background3`] = c;
  fluent[`${prefix}Foreground1`] = c;
  fluent[`${prefix}Foreground2`] = c;
  fluent[`${prefix}Foreground3`] = c;
  fluent[`${prefix}ForegroundInverted`] = c;
  fluent[`${prefix}BorderActive`] = c;
  fluent[`${prefix}Border1`] = mix(c, 40);
  fluent[`${prefix}Border2`] = c;
};
for (const [key, names] of Object.entries(PALETTE)) for (const n of names) tint(`colorPalette${n}`, v(key));
for (const [key, n] of Object.entries(STATUS)) {
  tint(`colorStatus${n}`, v(key));
  fluent[`colorStatus${n}Background3Hover`] = mix(v(key), 88, v('text'));
  fluent[`colorStatus${n}Background3Pressed`] = mix(v(key), 80);
}

// == Fluent v8, which Outlook still runs beside v9 ==

const fluent8 = {
  themeDarker: v('a1'),
  themeDark: v('a1'),
  themeDarkAlt: v('a1'),
  themePrimary: v('a1'),
  themeSecondary: mix(v('a1'), 70),
  themeTertiary: mix(v('a1'), 50),
  // The selected-mail tint.
  themeLight: mix(v('a1'), 22),
  themeLighter: mix(v('a1'), 16),
  themeLighterAlt: mix(v('a1'), 10),
  black: v('text'),
  neutralDark: v('text'),
  neutralPrimary: v('text'),
  neutralPrimaryAlt: v('subtle'),
  neutralSecondary: v('muted'),
  neutralSecondaryAlt: v('muted'),
  neutralTertiary: v('b1'),
  neutralTertiaryAlt: v('b3'),
  neutralQuaternary: v('b2'),
  neutralQuaternaryAlt: v('b2'),
  neutralLight: v('b2'),
  neutralLighter: v('bg'),
  neutralLighterAlt: v('sink'),
  white: v('bg'),
  redDark: v('err'),
};

const decls = (o) => Object.entries(o).map(([k, val]) => `--${k}: ${val} !important;`).join('\n');

// == zwipe's shapes on Fluent's components ==

const FLUENT_CSS = `
  :root, .fui-FluentProvider { ${decls(fluent)} }
  :root, body, [style*="--neutralPrimary"], [style*="--themePrimary"] { ${decls(fluent8)} }

  html, body { min-height: 100%; background-attachment: fixed; }
  ${grid('html, body')}
  body, input, textarea, button, select, [contenteditable] { font-family: ${FONT}; }

  /* panels: the dialog is zwipe's panel card, menus and popovers its banners */
  .fui-DialogSurface {
    border: 1px solid var(--og-b2) !important; border-radius: 1rem !important; box-shadow: var(--og-shadow-md) !important;
  }
  :is(.fui-PopoverSurface, .fui-MenuPopover, .fui-Listbox, .fui-Tooltip) {
    border: 1px solid var(--og-b2) !important; box-shadow: var(--og-shadow-sm) !important;
  }
  .fui-Card { border: 1px solid var(--og-b2) !important; transition: border-color .2s; }
  .fui-Card:hover { border-color: var(--og-a1) !important; }

  /* people as squircles; the presence dot stays round */
  ${squircle('.fui-Avatar, .fui-Avatar__image, .fui-Avatar__initials, .fui-Avatar__icon, .fui-Avatar::before, .fui-Avatar::after')}

  /* text buttons are zwipe's util buttons: outlined, the accent on hover */
  .fui-Button:not(:has(.fui-Button__icon)) { border: 1px solid var(--og-b1) !important; }
  .fui-Button:not(:has(.fui-Button__icon)):not(:disabled):hover { border-color: var(--og-a1) !important; }

  /* inputs: one outline that takes the accent on focus, in place of Fluent's underline */
  :is(.fui-Input, .fui-Textarea, .fui-Dropdown, .fui-Combobox, .fui-SearchBox) {
    border: 1px solid var(--og-b1) !important; border-radius: .4rem !important;
  }
  :is(.fui-Input, .fui-Textarea, .fui-Dropdown, .fui-Combobox, .fui-SearchBox):focus-within { border-color: var(--og-a1) !important; }
  :is(.fui-Input, .fui-Textarea, .fui-Dropdown, .fui-Combobox, .fui-SearchBox)::after { display: none !important; }

  /* zwipe's eyebrow on menu group headers */
  .fui-MenuGroupHeader {
    text-transform: uppercase; letter-spacing: .08em; font-size: .75rem !important; color: var(--og-a3) !important;
  }

  /* | / - \\ spinner */
  .fui-Spinner__spinner {
    mask: none !important; -webkit-mask: none !important; animation: none !important; background: none !important;
    display: grid !important; place-items: center;
  }
  .fui-Spinner__spinner > *, .fui-Spinner__spinner::before { display: none !important; }
  .fui-Spinner__spinner::after {
    content: '|'; font: 700 1.2em ${FONT}; color: var(--og-a2);
    animation: og-spin .5s steps(1) infinite;
  }
  @keyframes og-spin { 0% { content: '|'; } 25% { content: '/'; } 50% { content: '-'; } 75% { content: '\\\\'; } }

  @media (prefers-reduced-motion: no-preference) {
    .fui-Button { transition: background-color .2s ease, border-color .2s ease, color .2s ease !important; }
  }
`;

