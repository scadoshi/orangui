// What every orangui userscript shares: the theme as variables, the theme picker, and the Tampermonkey menu.
// scripts/sync.mjs copies this file into each *.user.js between the core markers. A site's own code runs after it and calls start().

// The order of a theme's colors in THEMES. Keep in step with KEYS in scripts/sync.mjs.
const KEYS = [
  'bg', 'text', 'muted', 'subtle',
  'b1', 'b2', 'b3',
  'err', 'ok', 'warn',
  'berr', 'bok', 'bwarn',
  'a1', 'a2', 'a3',
  'p1', 'p2', 'p3', 'p4', 'p5', 'p6',
];

// == theme names, as zwipe-core's display_theme_name spells them ==

const COLORBLIND = ['protanopia', 'deuteranopia', 'tritanopia', 'achromatopsia'];
const SPELLED = {
  'rose-pine': 'Rosé Pine',
  vscode: 'VS Code',
  github: 'GitHub',
  'synthwave-84': "Synthwave '84",
  powershell: 'PowerShell',
  'docs-rs': 'docs.rs',
};
const label = (slug) => SPELLED[slug] ?? slug.split('-').map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');
const SLUGS = [...new Set(Object.keys(THEMES).map((k) => k.replace(/-(dark|light)$/, '')))].sort();
const REGULAR = SLUGS.filter((s) => !COLORBLIND.includes(s));
const DEFAULT = { name: 'gruvbox', dark: true };

const FONT = "'JetBrains Mono', 'JetBrainsMono Nerd Font', 'JetBrainsMono NF', monospace";
const TOP = window.top === window;

const v = (k) => `var(--og-${k})`;
const mix = (c, pct, base = v('bg')) => `color-mix(in srgb, ${c} ${pct}%, ${base})`;
const clear = (c, pct) => `color-mix(in srgb, ${c} ${pct}%, transparent)`;

// == stored state, shared by every site and tab the scripts run on ==

const valid = (t) => (t && THEMES[`${t.name}-${t.dark ? 'dark' : 'light'}`] ? t : DEFAULT);
let current = valid(GM_getValue('theme', DEFAULT));
let enabled = GM_getValue('enabled', true);
let showLauncher = GM_getValue('launcher', true);
const spotKey = `launcher@${location.hostname}`;
const HOME = { right: 12, bottom: 12 };
let picker = null;
let onEnabled = () => {};

// == the theme as variables: zwipe's names, prefixed so a page can't collide with them ==

function themeCss({ name, dark }) {
  const colors = THEMES[`${name}-${dark ? 'dark' : 'light'}`];
  const vars = KEYS.map((k, i) => `--og-${k}: ${colors[i]};`).join(' ');
  // zwipe's sink: the content area sits darker than the chrome on a dark theme, brighter on a light one.
  const derived = dark
    ? '--og-sink: color-mix(in srgb, var(--og-bg), #000 15%); --og-overlay: rgba(0,0,0,.6); --og-shadow-sm: 0 4px 12px rgba(0,0,0,.3); --og-shadow-md: 0 4px 12px rgba(0,0,0,.5);'
    : '--og-sink: color-mix(in srgb, var(--og-bg), #fff 35%); --og-overlay: rgba(0,0,0,.3); --og-shadow-sm: 0 4px 12px rgba(0,0,0,.1); --og-shadow-md: 0 4px 12px rgba(0,0,0,.15);';
  return `:root { ${vars} ${derived} color-scheme: ${dark ? 'dark' : 'light'}; }`;
}

// Rounds the selected elements into zwipe's squircle, or a rounded square where the browser has no corner-shape.
const squircle = (selector) => `
  :is(${selector}) { border-radius: 30% !important; }
  @supports (corner-shape: squircle) { :is(${selector}) { border-radius: 50% !important; corner-shape: squircle; } }
`;

const CORE_CSS = `
  input, textarea, [contenteditable] { caret-color: var(--og-a2); }
  ::selection { background: var(--og-a2); color: var(--og-bg); }
  * { scrollbar-width: thin; scrollbar-color: var(--og-b1) transparent; }

  /* zwipe's theme wipe: the new theme sweeps across, right to left when going light to dark */
  html[data-og-wipe]::view-transition-old(root), html[data-og-wipe]::view-transition-new(root) {
    animation: none; mix-blend-mode: normal;
  }
  html[data-og-wipe]::view-transition-new(root) { animation: og-wipe .4s cubic-bezier(.65, 0, .35, 1); }
  html[data-og-wipe="left"]::view-transition-new(root) { animation-name: og-wipe-left; }
  @keyframes og-wipe { from { clip-path: inset(0 100% 0 0); } to { clip-path: inset(0); } }
  @keyframes og-wipe-left { from { clip-path: inset(0 0 0 100%); } to { clip-path: inset(0); } }
`;

const baseStyle = document.createElement('style');
baseStyle.dataset.orangui = 'base';
const themeStyle = document.createElement('style');
themeStyle.dataset.orangui = 'theme';

// Swaps the theme in, inside zwipe's wipe when asked for and the browser can run it.
function apply(theme, wipe = false) {
  const leaving = current;
  current = theme;
  // Writes whatever is current when it runs, as a fast run of picks can land their swaps out of order.
  const swap = () => { themeStyle.textContent = themeCss(current); };
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (!wipe || !enabled || still || !document.startViewTransition) return swap();
  const root = document.documentElement;
  root.dataset.ogWipe = theme.dark && !leaving.dark ? 'left' : 'right';
  document.startViewTransition(swap).finished.finally(() => delete root.dataset.ogWipe);
}

function setEnabled(on) {
  enabled = on;
  if (on) document.documentElement.append(baseStyle, themeStyle);
  else { baseStyle.remove(); themeStyle.remove(); }
  onEnabled(on);
  picker?.sync();
}

// == the picker: zwipe's ThemeSheet, in a shadow root so the page's CSS can't reach it ==

const h = (tag, props = {}, ...kids) => {
  const el = document.createElement(tag);
  for (const [k, val] of Object.entries(props)) {
    if (k.startsWith('on')) el.addEventListener(k.slice(2), val);
    else if (k === 'style') el.style.cssText = val;
    else if (val !== false && val != null) el.setAttribute(k, val === true ? '' : val);
  }
  el.append(...kids.flat());
  return el;
};

const PICKER_CSS = `
  :host { all: initial; }
  * { box-sizing: border-box; font-family: ${FONT}; }

  .launcher {
    position: fixed; z-index: 2147483646; width: 1.6rem; height: 1.6rem; padding: 0;
    display: grid; place-items: center; touch-action: none; user-select: none;
    background: var(--og-bg); color: var(--og-text); border: 1px solid var(--og-b1); border-radius: .5rem;
    box-shadow: var(--og-shadow-sm); font-size: .9rem; line-height: 1; cursor: grab; opacity: .3;
    transition: opacity .2s, border-color .2s, color .2s;
  }
  .launcher:hover, .launcher:focus-visible { opacity: 1; border-color: var(--og-a1); color: var(--og-a1); outline: none; }
  .launcher.dragging { cursor: grabbing; opacity: 1; }

  .backdrop { position: fixed; inset: 0; z-index: 2147483646; background: var(--og-overlay); }
  .sheet {
    position: fixed; z-index: 2147483647; left: 50%; top: 50%;
    width: min(26rem, calc(100vw - 2rem)); max-height: min(40rem, calc(100vh - 2rem));
    display: flex; flex-direction: column; overflow: hidden;
    background: var(--og-bg); color: var(--og-text);
    border: 1px solid var(--og-b2); border-radius: 1rem; box-shadow: var(--og-shadow-md);
  }
  .modal-header {
    position: relative; display: flex; justify-content: center; align-items: center; flex-shrink: 0;
    padding: 1.25rem 1.5rem; border-bottom: 1px solid var(--og-b2); cursor: grab; touch-action: none; user-select: none;
  }
  .modal-header.dragging { cursor: grabbing; }
  .modal-title { font-size: 1rem; color: var(--og-a3); }
  .mode { position: absolute; right: 1rem; }

  .chip {
    padding: .3rem .6rem; border: 1px solid var(--og-b1); border-radius: .6rem; font-size: .75rem;
    color: var(--og-text); background: var(--og-bg); box-shadow: var(--og-shadow-sm); cursor: pointer;
    transition: all .2s ease;
  }
  .chip:hover, .chip:focus-visible { border-color: var(--og-a2); color: var(--og-a2); outline: none; }

  .modal-content { padding: 1rem 1.5rem; overflow-y: auto; flex: 1; display: flex; flex-direction: column; gap: .4rem; scrollbar-width: none; }
  .pref-row {
    display: flex; align-items: center; justify-content: space-between; gap: .5rem; width: 100%;
    padding: .6rem 1rem; background: var(--og-bg); border: 1px solid var(--og-b1); border-radius: .4rem;
    color: var(--og-muted); font-size: .85rem; text-align: left; cursor: pointer;
    transition: border-color .15s, color .15s;
  }
  .pref-row:hover, .pref-row:focus-visible { color: var(--og-text); outline: none; }
  .pref-row.selected { border-color: var(--og-a2); color: var(--og-text); }
  .theme-swatches {
    display: flex; align-items: center; gap: .2rem; flex-shrink: 0; padding: .2rem .25rem;
    border-radius: .35rem; background: var(--s-bg); border: 1px solid var(--s-muted);
  }
  .theme-dot { width: .7rem; height: .7rem; border-radius: .22rem; flex-shrink: 0; }
  .pref-section-label { font-size: 1rem; color: var(--og-a1); letter-spacing: .1em; text-align: center; margin: .75rem 0 .4rem; }

  .util-bar {
    flex-shrink: 0; display: flex; justify-content: center; align-items: center; gap: .5rem;
    padding: 1rem; border-top: 1px solid var(--og-b2);
  }
  .util-btn {
    padding: .4rem .8rem; font-size: .8rem; border-radius: .5rem; background: var(--og-bg);
    border: 1px solid var(--og-b1); color: var(--og-text); cursor: pointer; transition: all .1s ease;
  }
  .util-btn:hover:not(:disabled) { border-color: var(--og-a1); color: var(--og-a1); }
  .util-btn:disabled { opacity: .5; cursor: not-allowed; }
  .hint { font-size: .7rem; color: var(--og-warn); text-align: center; padding: 0 1rem .75rem; }
  .hint:empty { display: none; }
`;

// Hands the pointer's travel to onMove once it passes a few pixels. A press that stays put is a click.
function draggable(handle, onMove, onDrop) {
  handle.addEventListener('pointerdown', (e) => {
    if (e.button !== 0 || e.target.closest('button:not(.launcher)')) return;
    const start = { x: e.clientX, y: e.clientY };
    let moved = false;
    handle.setPointerCapture(e.pointerId);
    const move = (m) => {
      const dx = m.clientX - start.x;
      const dy = m.clientY - start.y;
      if (!moved && Math.hypot(dx, dy) < 4) return;
      moved = true;
      handle.classList.add('dragging');
      onMove(m, dx, dy);
    };
    const up = () => {
      handle.removeEventListener('pointermove', move);
      handle.classList.remove('dragging');
      if (moved) {
        onDrop?.();
        // Swallows the click that ends a drag.
        handle.addEventListener('click', (c) => { c.stopPropagation(); c.preventDefault(); }, { capture: true, once: true });
      }
    };
    handle.addEventListener('pointermove', move);
    handle.addEventListener('pointerup', up, { once: true });
    handle.addEventListener('pointercancel', up, { once: true });
  });
}

function makePicker() {
  const host = h('orangui-picker');
  const shadow = host.attachShadow({ mode: 'closed' });
  const style = h('style');
  style.textContent = PICKER_CSS;

  // The launcher keeps its spot per site, as distances from the right and bottom edges.
  let spot = GM_getValue(spotKey, HOME);
  const launcher = h('button', { class: 'launcher', title: 'orangui themes (Alt+Shift+T)', 'aria-label': 'Themes' }, '◐');
  const place = () => {
    const max = (n, room) => Math.min(Math.max(n, 0), Math.max(room - 26, 0));
    launcher.style.right = `${max(spot.right, innerWidth)}px`;
    launcher.style.bottom = `${max(spot.bottom, innerHeight)}px`;
  };
  let from;
  launcher.addEventListener('pointerdown', () => { from = { ...spot }; });
  draggable(
    launcher,
    (_e, dx, dy) => { spot = { right: from.right - dx, bottom: from.bottom - dy }; place(); },
    () => {
      spot = { right: parseFloat(launcher.style.right), bottom: parseFloat(launcher.style.bottom) };
      GM_setValue(spotKey, spot);
    },
  );
  launcher.addEventListener('click', () => open());
  addEventListener('resize', place);
  place();

  const backdrop = h('div', { class: 'backdrop', onclick: () => back() });
  const title = h('div', { class: 'modal-title', id: 'og-title' }, 'Themes');
  const mode = h('button', { class: 'chip mode', title: 'Dark or light (← →)', onclick: () => flip() });
  const header = h('div', { class: 'modal-header' }, title, mode);
  const list = h('div', { class: 'modal-content' });
  const save = h('button', { class: 'util-btn', onclick: () => keep() }, 'Save');
  const hint = h('div', { class: 'hint' });
  const sheet = h(
    'div',
    { class: 'sheet', role: 'dialog', 'aria-modal': 'true', 'aria-labelledby': 'og-title' },
    header,
    list,
    h('div', { class: 'util-bar' }, h('button', { class: 'util-btn', onclick: () => back() }, 'Back'), save),
    hint,
  );

  // The sheet's offset from center, kept while the page stays open.
  let offset = { x: 0, y: 0 };
  let grab;
  const position = () => { sheet.style.transform = `translate(calc(-50% + ${offset.x}px), calc(-50% + ${offset.y}px))`; };
  header.addEventListener('pointerdown', () => { grab = { ...offset }; });
  draggable(header, (_e, dx, dy) => { offset = { x: grab.x + dx, y: grab.y + dy }; position(); });
  position();

  let original = null;
  let returnFocus = null;
  const isOpen = () => original !== null;
  const same = (a, b) => a.name === b.name && a.dark === b.dark;

  const row = (slug, dark) => {
    const colors = THEMES[`${slug}-${dark ? 'dark' : 'light'}`];
    const at = (k) => colors[KEYS.indexOf(k)];
    const selected = current.name === slug;
    return h(
      'button',
      { class: selected ? 'pref-row selected' : 'pref-row', 'aria-pressed': String(selected), onclick: () => pick({ name: slug, dark }) },
      h('span', {}, label(slug)),
      h(
        'span',
        { class: 'theme-swatches', style: `--s-bg: ${at('bg')}; --s-muted: ${at('muted')}` },
        ['p1', 'p2', 'p3', 'p4', 'p5', 'p6'].map((k) => h('span', { class: 'theme-dot', style: `background: ${at(k)}` })),
      ),
    );
  };

  // The picker's order: the regular themes, then the colorblind ones under their own label.
  const ORDER = [...REGULAR, ...COLORBLIND.filter((s) => SLUGS.includes(s))];

  function render() {
    const { dark } = current;
    mode.textContent = dark ? '☾ dark' : '☀ light';
    mode.setAttribute('aria-pressed', String(dark));
    list.replaceChildren(
      ...REGULAR.map((s) => row(s, dark)),
      h('div', { class: 'pref-section-label' }, 'Colorblind'),
      ...ORDER.slice(REGULAR.length).map((s) => row(s, dark)),
    );
    save.disabled = same(current, original);
  }

  // A held arrow key skips the wipe, so a fast run through the list doesn't queue up sweeps.
  function pick(theme, wipe = true) {
    if (same(theme, current)) return;
    apply(theme, wipe);
    render();
    const selected = list.querySelector('.selected');
    selected?.focus({ preventScroll: true });
    selected?.scrollIntoView({ block: 'nearest' });
  }

  const flip = (wipe = true) => pick({ ...current, dark: !current.dark }, wipe);
  const jump = (to, wipe = true) => pick({ ...current, name: ORDER[(to + ORDER.length) % ORDER.length] }, wipe);
  const at = () => ORDER.indexOf(current.name);

  function open() {
    if (isOpen()) return;
    if (!enabled) setEnabled(true);
    original = current;
    returnFocus = document.activeElement;
    // Dark Reader rewrites the same colors, and the two fight.
    hint.textContent = document.documentElement.hasAttribute('data-darkreader-mode')
      ? 'Dark Reader is on here. Turn it off for this site so the themes come through.'
      : '';
    render();
    shadow.append(backdrop, sheet);
    launcher.remove();
    (list.querySelector('.selected') ?? list.firstElementChild)?.focus({ preventScroll: true });
    list.querySelector('.selected')?.scrollIntoView({ block: 'center' });
  }

  function close() {
    original = null;
    backdrop.remove();
    sheet.remove();
    sync();
    returnFocus?.focus?.();
  }

  function back() {
    if (!isOpen()) return;
    if (!same(current, original)) apply(original, true);
    close();
  }

  function keep() {
    GM_setValue('theme', current);
    close();
  }

  function sync() {
    if (showLauncher && enabled && !isOpen()) shadow.append(launcher);
    else launcher.remove();
  }

  function home() {
    spot = HOME;
    GM_setValue(spotKey, spot);
    place();
  }

  // Capture phase, so the page never sees the keys the picker answers.
  addEventListener('keydown', (e) => {
    const toggle = e.altKey && e.shiftKey && !e.ctrlKey && !e.metaKey && e.code === 'KeyT';
    const plain = !e.altKey && !e.ctrlKey && !e.metaKey;
    const keys = {
      Escape: () => back(),
      Enter: () => (save.disabled ? close() : keep()),
      ArrowDown: () => jump(at() + 1, !e.repeat),
      ArrowUp: () => jump(at() - 1, !e.repeat),
      Home: () => jump(0),
      End: () => jump(-1),
      ArrowLeft: () => flip(!e.repeat),
      ArrowRight: () => flip(!e.repeat),
    };
    if (!toggle && !(isOpen() && plain && keys[e.key])) return;
    e.preventDefault();
    e.stopPropagation();
    if (toggle) isOpen() ? back() : open();
    else keys[e.key]();
  }, true);

  shadow.append(style);
  sync();
  const mount = () => document.documentElement.append(host);
  if (document.readyState === 'loading') addEventListener('DOMContentLoaded', mount, { once: true });
  else mount();

  return { open, isOpen, sync, home };
}

// == the recolor: color literals in a page's stylesheets and inline styles, sent to the nearest theme color ==

// Each literal is read as the light app it was written for and swapped for a theme variable, so a theme change needs no second pass.
// A site that themes its grays some other way (Fluent's tokens) asks for the saturated colors only.

const COLOR = /#[0-9a-f]{3,8}\b|rgba?\([^)]*\)|\b(?:white|black)\b/gi;

function parse(text) {
  const t = text.toLowerCase();
  if (t === 'white') return { r: 255, g: 255, b: 255, a: 1 };
  if (t === 'black') return { r: 0, g: 0, b: 0, a: 1 };
  if (t[0] === '#') {
    let hex = t.slice(1);
    if (hex.length <= 4) hex = [...hex].map((c) => c + c).join('');
    if (hex.length !== 6 && hex.length !== 8) return null;
    const n = (i) => parseInt(hex.slice(i, i + 2), 16);
    return { r: n(0), g: n(2), b: n(4), a: hex.length === 8 ? n(6) / 255 : 1 };
  }
  const parts = t.match(/[\d.]+%?/g);
  if (!parts || parts.length < 3) return null;
  const alpha = parts[3] === undefined ? 1 : parts[3].endsWith('%') ? parseFloat(parts[3]) / 100 : parseFloat(parts[3]);
  return { r: +parts[0], g: +parts[1], b: +parts[2], a: alpha };
}

// Hue in degrees, HSL saturation and lightness 0 to 1, and the lightness over white, as the light app would show it.
function measure({ r, g, b, a }) {
  const [R, G, B] = [r / 255, g / 255, b / 255];
  const max = Math.max(R, G, B);
  const min = Math.min(R, G, B);
  const l = (max + min) / 2;
  const d = max - min;
  const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1));
  let hue = 0;
  if (d) hue = max === R ? ((G - B) / d) % 6 : max === G ? (B - R) / d + 2 : (R - G) / d + 4;
  return { hue: (hue * 60 + 360) % 360, s, l, over: l * a + (1 - a), under: l * a };
}

const hueColor = (hue) =>
  hue < 15 || hue >= 335 ? 'err' : hue < 40 ? 'p2' : hue < 70 ? 'warn' : hue < 160 ? 'ok' : hue < 190 ? 'p6' : hue < 255 ? 'a1' : 'p4';

// Returns the theme expression for one color literal in a property of the given role, or null to leave it alone.
// `dark` says the color was written for a dark page, where the gray ladder runs the other way.
function recolor(text, role, neutrals, dark) {
  const c = parse(text);
  if (!c || c.a === 0) return null;
  const { hue, s, l, over, under } = measure(c);
  const neutral = s < 0.18 || l > 0.94 || l < 0.08;
  if (neutral && !neutrals) return null;

  if (role === 'shadow') return neutral ? null : clear(v(hueColor(hue)), Math.round(c.a * 100));

  if (!neutral) {
    const theme = v(hueColor(hue));
    if (role !== 'bg') return c.a < 1 ? clear(theme, Math.round(c.a * 100)) : theme;
    // zwipe never fills with a full accent: a tint of it, pastels fainter still.
    const tint = mix(theme, l > 0.85 ? 12 : 22);
    return c.a < 0.5 ? clear(theme, Math.round(c.a * 40)) : tint;
  }

  if (dark) return darkGray(c, role, l, under);

  if (role === 'bg') {
    // A dim black wash is a modal backdrop or a shadow layer, and stays one.
    if (l < 0.2 && c.a >= 0.3 && c.a < 0.95) return null;
    if (over >= 0.985) return c.a < 1 ? clear(v('bg'), Math.round(c.a * 100)) : v('bg');
    if (over >= 0.93) return v('sink');
    if (over >= 0.7) return v('b2');
    if (over >= 0.3) return v('b1');
    return v('b2');
  }
  if (role === 'border') return over >= 0.75 ? v('b2') : v('b1');
  // Text: near black and near white are both body text, the grays between them step down.
  if (over < 0.3 || over > 0.85) return v('text');
  if (over < 0.45) return v('subtle');
  return v('muted');
}

// A gray from a dark page, by its lightness over black: Halo's dark theme is about 21% for the page, 26% for panels, 32% for raised and hover.
function darkGray(c, role, l, under) {
  if (role === 'bg') {
    if (l < 0.2 && c.a >= 0.3 && c.a < 0.95) return null;
    // A faint white wash lightens whatever is under it.
    if (l > 0.8 && c.a < 1) return clear(v('text'), Math.round(c.a * 30));
    if (under < 0.235) return v('sink');
    if (under < 0.3) return v('bg');
    if (under < 0.38) return v('b2');
    if (under < 0.75) return v('b1');
    return v('bg');
  }
  if (role === 'border') return under < 0.36 ? v('b2') : v('b1');
  if (under > 0.78 || under < 0.25) return v('text');
  if (under > 0.6) return v('subtle');
  return v('muted');
}

const roleOf = (prop) => {
  // A custom property's role is a guess from its name: Teams' --colorBrandBackground, --toastify-color-light.
  if (prop.startsWith('--')) {
    if (/shadow/i.test(prop)) return 'shadow';
    if (/background|bg|fill|surface/i.test(prop)) return 'bg';
    if (/stroke|border|outline/i.test(prop)) return 'border';
    return /color|colour|brand|foreground/i.test(prop) ? 'fg' : null;
  }
  if (prop === 'background-color' || prop === 'background-image') return 'bg';
  if (prop === 'box-shadow') return 'shadow';
  if (/^(border|outline|column-rule)(-\w+)*-color$/.test(prop)) return 'border';
  if (prop === 'color' || prop === 'fill' || prop === 'stroke' || /^(text-decoration|caret|text-emphasis)-color$/.test(prop)) return 'fg';
  return null;
};

const ICON_FONT = /awesome|icons?\b|gantt|froala|dashicons|material|glyph/i;

// zwipe's radii: 0.4rem inputs, 0.6rem chips and rows, 1rem panels.
function radius(value) {
  const px = value.match(/^([\d.]+)px$/);
  if (!px) return null;
  const n = +px[1];
  if (n <= 0 || n >= 40) return null;
  return n <= 6 ? '.4rem' : n <= 12 ? '.6rem' : '1rem';
}

// What recolorPage was asked to touch: the grays, font families, and border radii, on top of the saturated colors it always takes.
// `darkPage` is a selector that, when it matches, says inline styles were written for the site's own dark mode.
let paint = { neutrals: false, fonts: false, radii: false, darkPage: null };

// The original of every stylesheet declaration rewritten, so turning orangui off can put the page back.
const saved = [];

function restyle(style, keep, dark) {
  for (let i = 0; i < style.length; i++) {
    const prop = style[i];
    const value = style.getPropertyValue(prop);
    if (!value || value.includes('--og-')) continue;
    let next = null;
    const role = roleOf(prop);
    if (role && !(role === 'bg' && value.includes('url('))) {
      const swapped = value.replace(COLOR, (m) => recolor(m, role, paint.neutrals, dark) ?? m);
      if (swapped !== value) next = swapped;
    } else if (paint.fonts && prop === 'font-family' && !ICON_FONT.test(value)) {
      next = FONT;
    } else if (paint.radii && /^border-.*-radius$/.test(prop)) {
      next = radius(value);
    }
    if (next === null || next === value) continue;
    const priority = style.getPropertyPriority(prop);
    if (keep) saved.push([style, prop, value, priority]);
    style.setProperty(prop, next, priority);
  }
}

const seen = new WeakMap();
// A rule whose selector or media query mentions dark (.theme-dark, .fr-dark, prefers-color-scheme: dark) was written for a dark page.
const walk = (rules, dark = false) => {
  for (const rule of rules) {
    const inDark = dark || /dark/i.test(rule.selectorText ?? rule.conditionText ?? '');
    if (rule.style) restyle(rule.style, true, inDark);
    if (rule.cssRules) walk(rule.cssRules, inDark);
  }
};

// Rewrites every sheet that is new or has grown. Cross-origin sheets can't be read and are skipped.
function scan() {
  if (!enabled) return;
  for (const sheet of [...document.styleSheets, ...document.adoptedStyleSheets]) {
    if (sheet.ownerNode?.dataset?.orangui) continue;
    let rules;
    try { rules = sheet.cssRules; } catch { continue; }
    if (seen.get(sheet) === rules.length) continue;
    seen.set(sheet, rules.length);
    walk(rules);
  }
}

// Inline styles come and go with every React render, so they are rewritten as they appear and never restored.
// SVG color attributes (FontAwesome's color="white") are carried into the inline style, which outranks them.
const PAINTED = ['color', 'fill', 'stroke'];
const INLINE = '[style], svg[color], svg [fill], svg [stroke]';
const pending = new Set();
let frame = 0;
const flush = () => {
  frame = 0;
  const dark = Boolean(paint.darkPage && document.querySelector(paint.darkPage));
  for (const el of pending) {
    if (!el.style) continue;
    for (const attr of PAINTED) {
      const value = el.getAttribute(attr);
      if (!value || el.style.getPropertyValue(attr)) continue;
      const next = value.replace(COLOR, (m) => recolor(m, 'fg', paint.neutrals, dark) ?? m);
      if (next !== value) el.style.setProperty(attr, next);
    }
    restyle(el.style, false, dark);
  }
  pending.clear();
};
const queue = (el) => {
  pending.add(el);
  frame ||= requestAnimationFrame(flush);
};

const watcher = new MutationObserver((records) => {
  let sheets = false;
  for (const r of records) {
    if (r.type === 'attributes') {
      // An attribute change means the page re-rendered it; the stale rewrite of that attribute comes off.
      if (r.attributeName !== 'style') r.target.style?.removeProperty(r.attributeName);
      queue(r.target);
      continue;
    }
    for (const node of r.addedNodes) {
      if (node.nodeType !== 1) continue;
      if (node.matches('style, link[rel="stylesheet"]')) {
        sheets = true;
        node.addEventListener('load', scan, { once: true });
      }
      if (node.matches(INLINE)) queue(node);
      node.querySelectorAll?.(INLINE).forEach(queue);
    }
  }
  if (sheets) requestAnimationFrame(scan);
});

let timer = 0;
function run(on) {
  if (on && timer) return;
  if (on) {
    watcher.observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ['style', ...PAINTED] });
    document.querySelectorAll(INLINE).forEach(queue);
    scan();
    // Emotion and react-select add rules through insertRule, which no observer sees.
    timer = setInterval(() => { if (!document.hidden) scan(); }, 1500);
  } else {
    watcher.disconnect();
    clearInterval(timer);
    timer = 0;
    for (const [style, prop, value, priority] of saved) style.setProperty(prop, value, priority);
    saved.length = 0;
    for (const sheet of document.styleSheets) seen.delete(sheet);
  }
}

// Sets what the recolor touches and returns the switch a site hands to start() as onEnabled.
function recolorPage(options) {
  paint = { ...paint, ...options };
  return run;
}

// == start: a site script calls this once with its own CSS ==

function start({ css, onEnabled: hook }) {
  baseStyle.textContent = css + CORE_CSS;
  if (hook) onEnabled = hook;
  apply(current);
  setEnabled(enabled);

  GM_addValueChangeListener('theme', (_key, _old, next, remote) => {
    if (remote && !picker?.isOpen()) apply(valid(next), true);
  });
  GM_addValueChangeListener('enabled', (_key, _old, next, remote) => { if (remote) setEnabled(next); });
  GM_addValueChangeListener('launcher', (_key, _old, next, remote) => {
    if (remote) { showLauncher = next; picker?.sync(); }
  });

  if (!TOP) return;
  picker = makePicker();
  GM_registerMenuCommand('Themes… (Alt+Shift+T)', () => picker.open());
  GM_registerMenuCommand('Show or hide the ◐ launcher', () => {
    showLauncher = !showLauncher;
    GM_setValue('launcher', showLauncher);
    picker.sync();
  });
  GM_registerMenuCommand('Put the ◐ back in the corner', () => picker.home());
  GM_registerMenuCommand('Turn orangui on or off', () => {
    setEnabled(!enabled);
    GM_setValue('enabled', enabled);
  });
}
