//! One theme sheet on every host: the sites' `ThemePicker` opens the same sheet body the apps'
//! `ThemeSheet` draws (its search, red clear, group chips, rows and empty state), styled by one set
//! of rules.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use orangui::{
    SheetFrame, SheetWide, ThemeConfig, ThemePicker, ThemeSheet, use_overlay_back_stack,
};

const COMPONENTS: &str = include_str!("../assets/components.css");

/// The declarations of the first rule whose selector is exactly `selector`, or `""`.
fn rule(selector: &str) -> &'static str {
    let head = format!("\n{selector} {{");
    COMPONENTS
        .split_once(head.as_str())
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

/// Mounts `app`, lets its effects run, and renders it, inside a runtime for the timers a sheet
/// starts.
fn render(app: fn() -> Element) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _entered = runtime.enter();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// The sheet's body: from its search bar to the end of its last theme row.
fn body(html: &str) -> &str {
    let start = html.find(r#"<div class="theme-search-bar""#).unwrap();
    let end = html.rfind("</button>").unwrap();
    html.get(start..end).unwrap()
}

fn darcula() -> ThemeConfig {
    ThemeConfig {
        name: "darcula".to_string(),
        is_dark: true,
    }
}

#[test]
fn the_site_picker_opens_the_app_sheet_body() {
    fn site() -> Element {
        let stack = use_overlay_back_stack();
        use_context_provider(|| stack);
        let theme = use_signal(darcula);
        let open = use_signal(|| true);
        rsx! { ThemePicker { theme, open } }
    }
    fn app() -> Element {
        let stack = use_overlay_back_stack();
        use_context_provider(|| stack);
        let theme = use_signal(darcula);
        let open = use_signal(|| true);
        rsx! {
            SheetFrame { wide: SheetWide::Drawer,
                ThemeSheet { open, theme, on_save: |_| {} }
            }
        }
    }
    let site = render(site);
    let app = render(app);
    assert!(body(&app).contains(r#"class="pref-row selected""#), "{app}");
    assert_eq!(body(&site), body(&app), "the site picker's sheet body");
}

#[test]
fn the_kit_clear_is_red() {
    let clear = rule(".input-clear-btn");
    assert!(
        clear.contains("color: var(--ink-color-error, var(--color-error));")
            && clear.contains("border: 1px solid var(--color-error);"),
        "{clear}"
    );
}

#[test]
fn one_dot_size_and_no_site_only_rows() {
    let dot = rule(".theme-dot");
    assert!(
        dot.contains("width: 0.7rem;") && dot.contains("height: 0.7rem;"),
        "the dots live in the shared sheet: {dot}"
    );
    assert!(!rule(".theme-swatches").is_empty(), "the swatch strip too");
    for gone in [
        ".theme-picker-sheet .theme-dot",
        ".theme-picker-sheet .theme-swatches",
        ".theme-picker-sheet .nav-dropdown-item",
        ".theme-picker-sheet .nav-dropdown-label",
    ] {
        assert!(
            !COMPONENTS.contains(gone),
            "{gone} still styles a second sheet"
        );
    }
}

#[test]
fn a_host_can_leave_the_search_out() {
    fn app() -> Element {
        let stack = use_overlay_back_stack();
        use_context_provider(|| stack);
        let theme = use_signal(darcula);
        let open = use_signal(|| true);
        rsx! { ThemeSheet { open, theme, search: false, on_save: |_| {} } }
    }
    let html = render(app);
    assert!(!html.contains("theme-search"), "no search bar: {html}");
    assert!(
        html.contains(r#"class="pref-row selected""#),
        "the rows stay: {html}"
    );
}
