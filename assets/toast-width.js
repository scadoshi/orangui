// The collapsed stack takes the newest toast's width, so a toast arriving or leaving changes
// every toast's width at once, and tapping the stack open lets each one hug its own message.
// This eases each toast from the width it had to the one it now takes instead of jumping there.
// Widths are layout widths (offsetWidth), so the arrival's scale never reads as a change.
(() => {
    if (window.__oranguiToastWidth) return;
    window.__oranguiToastWidth = true;
    const EASE_MS = 200;
    const widths = new Map();
    const easing = new Map();
    let frame = 0;
    const live = () => [...document.querySelectorAll('.toast-container:not(.toast-ghost) li')];
    const toastOf = (li) => li.querySelector('[role="alertdialog"]');
    // Each toast's width every frame while any is up, so a change knows where it eases from.
    const measure = () => {
        const items = live();
        for (const li of widths.keys()) {
            if (!li.isConnected) widths.delete(li);
        }
        items.forEach((li) => {
            const toast = toastOf(li);
            if (toast) widths.set(li, toast.offsetWidth);
        });
        frame = items.length ? requestAnimationFrame(measure) : 0;
    };
    const watch = () => {
        if (!frame) frame = requestAnimationFrame(measure);
    };
    // Whether a message sits on one line at its new width, so it can be held to one line while
    // its toast eases without that changing its height.
    const oneLine = (toast) => {
        const label = toast.querySelector('[id$="-label"]');
        if (!label) return true;
        const range = document.createRange();
        range.selectNodeContents(label);
        const tops = new Set([...range.getClientRects()].map((r) => Math.round(r.top)));
        return tops.size <= 1;
    };
    const ease = () => {
        if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;
        live().forEach((li) => {
            const toast = toastOf(li);
            const from = widths.get(li);
            // A toast just landing has no width to ease from; it arrives at its own.
            if (!toast || from === undefined) return;
            const running = easing.get(toast);
            if (running) running.cancel();
            const to = toast.offsetWidth;
            if (Math.abs(from - to) < 1) return;
            const hold = oneLine(toast);
            if (hold) toast.classList.add('toast-easing');
            const motion = toast.animate([{ width: `${from}px` }, { width: `${to}px` }], {
                duration: EASE_MS,
                easing: 'ease-out',
            });
            easing.set(toast, motion);
            const done = () => {
                if (easing.get(toast) !== motion) return;
                easing.delete(toast);
                toast.classList.remove('toast-easing');
            };
            motion.onfinish = done;
            motion.oncancel = done;
        });
    };
    new MutationObserver((records) => {
        let changed = false;
        for (const record of records) {
            const at = record.target.nodeType === 1 ? record.target : null;
            if (!at || at.closest('.toast-ghost')) continue;
            if (record.type === 'attributes') {
                if (at.classList.contains('toast-container')) changed = true;
            } else if (at.closest('.toast-container')) {
                const lis = [...record.addedNodes, ...record.removedNodes];
                if (lis.some((n) => n.nodeType === 1 && n.tagName === 'LI')) changed = true;
            }
            if (record.addedNodes.length) watch();
        }
        if (changed) ease();
    }).observe(document.body, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ['class'],
    });
    watch();
})();
