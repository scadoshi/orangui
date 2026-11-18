// A toast leaving drifts up a little from where it sat and fades out, like a puff of smoke, by
// transform and opacity alone; under reduced motion it only fades. The toast library takes a
// toast's element out of the page outright, with no closing state, so this keeps each shown
// toast's last box and, as one goes, fades a copy of it from there. A timed toast that already
// faded by its own keyframes is let go without one. The newest toast leaves in front of the rest,
// an older one behind them.
(() => {
    if (window.__oranguiToastLeave) return;
    window.__oranguiToastLeave = true;
    const seen = new Map();
    let frame = 0;
    // Each shown toast's box and whether it has already faded out, every frame while any is up:
    // only the box it last had is left to fade from once the library removes it.
    const measure = () => {
        const items = [...document.querySelectorAll('.toast-container:not(.toast-ghost) li')];
        items.forEach((li, index) => {
            const toast = li.querySelector('[role="alertdialog"]');
            if (!toast) return;
            const box = toast.getBoundingClientRect();
            const style = getComputedStyle(toast);
            const gone = style.animationName.startsWith('toast-life') && Number(style.opacity) <= 0.02;
            seen.set(li, { box, front: index === 0, gone });
        });
        frame = items.length ? requestAnimationFrame(measure) : 0;
    };
    const watch = () => {
        if (!frame) frame = requestAnimationFrame(measure);
    };
    const fade = (li) => {
        const last = seen.get(li);
        seen.delete(li);
        if (!last || last.gone) return;
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
        const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
        const frames = still
            ? [{ opacity: 1 }, { opacity: 0 }]
            : [{ transform: 'translateY(0)', opacity: 1 }, { transform: 'translateY(-0.75rem)', opacity: 0 }];
        const motion = copy.animate(frames, { duration: 200, easing: 'ease-out' });
        motion.onfinish = () => holder.remove();
        motion.oncancel = () => holder.remove();
    };
    new MutationObserver((records) => {
        for (const record of records) {
            record.removedNodes.forEach((node) => {
                if (seen.has(node)) fade(node);
            });
            if (record.addedNodes.length) watch();
        }
    }).observe(document.body, { childList: true, subtree: true });
    watch();
})();
