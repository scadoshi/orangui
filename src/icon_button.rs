//! A small icon button: one dock icon in the kit's icon-button box (`btn-action btn-icon`), named
//! for what it does by its `aria-label` and its tooltip. A settings row's pencil ("Change theme")
//! is one.

use dioxus::prelude::*;

use crate::icons::DockIcon;

/// An icon button: `path` (one of [`icons`](crate::icons)' constants) in the kit's icon-button
/// box, with `label` as its `aria-label` and its tooltip, since the icon carries no words. Use it
/// where a text button's word would be cut off, such as a settings row's `Change`.
#[component]
pub fn IconButton(
    /// What it does, read by a screen reader and shown on hover: "Change theme".
    #[props(into)]
    label: String,
    /// The icon's path.
    path: &'static str,
    /// Whether it is off.
    #[props(default)]
    disabled: bool,
    /// What a press does.
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "btn-action btn-icon",
            r#type: "button",
            "aria-label": "{label}",
            title: "{label}",
            disabled,
            onclick: move |evt| onclick.call(evt),
            DockIcon { path }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::PENCIL;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn an_icon_button_draws_its_icon_and_is_named_for_screen_readers_and_on_hover() {
        fn app() -> Element {
            rsx! { IconButton { label: "Change theme", path: PENCIL, onclick: |_| {} } }
        }
        let html = render(app);
        assert!(html.contains(r#"class="btn-action btn-icon""#), "{html}");
        assert!(html.contains(r#"type="button""#), "{html}");
        assert!(
            html.contains(r#"aria-label="Change theme""#)
                && html.contains(r#"title="Change theme""#),
            "{html}"
        );
        assert!(
            html.contains(&format!(r#"d="{PENCIL}""#)),
            "the pencil: {html}"
        );
    }

    #[test]
    fn a_disabled_icon_button_says_so() {
        fn app() -> Element {
            rsx! { IconButton { label: "Change exceptions", path: PENCIL, disabled: true, onclick: |_| {} } }
        }
        assert!(render(app).contains("disabled"), "{}", render(app));
    }
}
