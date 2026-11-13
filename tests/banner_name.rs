//! A banner's dismiss button carries a name a screen reader says, not only its glyph.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use orangui::Banner;

fn app() -> Element {
    rsx! {
        Banner { "A new version is out" }
    }
}

#[test]
fn a_banner_s_dismiss_button_is_named() {
    let mut dom = VirtualDom::new(app);
    dom.rebuild(&mut NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains(r#"class="banner-dismiss" aria-label="Dismiss""#),
        "{html}"
    );
    assert!(
        html.contains(r#"<span aria-hidden="true">✕</span>"#),
        "the glyph stays out of the name: {html}"
    );
}
