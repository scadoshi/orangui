// A toast leaving drops from where it sat to below the screen's bottom edge, by transform alone.
// The toast library takes a toast's element out of the page outright, with no closing state, so
// this keeps each shown toast's last box and, as one goes, drops a copy of it from there. A timed
// toast that already dropped by its own keyframes, and any toast under reduced motion, is let go
// without one. The newest toast falls in front of the rest, an older one behind them.
(() => {
    if (window.__oranguiToastLeave) return;
    window.__oranguiToastLeave = true;
    const seen = new Map();
    let frame = 0;
    // Each shown toast's box and whether it has already gone below the edge, every frame while any
    // is up: only the box it last had is left to drop from once the library removes it.
    const measure = () => {
        const items = [...document.querySelectorAll('.toast-container:not(.toast-ghost) li')];
        items.forEach((li, index) => {
            const toast = li.querySelector('[role="alertdialog"]');
            if (!toast) return;
            const box = toast.getBoundingClientRect();
            const gone = getComputedStyle(toast).animationName.startsWith('toast-life') && box.top >= innerHeight - 1;
            seen.set(li, { box, front: index === 0, gone });
        });
        frame = items.length ? requestAnimationFrame(measure) : 0;
    };
    const watch = () => {
        if (!frame) frame = requestAnimationFrame(measure);
    };
    const drop = (li) => {
        const last = seen.get(li);
        seen.delete(li);
        if (!last || last.gone || matchMedia('(prefers-reduced-motion: reduce)').matches) return;
        const { box, front } = last;
        const holder = document.createElement('div');
        holder.className = 'toast-container toast-ghost';
        holder.setAttribute('aria-hidden', 'true');
        holder.style.cssText = `left: ${box.left}px; top: ${box.top}px; bottom: auto; max-width: none; pointer-events: none; z-index: ${front ? '10000' : '9998'};`;
        const list = document.createElement('ol');
        const copy = li.cloneNode(true);
        list.appendChild(copy);
        holder.appendChild(list);
        const toast = copy.querySelector('[role="alertdialog"]');
        if (toast) {
            toast.style.animation = 'none';
            toast.style.width = `${box.width}px`;
        }
        document.body.appendChild(holder);
        const fall = innerHeight - box.top + 16;
        const motion = copy.animate(
            [{ transform: 'translateY(0)' }, { transform: `translateY(${fall}px)` }],
            { duration: 220, easing: 'cubic-bezier(0.4, 0, 1, 1)' },
        );
        motion.onfinish = () => holder.remove();
        motion.oncancel = () => holder.remove();
    };
    new MutationObserver((records) => {
        for (const record of records) {
            record.removedNodes.forEach((node) => {
                if (seen.has(node)) drop(node);
            });
            if (record.addedNodes.length) watch();
        }
    }).observe(document.body, { childList: true, subtree: true });
    watch();
})();
