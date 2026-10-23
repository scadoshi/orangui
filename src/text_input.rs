//! A labeled form field: the label with an optional "?", the input, and an
//! optional error line under it. Styled by `assets/app.css`.

use dioxus::prelude::*;

use crate::{InfoButton, PasswordInput};

/// A form label, with a "?" hanging off its right edge when `on_info` is
/// given. The label alone stays centered, so it lines up with the labels
/// above and below that have no "?". `for_id` ties it to its input.
///
/// [`TextInput`] draws one; a field the kit has no input for, such as a row
/// of chips, puts one above it.
#[component]
pub fn FieldLabel(
    label: String,
    #[props(default)] for_id: Option<String>,
    #[props(default)] on_info: Option<EventHandler<MouseEvent>>,
) -> Element {
    match on_info {
        Some(onclick) => rsx! {
            div { class: "label-hint",
                span { class: "label-hint-anchor",
                    label { class: "label", r#for: for_id, "{label}" }
                    InfoButton { onclick }
                }
            }
        },
        None => rsx! {
            label { class: "label", r#for: for_id, "{label}" }
        },
    }
}

/// A labeled text field bound to `value`.
///
/// - `label` draws a [`FieldLabel`] above the input, with a "?" when
///   `on_info` is given. What the "?" opens stays the host's.
/// - `input_type` is the input's `type`, `text` by default. `password` draws
///   a [`PasswordInput`] with its Show/Hide toggle instead.
/// - `error` marks the input with `input-error` and puts the message under
///   it in a `message-error` line.
/// - `filter` maps each typed value before it is stored, such as keeping
///   only the digits of an amount.
///
/// Autocapitalize, autocorrect and spellcheck are off: a field here holds a
/// name, an email or an amount, which the keyboard's fixes only get wrong.
#[component]
pub fn TextInput(
    value: Signal<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] label: Option<String>,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] input_type: Option<String>,
    #[props(default)] error: Option<String>,
    #[props(default)] on_info: Option<EventHandler<MouseEvent>>,
    /// The keyboard to bring up, such as `numeric`, for a field typed as
    /// text.
    #[props(default)]
    inputmode: Option<String>,
    #[props(default)] maxlength: Option<usize>,
    #[props(default)] filter: Option<Callback<String, String>>,
    #[props(default = false)] disabled: bool,
) -> Element {
    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let class = input_class(error.is_some());

    rsx! {
        if let Some(label) = label {
            FieldLabel { label, for_id: id.clone(), on_info }
        }
        if input_type == "password" {
            PasswordInput {
                value,
                id,
                placeholder,
                disabled,
                class: class.to_string(),
            }
        } else {
            input {
                class,
                id,
                r#type: "{input_type}",
                placeholder,
                inputmode,
                maxlength: maxlength.map(|n| n.to_string()),
                value: "{value}",
                disabled,
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    let typed = event.value();
                    value.set(match filter {
                        Some(filter) => filter.call(typed),
                        None => typed,
                    });
                },
            }
        }
        if let Some(error) = error {
            div { class: "message-error", "{error}" }
        }
    }
}

/// The input's classes: `input`, plus `input-error` while it shows an error.
fn input_class(is_error: bool) -> &'static str {
    if is_error {
        "input input-error"
    } else {
        "input"
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
    fn input_class_marks_an_error() {
        assert_eq!(input_class(false), "input");
        assert_eq!(input_class(true), "input input-error");
    }

    #[test]
    fn a_plain_field_has_its_label_and_no_hint() {
        fn app() -> Element {
            let value = use_signal(|| "ada".to_string());
            rsx! {
                TextInput { value, id: "name", label: "Name", maxlength: 40usize }
            }
        }
        let html = render(app);
        assert!(html.contains(r#"<label class="label" for="name">Name</label>"#));
        assert!(!html.contains("label-hint"));
        assert!(!html.contains("info-button"));
        assert!(html.contains(r#"class="input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"maxlength="40""#));
        assert!(html.contains(r#"value="ada""#));
        assert!(html.contains(r#"spellcheck="false""#));
        assert!(!html.contains("message-error"));
    }

    #[test]
    fn on_info_hangs_a_hint_off_the_label() {
        fn app() -> Element {
            let value = use_signal(String::new);
            rsx! {
                TextInput { value, label: "Goal", on_info: move |_| {} }
            }
        }
        let html = render(app);
        assert!(html.contains(r#"<div class="label-hint"><span class="label-hint-anchor"><label class="label">Goal</label><button class="info-button""#));
    }

    #[test]
    fn an_error_marks_the_input_and_shows_under_it() {
        fn app() -> Element {
            let value = use_signal(String::new);
            rsx! {
                TextInput { value, error: "Required" }
            }
        }
        let html = render(app);
        assert!(html.contains(r#"class="input input-error""#));
        assert!(html.contains(r#"<div class="message-error">Required</div>"#));
    }

    #[test]
    fn a_password_field_takes_the_toggle() {
        fn app() -> Element {
            let value = use_signal(String::new);
            rsx! {
                TextInput { value, input_type: "password", error: "Too short" }
            }
        }
        let html = render(app);
        assert!(html.contains("password-input-wrapper"));
        assert!(html.contains(r#"class="input input-error input-password""#));
        assert!(html.contains(r#"type="password""#));
        assert!(html.contains("password-toggle-btn"));
    }
}
