// A toast leaving drifts up a little from where it sat and fades out, like a puff of smoke, by
// transform and opacity alone; under reduced motion it only fades. The toast library takes a
// toast's element out of the page outright, with no closing state, so this keeps each shown
// toast's last box and, as one goes, fades a copy of it from there. A timed toast that already
// faded by its own keyframes is let go without one.
//
// The copy keeps the leaving toast's place in the stack's order for its whole leave: it sits
// beside the live toasts' container, in the same stacking context, so an app shell that isolates
// its layers can't lift it over them. It goes behind every live toast when a newer one is still
// up, read off the list as the toast leaves it, and in front when none is.
(() => {
    if (window.__oranguiToastLeave) return;
    window.__oranguiToastLeave = true;
    const seen = new Map();
    let frame = 0;
    // Each shown toast's box and whether it has already faded out, every frame while any is up:
    // only the box it last had is left to fade from once the library removes it.
    const measure = () => {
        const items = [...document.querySelectorAll('.toast-container:not(.toast-ghost) li')];
        items.forEach((li) => {
            const toast = li.querySelector('[role="alertdialog"]');
            if (!toast) return;
            const box = toast.getBoundingClientRect();
            const style = getComputedStyle(toast);
            const gone = style.animationName.startsWith('toast-life') && Number(style.opacity) <= 0.02;
            seen.set(li, { box, gone });
        });
        frame = items.length ? requestAnimationFrame(measure) : 0;
    };
    const watch = () => {
        if (!frame) frame = requestAnimationFrame(measure);
    };
    // Whether a toast still showing sat before the one that left: the list runs newest first.
    const newer = (record) => {
        for (let node = record.previousSibling; node; node = node.previousSibling) {
            if (node.nodeType === 1 && node.tagName === 'LI' && node.isConnected) return true;
        }
        return false;
    };
    const fade = (li, container, front) => {
        const last = seen.get(li);
        seen.delete(li);
        if (!last || last.gone || !container || !container.parentNode) return;
        const { box } = last;
        const holder = document.createElement('div');
        holder.className = 'toast-container toast-ghost';
        holder.setAttribute('aria-hidden', 'true');
        holder.style.cssText = `left: 0px; top: 0px; bottom: auto; max-width: none; pointer-events: none; z-index: ${front ? '10000' : '9998'};`;
        const list = document.createElement('ol');
        const copy = li.cloneNode(true);
        list.appendChild(copy);
        holder.appendChild(list);
        const toast = copy.querySelector('[role="alertdialog"]');
        if (toast) {
            toast.style.animation = 'none';
            toast.style.width = `${box.width}px`;
        }
        container.after(holder);
        // A parent with a transform places and sizes a fixed box in its own frame, not the
        // window's: read that frame off the copy where it first lands, then put the copy back on
        // the box the toast left.
        const origin = holder.getBoundingClientRect();
        const drawn = toast ? toast.getBoundingClientRect().width : box.width;
        const scale = box.width > 0 && drawn > 0 ? drawn / box.width : 1;
        holder.style.left = `${(box.left - origin.left) / scale}px`;
        holder.style.top = `${(box.top - origin.top) / scale}px`;
        if (toast) toast.style.width = `${box.width / scale}px`;
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
                if (!seen.has(node)) return;
                const at = record.target.nodeType === 1 ? record.target : null;
                const container = at ? at.closest('.toast-container:not(.toast-ghost)') : null;
                fade(node, container, !newer(record));
            });
            if (record.addedNodes.length) watch();
        }
    }).observe(document.body, { childList: true, subtree: true });
    watch();
})();
