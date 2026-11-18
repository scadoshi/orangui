// A toast stamps as it lands: once it has risen into its spot it jumps a little and grows, then
// settles, as a stack badge pops. Transform only, added on top of the toast's own animation, and
// small enough that a phone-wide toast stays inside its corner gap; nothing moves under reduced
// motion. Only a toast new to the list stamps: a keyed toast's update changes it in place and
// crossfades its text instead.
(() => {
    if (window.__oranguiToastStamp) return;
    window.__oranguiToastStamp = true;
    const STAMP_MS = 200;
    // How long a timed toast takes to rise into its spot (toast-life-* in toast.css).
    const TIMED_RISE_MS = 200;
    const still = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
    const root = () => getComputedStyle(document.documentElement);
    const ease = () => root().getPropertyValue('--ease-enter').trim() || 'ease-out';
    const rise = (li) => {
        if (li.closest('.timed-fade')) return TIMED_RISE_MS;
        const ms = parseFloat(root().getPropertyValue('--enter-duration'));
        return Number.isFinite(ms) ? ms : 280;
    };
    const stamp = (li) => {
        if (still() || li.querySelector('[data-live-update]')) return;
        const toast = li.querySelector('[role="alertdialog"]');
        if (!toast) return;
        // The delay runs on the page's animation clock, as the rise does, so the stamp starts as
        // the toast lands.
        toast.animate(
            [{ transform: 'none' }, { transform: 'translateY(-2px) scale(1.04)', offset: 0.4 }, { transform: 'none' }],
            { duration: STAMP_MS, delay: rise(li), easing: ease(), composite: 'add' },
        );
    };
    new MutationObserver((records) => {
        for (const record of records) {
            record.addedNodes.forEach((node) => {
                if (node.nodeType !== 1) return;
                const items = node.matches('li') ? [node] : [...node.querySelectorAll('li')];
                items.forEach((li) => {
                    if (li.closest('.toast-container') && !li.closest('.toast-ghost')) stamp(li);
                });
            });
        }
    }).observe(document.body, { childList: true, subtree: true });
})();
