//! A clear button beside an input's box, at its right end, for any field
//! that can be emptied: a text input, a tappable picker box, a box of
//! chips. Styled by `assets/components.css`.

use dioxus::prelude::*;

/// Wraps one input-like child and draws a `×` beside its right end while
/// `filled`. The child gives up the button's width as it slides in, and
/// takes it back when the field empties. The tap clears
/// through `on_clear` and never reaches the child, so a tappable box does
/// not open on it. `label` is the button's accessible name, `Clear` by
/// default.
#[component]
pub fn ClearWrap(
    filled: bool,
    on_clear: EventHandler<()>,
    #[props(default)] label: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: if filled { "input-clear-wrap has-text" } else { "input-clear-wrap" },
            {children}
            button {
                r#type: "button",
                class: "input-clear-btn",
                tabindex: if filled { "0" } else { "-1" },
                aria_label: label.unwrap_or_else(|| "Clear".to_string()),
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_clear.call(());
                },
                "\u{00d7}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_filled_field_shows_its_clear() {
        fn app() -> Element {
            rsx! {
                ClearWrap { filled: true, on_clear: |()| {}, input { class: "input", value: "a" } }
            }
        }
        let html = render(app);
        assert!(html.contains("input-clear-wrap has-text"), "{html}");
        assert!(html.contains("tabindex=\"0\""), "{html}");
    }

    #[test]
    fn an_empty_field_keeps_the_button_out_of_the_tab_order() {
        fn app() -> Element {
            rsx! {
                ClearWrap { filled: false, on_clear: |()| {}, input { class: "input" } }
            }
        }
        let html = render(app);
        assert!(!html.contains("has-text"), "{html}");
        assert!(html.contains("tabindex=\"-1\""), "{html}");
    }
}
