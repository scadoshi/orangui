// orangui::GridCells: draws the backdrop field on a canvas. One element per
// layer instead of a node per cell, so the rain is a short frame loop that
// stops when the last cell lands, and a still layer is one draw.
//
// Colors come from the layer's computed style (its `color` is the ink, set
// to the theme's text color), so a theme change redraws every layer in the
// new colors without animating it again. The pattern is seeded, so it is the
// same field on every load.
(() => {
    if (window.__oranguiGridCells) return;
    window.__oranguiGridCells = true;

    const RAIN_STEP_MS = 18;   // how far apart the rows land
    const JITTER_MS = 160;     // per-cell scatter inside its row
    const DROP_MS = 350;       // one cell's fall
    const DROP_CELLS = 3;      // how far it falls
    const SHADES = [0.015, 0.03, 0.05, 0.075, 0.1];
    const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const layers = new Map();

    const ease = (t) => 1 - Math.pow(1 - t, 3);
    const lcg = (seed) => () => {
        seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
        return seed >>> 8;
    };

    function setup(canvas) {
        if (layers.has(canvas)) return;
        const layer = canvas.parentElement;
        const state = {
            canvas, layer, cells: [], cell: 28, dpr: 1, raf: 0, ink: '',
            start: performance.now(),
            still: reduced || layer.hasAttribute('data-still'),
        };
        layers.set(canvas, state);
        state.ro = new ResizeObserver(() => {
            size(state);
            draw(state, state.raf === 0);
        });
        state.ro.observe(layer);
        size(state);
        if (state.still) draw(state, true);
        else animate(state);
    }

    function teardown(canvas) {
        const state = layers.get(canvas);
        if (!state) return;
        state.ro.disconnect();
        if (state.raf) cancelAnimationFrame(state.raf);
        layers.delete(canvas);
    }

    function size(state) {
        const { canvas, layer } = state;
        const w = layer.clientWidth, h = layer.clientHeight;
        state.dpr = window.devicePixelRatio || 1;
        canvas.width = Math.round(w * state.dpr);
        canvas.height = Math.round(h * state.dpr);
        state.cell = parseFloat(getComputedStyle(layer).getPropertyValue('--grid-size')) || 28;
        const cols = Math.ceil(w / state.cell), rows = Math.ceil(h / state.cell);
        const next = lcg(0x9e3779b9);
        const cells = [];
        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                const roll = next();
                cells.push({ c, r, shade: SHADES[roll % 5], at: r * RAIN_STEP_MS + (roll >>> 4) % JITTER_MS });
            }
        }
        state.cells = cells;
    }

    // Paints the field at `now`, or finished when `done`. Returns whether
    // any cell is still falling.
    function draw(state, done, now = performance.now()) {
        const { canvas, cells, cell, dpr } = state;
        const ctx = canvas.getContext('2d');
        state.ink = getComputedStyle(state.layer).color;
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        ctx.clearRect(0, 0, canvas.width / dpr, canvas.height / dpr);
        ctx.fillStyle = state.ink;
        const w = cell - 2, radius = w * 0.18, t = now - state.start;
        let falling = false;
        for (const k of cells) {
            let p = done ? 1 : (t - k.at) / DROP_MS;
            if (p <= 0) { falling = true; continue; }
            if (p < 1) falling = true; else p = 1;
            const e = ease(p);
            ctx.globalAlpha = k.shade * e;
            ctx.beginPath();
            ctx.roundRect(k.c * cell + 1, k.r * cell + 1 - (1 - e) * DROP_CELLS * cell, w, w, radius);
            ctx.fill();
        }
        ctx.globalAlpha = 1;
        return falling;
    }

    function animate(state) {
        const step = (now) => {
            state.raf = draw(state, false, now) ? requestAnimationFrame(step) : 0;
        };
        state.raf = requestAnimationFrame(step);
    }

    // New layers (a sheet opening, the splash) get drawn; removed ones let go.
    // A class change anywhere (the theme wrapper, most of all) redraws the
    // settled layers whose ink changed, with no rain.
    let recolor = 0;
    const observer = new MutationObserver((records) => {
        for (const r of records) {
            if (r.type === 'childList') {
                for (const n of r.addedNodes) {
                    if (n.nodeType !== 1) continue;
                    if (n.matches('.grid-cells-canvas')) setup(n);
                    n.querySelectorAll?.('.grid-cells-canvas').forEach(setup);
                }
                for (const n of r.removedNodes) {
                    if (n.nodeType !== 1) continue;
                    if (n.matches('.grid-cells-canvas')) teardown(n);
                    n.querySelectorAll?.('.grid-cells-canvas').forEach(teardown);
                }
            } else if (!recolor) {
                recolor = requestAnimationFrame(() => {
                    recolor = 0;
                    for (const state of layers.values()) {
                        if (state.raf === 0 && getComputedStyle(state.layer).color !== state.ink) draw(state, true);
                    }
                });
            }
        }
    });

    function init() {
        document.querySelectorAll('.grid-cells-canvas').forEach(setup);
        observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['class'] });
    }
    if (document.body) init();
    else addEventListener('DOMContentLoaded', init);
})();
