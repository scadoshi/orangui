// orangui::Splash on a prerendered page: runs the opening from first paint,
// before the app is up. The mark builds in the two waves `Decode` draws,
// leaves the same way in reverse, and the overlay fades, on the timings the
// splash element carries in its data attributes. The app waits on
// `window.__oranguiSplash.out`, which settles as the fade starts.
//
// It drives the overlay through classes on the root, so the host's sheet
// styles them and nothing here touches an element the app renders:
// `splash-running` once the mark is blanked and ready to build, then
// `splash-fading` as the fade starts and `splash-over` once it has ended.
(() => {
    if (window.__oranguiSplash) return;
    let settle;
    window.__oranguiSplash = { out: new Promise((resolve) => (settle = resolve)) };

    const NOISE = '░▒▓▎▍';
    const NOISE_PCT = 45;          // the share of a pass the noise wave takes to cover the art
    const DECODE_DELAY_PCT = 35;   // how far into a pass the decode wave starts
    const OVERRUN = 4;             // columns the decode front runs past the end

    // xorshift, enough randomness for a flicker.
    const flicker = (seed) => {
        let state = (Math.imul(seed, 0x9e3779b9) | 1) >>> 0;
        return () => {
            state ^= state << 13;
            state ^= state >>> 17;
            state ^= state << 5;
            return state >>> 0;
        };
    };

    // Where the two waves stand at `frame` of `frames`.
    const fronts = (columns, frame, frames) => {
        const pct = frame * 100;
        const noise = Math.min(Math.floor(((columns + 1) * pct) / Math.max(frames * NOISE_PCT, 1)), columns + 1);
        const decode = Math.floor(((columns + OVERRUN) * Math.max(pct - frames * DECODE_DELAY_PCT, 0)) / Math.max(frames * (100 - DECODE_DELAY_PCT), 1));
        return [noise, decode];
    };

    // Each glyph of `text` as `pick` says, given its column and a wobble of
    // up to three columns; spaces stay spaces.
    const map = (text, seed, pick) => {
        const next = flicker(seed);
        return text.split('\n').map((line) => {
            let out = '';
            for (let column = 0; column < line.length; column++) {
                const glyph = line[column];
                if (glyph === ' ') {
                    out += ' ';
                    continue;
                }
                out += pick(glyph, column + (next() % 4), () => NOISE[next() % 5]);
            }
            return out;
        }).join('\n');
    };

    // On the way in: glyphs behind the decode front, noise up to the noise
    // front, blank past it.
    const decoded = (text, decode, noise, seed) =>
        map(text, seed, (glyph, at, noisy) => (at < decode ? glyph : at >= noise ? ' ' : noisy()));
    // On the way out: blank behind the clear front, noise up to the noise
    // front, the art past it.
    const encoded = (text, noise, clear, seed) =>
        map(text, seed, (glyph, at, noisy) => (at < clear ? ' ' : at < noise ? noisy() : glyph));
    const blank = (text) => text.replace(/[^\n]/g, ' ');

    function init() {
        const root = document.documentElement;
        const splash = document.querySelector('.splash[data-prerendered]');
        if (!splash || root.classList.contains('splash-skip')) {
            settle();
            return;
        }
        const at = (key) => parseInt(splash.dataset[key], 10) || 0;
        const tick = at('tick') || 40;
        const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

        // The mark's text sits after the comment the prerender marks it with.
        let node = null;
        const walker = document.createTreeWalker(splash.querySelector('.splash-mark') || splash, NodeFilter.SHOW_TEXT);
        while (!node && walker.nextNode()) {
            if (walker.currentNode.data.trim()) node = walker.currentNode;
        }
        const real = node ? node.data : '';
        const columns = real.split('\n').reduce((widest, line) => Math.max(widest, line.length), 0);
        const write = (text) => {
            if (node) node.data = text;
        };

        // One pass over `ms`, a frame every tick, ending on `last`.
        const pass = (ms, frame, last) => {
            const frames = Math.max(Math.ceil(ms / tick), 1);
            let done = 0;
            const timer = setInterval(() => {
                done++;
                if (done >= frames) {
                    clearInterval(timer);
                    write(last);
                } else {
                    write(frame(done, frames));
                }
            }, tick);
        };

        write(blank(real));
        root.classList.add('splash-running');
        setTimeout(() => {
            if (reduced) {
                write(real);
                return;
            }
            pass(at('decode'), (frame, frames) => {
                const [noise, decode] = fronts(columns, frame, frames);
                return decoded(real, decode, noise, frame);
            }, real);
        }, at('markAt'));
        if (!reduced) {
            setTimeout(() => {
                pass(at('exit'), (frame, frames) => {
                    const [noise, clear] = fronts(columns, frame, frames);
                    return encoded(real, noise, clear, frame);
                }, blank(real));
            }, at('leaveAt'));
        }
        setTimeout(() => {
            root.classList.add('splash-fading');
            settle();
            setTimeout(() => root.classList.add('splash-over'), at('fade'));
        }, at('outAt'));
    }

    if (document.readyState === 'loading') addEventListener('DOMContentLoaded', init);
    else init();
})();
