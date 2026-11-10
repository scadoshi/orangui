//! The HTML gate: the components a host renders today keep their exact markup.
//! Each case renders a component with props a host passes and compares the
//! result to a saved copy in `tests/html/`. Run with `BLESS_HTML=1` to rewrite
//! the copies after a deliberate markup change; the theme sheet's copies list
//! every theme, so a theme added or renamed rewrites them too.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::fs;
use std::path::PathBuf;

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use orangui::{
    ActionButton, AlertDialog, BannerStatus, BottomSheet, CollapseAll, CollapseBody, CollapseCaret,
    CollapseHeader, DialogAction, DialogHostView, Panel, ReleaseCard, ReleaseTag, ThemeConfig,
    ThemeSheet, use_dialog_host, use_overlay_back_stack, use_theme_follow,
};

/// Compares `html` with the saved copy `name`, or saves it under `BLESS_HTML`.
fn golden(name: &str, html: &str) {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "html",
        &format!("{name}.html"),
    ]
    .iter()
    .collect();
    if std::env::var_os("BLESS_HTML").is_some() {
        fs::write(&path, html).unwrap();
        return;
    }
    let saved =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("no saved copy at {}", path.display()));
    assert_eq!(html, saved, "{name} renders different markup");
}

/// Mounts `app`, lets its effects run, and renders it, inside a runtime for
/// the timers a component starts.
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

/// Two sections, `1.1.0` shut and `1.0.0` open, with the list's button.
fn collapse_app(shut: &'static [&'static str]) -> Element {
    let mut set = orangui::use_collapse_set();
    use_hook(move || set.collapse_all(shut.iter().copied()));
    let keys = vec!["1.1.0".to_owned(), "1.0.0".to_owned()];
    rsx! {
        CollapseAll { set, keys: keys.clone() }
        for key in keys {
            div { key: "{key}",
                CollapseCaret { set, section: key.clone(), label: key.clone() }
                CollapseBody { set, section: key.clone(), p { "{key}" } }
            }
        }
    }
}

/// Two sections whose whole header row toggles, `1.1.0` shut and `1.0.0`
/// open, each header holding its version and a tag after the caret.
fn collapse_header_app() -> Element {
    let mut set = orangui::use_collapse_set();
    use_hook(move || set.collapse_all(["1.1.0"]));
    let keys = vec!["1.1.0".to_owned(), "1.0.0".to_owned()];
    rsx! {
        for key in keys {
            div { key: "{key}",
                CollapseHeader { set, section: key.clone(), label: key.clone(), class: "version-row",
                    h2 { "{key}" }
                    span { class: "status-tag", "Latest" }
                }
                CollapseBody { set, section: key.clone(), p { "{key}" } }
            }
        }
    }
}

/// Two release cards, `1.1.0` shut with a linked version and an amber tag,
/// `1.0.0` open with a green tag and no date.
fn release_cards_app() -> Element {
    let mut set = orangui::use_collapse_set();
    use_hook(move || set.collapse_all(["1.1.0"]));
    rsx! {
        ReleaseCard {
            set,
            version: "1.1.0",
            date: "Nov 9, 2026".to_owned(),
            tag: ReleaseTag { status: BannerStatus::Doing, label: "Upcoming".to_owned() },
            entries: vec!["A new quote".to_owned()],
            id: "v1-1-0".to_owned(),
            href: "#v1-1-0".to_owned(),
        }
        ReleaseCard {
            set,
            version: "1.0.0",
            tag: ReleaseTag { status: BannerStatus::Done, label: "Latest".to_owned() },
            entries: vec!["First release".to_owned(), "Second note".to_owned()],
        }
    }
}

#[test]
fn release_cards_one_shut() {
    golden("release_cards_one_shut", &render(release_cards_app));
}

#[test]
fn collapse_header_one_shut() {
    golden("collapse_header_one_shut", &render(collapse_header_app));
}

#[test]
fn collapse_one_shut() {
    golden("collapse_one_shut", &render(|| collapse_app(&["1.1.0"])));
}

#[test]
fn collapse_all_shut() {
    golden(
        "collapse_all_shut",
        &render(|| collapse_app(&["1.1.0", "1.0.0"])),
    );
}

#[test]
fn panel_title_end() {
    golden(
        "panel_title_end",
        &dioxus_ssr::render_element(rsx! {
            Panel {
                eyebrow: "Cairn",
                title: "Changelog",
                title_h1: true,
                title_end: rsx! {
                    button { r#type: "button", "End" }
                },
                p { "Body" }
            }
        }),
    );
}

#[test]
fn panel_body_only() {
    golden(
        "panel_body_only",
        &dioxus_ssr::render_element(rsx! {
            Panel { p { "Body" } }
        }),
    );
}

#[test]
fn panel_title_only() {
    golden(
        "panel_title_only",
        &dioxus_ssr::render_element(rsx! {
            Panel { title: "Title", p { "Body" } }
        }),
    );
}

#[test]
fn panel_full_header_and_actions() {
    golden(
        "panel_full",
        &dioxus_ssr::render_element(rsx! {
            Panel {
                eyebrow: "One-Time",
                title: "Title",
                status: BannerStatus::Done,
                actions: rsx! {
                    a { class: "panel-action", href: "/go", "Go" }
                },
                p { "Body" }
            }
        }),
    );
}

#[test]
fn panel_hero_centered_with_trailing() {
    golden(
        "panel_hero",
        &dioxus_ssr::render_element(rsx! {
            Panel {
                title: "Hero",
                title_h1: true,
                centered: true,
                status: BannerStatus::Doing,
                status_label: "Live",
                title_trailing: rsx! {
                    span { class: "chip", "Rust" }
                },
                p { "Body" }
            }
        }),
    );
}

#[test]
fn panel_title_bar_and_class() {
    golden(
        "panel_title_bar",
        &dioxus_ssr::render_element(rsx! {
            Panel {
                class: "form-panel",
                title: "Ignored",
                title_bar: rsx! {
                    h2 { class: "panel-title", "Login" }
                    button { class: "btn-action", "?" }
                },
                actions: rsx! {
                    button { class: "btn-action", "Log in" }
                },
                p { "Body" }
            }
        }),
    );
}

fn dialog_app() -> Element {
    let stack = use_overlay_back_stack();
    use_context_provider(|| stack);
    let host = use_dialog_host();
    use_context_provider(|| host);
    rsx! {
        DialogOwner {}
        DialogHostView {}
    }
}

#[component]
fn DialogOwner() -> Element {
    let open = use_signal(|| true);
    let blocked = use_signal(|| true);
    rsx! {
        AlertDialog {
            open,
            title: "Delete deck?",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Link {
                    label: "Help".into(),
                    href: "https://example.com".into(),
                },
                DialogAction::Confirm {
                    label: "Delete".into(),
                    danger: true,
                    on_confirm: Callback::new(|()| {}),
                },
                ActionButton::new("Wait").disabled(blocked).on_click(|| {}).into(),
                DialogAction::Confirm {
                    label: "Keep".into(),
                    danger: false,
                    on_confirm: Callback::new(|()| {}),
                },
            ],
            p { "This cannot be undone." }
        }
    }
}

#[test]
fn dialog_open() {
    golden("dialog_open", &render(dialog_app));
}

fn theme_sheet_app(open: bool) -> Element {
    let stack = use_overlay_back_stack();
    use_context_provider(|| stack);
    let theme = use_signal(|| ThemeConfig {
        name: "darcula".to_string(),
        is_dark: true,
    });
    let follow = use_theme_follow(theme, ".app-shell");
    use_context_provider(|| follow);
    let open = use_signal(|| open);
    let hint = use_signal(|| false);
    rsx! {
        ThemeSheet {
            open,
            theme,
            on_save: |_| {},
            hint,
        }
    }
}

#[test]
fn theme_sheet_open() {
    golden("theme_sheet_open", &render(|| theme_sheet_app(true)));
}

#[test]
fn theme_sheet_closed() {
    golden("theme_sheet_closed", &render(|| theme_sheet_app(false)));
}

#[test]
fn a_closable_sheet_trades_the_close_bar_for_a_corner_button() {
    fn closable() -> Element {
        let open = use_signal(|| true);
        rsx! {
            BottomSheet { open, title: "Themes".to_string(), closable: true, "rows" }
        }
    }
    fn plain() -> Element {
        let open = use_signal(|| true);
        rsx! {
            BottomSheet { open, title: "Themes".to_string(), "rows" }
        }
    }
    let html = render(closable);
    assert!(html.contains("class=\"sheet-close\""), "{html}");
    assert!(!html.contains("action-bar"), "{html}");
    let html = render(plain);
    assert!(!html.contains("sheet-close"), "{html}");
    assert!(html.contains("action-bar"), "{html}");
}
