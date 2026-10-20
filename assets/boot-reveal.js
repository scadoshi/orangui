// Shows #main, which BOOT_HEAD hides, once every stylesheet in the head has
// loaded. The inline ones are in already: Dioxus adds head elements in order,
// and BootReveal comes last. A linked sheet still loading holds the reveal
// until it fires load or error.
//
// The timeout shows the app anyway if a sheet never reports, so a stuck load
// leaves it unstyled rather than blank.
(() => {
    const main = document.getElementById("main");
    if (!main) return;

    const show = () => {
        main.style.opacity = "1";
    };
    const pending = [...document.querySelectorAll('link[rel="stylesheet"]')].filter(
        (link) => !link.sheet,
    );
    let left = pending.length;
    if (left === 0) {
        show();
        return;
    }
    const done = () => {
        left -= 1;
        if (left === 0) show();
    };
    for (const link of pending) {
        link.addEventListener("load", done, { once: true });
        link.addEventListener("error", done, { once: true });
    }
    setTimeout(show, 2000);
})();
