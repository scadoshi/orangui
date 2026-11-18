//! The app's toast stack, behind the `toast` feature. Styled by
//! `assets/toast.css` ([`TOAST_CSS`](crate::TOAST_CSS)).
//!
//! Toasts collapse into a stack and expand on tap, the way grouped
//! notifications do, so three of them cost about one toast of screen. A toast
//! stamps as it lands, and one pushed under a key already showing
//! ([`LiveToasts`]) changes in place.

use std::collections::HashMap;
use std::iter;
use std::time::Duration;

use dioxus::dioxus_core::DynamicNode;
use dioxus::prelude::*;
use dioxus_primitives::toast::{
    Toast, ToastCloseButton, ToastContent, ToastOptions, ToastProps, ToastPropsWithOwner,
    ToastProvider, ToastTitle, ToastType, Toasts,
};
use orangui_core::live_toast::{LiveText, live_key, live_title};

use crate::{TOAST_NORMAL, entrance};

/// The toast provider with the collapsing stack. Wrap the router in it; every
/// `use_toast` below reaches it.
#[component]
pub fn ToastStack(
    /// How many toasts show at once.
    #[props(default = 3)]
    max_toasts: usize,
    /// How long a toast stays when its call passes no duration.
    #[props(default = TOAST_NORMAL)]
    default_duration: Duration,
    /// Fade each toast out in CSS on a life read off its type: 3s for info,
    /// 5s for the rest. For a host whose info toasts all use `TOAST_QUICK`
    /// and the rest `TOAST_NORMAL`; without it a toast drops in and the
    /// library removes it.
    #[props(default)]
    timed_fade: bool,
    children: Element,
) -> Element {
    // A leaving toast drops below the bottom edge.
    use_hook(|| {
        document::eval(crate::TOAST_LEAVE_JS);
        document::eval(crate::TOAST_STAMP_JS);
    });
    use_context_provider(|| LiveToasts(Signal::new(HashMap::new())));
    let expanded = use_signal(|| false);
    // Set only for the length of a tap-driven toggle. The collapse offset is
    // a margin, and an arriving toast changes that same margin by taking over
    // as `:first-child`, so a permanent transition animated the stack
    // expanding and dropping back every time one landed. CSS cannot tell the
    // two apart; this can, because only the tap sets it.
    let animating = use_signal(|| false);
    let shown = use_signal(|| 0);
    use_context_provider(|| StackTap {
        expanded,
        animating,
        shown,
    });
    let mut class = String::from("toast-container");
    if timed_fade {
        class.push_str(" timed-fade");
    }
    if expanded() {
        class.push_str(" expanded");
    }
    if animating() {
        class.push_str(" animating");
    }

    rsx! {
        ToastProvider {
            max_toasts,
            default_duration: Some(default_duration),
            class,
            // The library owns the container and list DOM and does not pass
            // event handlers through `attributes`, so the tap target has to
            // be the toast itself. `display: contents` keeps this wrapper out
            // of the layout.
            render_toast: move |props: ToastPropsWithOwner| {
                let toast = DynamicNode::Component(props.into_vcomponent(StackToast));
                rsx! { {toast} }
            },
            {children}
        }
    }
}

/// The stack's expanded state and whether a tap is animating it, for each toast's tap.
#[derive(Clone, Copy)]
struct StackTap {
    expanded: Signal<bool>,
    animating: Signal<bool>,
    /// How many toasts are on the page.
    shown: Signal<usize>,
}

impl StackTap {
    /// Expands or collapses the stack on a tap, animating the change.
    fn toggle(self) {
        let Self {
            mut expanded,
            mut animating,
            ..
        } = self;
        expanded.toggle();
        animating.set(true);
        spawn(async move {
            // Outlasts the 0.2s transition in toast.css.
            entrance::sleep_ms(250).await;
            animating.set(false);
        });
    }
}

/// The fewest toasts that make a stack to expand.
const STACK_MIN: usize = 2;

/// Counts a toast in the stack while it is mounted, and collapses the stack once fewer than
/// [`STACK_MIN`] are left, so the next stack arrives collapsed. A tap on a toast's × also reaches
/// its tap and expands the stack as the toast closes.
fn use_stack_slot(tap: StackTap) {
    let StackTap {
        mut expanded,
        mut shown,
        ..
    } = tap;
    use_hook(move || {
        let up = shown.peek().saturating_add(1);
        shown.set(up);
    });
    use_drop(move || {
        let left = shown.peek().saturating_sub(1);
        shown.set(left);
        if left < STACK_MIN && *expanded.peek() {
            expanded.set(false);
        }
    });
}

/// One toast in the stack, tapped to expand or collapse it: a keyed one ([`LiveToast`]) or a
/// plain one as the library draws it.
#[expect(non_snake_case, reason = "a component, named as one")]
fn StackToast(props: ToastProps) -> Element {
    let tap = use_context::<StackTap>();
    use_stack_slot(tap);
    if let Some(name) = live_key(&props.title).map(str::to_owned) {
        return rsx! { LiveToast { name, toast: props } };
    }
    rsx! {
        div { class: "toast-tap", onclick: move |_| tap.toggle(),
            {Toast(props)}
            ToastIcons {}
        }
    }
}

/// The keyed toasts showing, each its live text by key. [`ToastStack`] provides it; reach it with
/// [`use_live_toasts`].
///
/// A toast pushed with [`LiveToasts::show`] under a key that is already showing changes that toast
/// in place: its text crossfades where it changed, it doesn't rise or stamp again, and its timer
/// starts over. Under a new key it lands as any toast does.
#[derive(Clone, Copy, Debug)]
pub struct LiveToasts(Signal<HashMap<String, LiveText>>);

impl LiveToasts {
    /// Shows `text` under `key`: updates the toast showing under it, or pushes a new one through
    /// `toasts` as `toast_type` with `options`.
    pub fn show(
        mut self,
        toasts: Toasts,
        key: &str,
        text: String,
        toast_type: ToastType,
        options: ToastOptions,
    ) {
        {
            let mut live = self.0.write();
            if let Some(showing) = live.get_mut(key) {
                showing.update(text);
                return;
            }
            live.insert(key.to_owned(), LiveText::new(text));
        }
        toasts.show(live_title(key), toast_type, options);
    }

    /// Whether a toast under `key` is showing.
    #[must_use]
    pub fn is_showing(&self, key: &str) -> bool {
        self.0.peek().contains_key(key)
    }
}

/// The keyed toasts below [`ToastStack`].
#[must_use]
pub fn use_live_toasts() -> LiveToasts {
    use_context::<LiveToasts>()
}

/// A keyed toast: its live text in the library's toast. Its key leaves [`LiveToasts`] as the
/// toast leaves the list.
#[component]
fn LiveToast(name: String, toast: ToastProps) -> Element {
    let LiveToasts(mut live) = use_live_toasts();
    let gone = name.clone();
    use_drop(move || {
        live.write().remove(&gone);
    });
    let Some(text) = live.read().get(&name).cloned() else {
        return rsx! {};
    };
    let updates = text.updates();
    rsx! {
        for update in iter::once(updates) {
            LiveToastBody { key: "{update}", text: text.clone(), toast: toast.clone() }
        }
    }
}

/// A keyed toast's body, mounted afresh on each update so the library's timer starts over; an
/// update holds its place rather than rising again (toast.css).
#[component]
fn LiveToastBody(text: LiveText, toast: ToastProps) -> Element {
    let tap = use_context::<StackTap>();
    let updated = text.updates() > 0;
    let mut toast = toast;
    toast.children = Some(rsx! {
        ToastContent {
            ToastTitle {
                LiveTitle { text }
            }
        }
        ToastCloseButton {}
    });
    rsx! {
        div {
            class: "toast-tap",
            "data-live-update": updated.then_some("true"),
            onclick: move |_| tap.toggle(),
            {Toast(toast)}
            ToastIcons {}
        }
    }
}

/// Draws any icon token in the shown toasts' titles as its key, once its toast is on the page: an
/// effect, since a toast renders through a portal, where an element's `onmounted` never fires.
#[component]
fn ToastIcons() -> Element {
    use_effect(|| {
        document::eval(&crate::icon_text::toast_icons_js());
    });
    rsx! {}
}

/// A keyed toast's title: its text whole as it lands; after an update the part the old and new
/// text share, then the new rest with the old rest laid over it, fading out (toast.css).
#[component]
fn LiveTitle(text: LiveText) -> Element {
    let change = text.change();
    let updated = text.updates() > 0;
    rsx! {
        span { class: "toast-live",
            "{change.kept}"
            if updated {
                span { class: "toast-live-change",
                    span { class: "toast-live-new", "{change.new}" }
                    span { class: "toast-live-old", "aria-hidden": "true", "{change.old}" }
                }
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
    fn a_keyed_toast_lands_whole_then_crossfades_what_changed() {
        fn landed() -> Element {
            rsx! { LiveTitle { text: LiveText::new("Pull-ups +1".to_owned()) } }
        }
        fn updated() -> Element {
            let mut text = LiveText::new("Pull-ups +1".to_owned());
            text.update("Pull-ups +2".to_owned());
            rsx! { LiveTitle { text } }
        }
        assert_eq!(
            render(landed),
            "<span class=\"toast-live\">Pull-ups +1</span>"
        );
        assert_eq!(
            render(updated),
            "<span class=\"toast-live\">Pull-ups +<span class=\"toast-live-change\"><span class=\"toast-live-new\">2</span><span class=\"toast-live-old\" aria-hidden=\"true\">1</span></span></span>"
        );
    }

    mod stack_slots {
        use std::cell::Cell;

        use dioxus::dioxus_core::NoOpMutations;

        use super::super::*;

        thread_local! {
            static PROBE: Cell<Option<(StackTap, Signal<usize>)>> = const { Cell::new(None) };
        }

        /// The stack's state and how many toasts the app shows.
        fn probe() -> (StackTap, Signal<usize>) {
            PROBE.with(Cell::get).unwrap()
        }

        /// A toast as the stack counts it.
        #[component]
        fn Slot() -> Element {
            use_stack_slot(use_context::<StackTap>());
            rsx! {}
        }

        fn app() -> Element {
            let tap = StackTap {
                expanded: use_signal(|| false),
                animating: use_signal(|| false),
                shown: use_signal(|| 0),
            };
            use_context_provider(|| tap);
            let toasts = use_signal(|| 0_usize);
            PROBE.with(|p| p.set(Some((tap, toasts))));
            rsx! {
                for i in 0..toasts() {
                    Slot { key: "{i}" }
                }
            }
        }

        fn settle(dom: &mut VirtualDom) {
            for _ in 0..8 {
                dom.process_events();
                dom.render_immediate(&mut NoOpMutations);
            }
        }

        /// Shows `n` toasts.
        fn show(dom: &mut VirtualDom, n: usize) {
            let (_, mut toasts) = probe();
            dom.in_runtime(|| toasts.set(n));
            settle(dom);
        }

        fn expand(dom: &VirtualDom) {
            let (tap, _) = probe();
            let mut expanded = tap.expanded;
            dom.in_runtime(|| expanded.set(true));
        }

        fn expanded(dom: &VirtualDom) -> bool {
            let (tap, _) = probe();
            dom.in_runtime(|| *tap.expanded.peek())
        }

        fn mount() -> VirtualDom {
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            settle(&mut dom);
            dom
        }

        #[test]
        fn closing_a_lone_toast_its_tap_expanded_leaves_the_next_stack_collapsed() {
            let mut dom = mount();
            show(&mut dom, 1);
            // The ×'s click bubbles to the toast's tap, which expands the stack as it closes.
            expand(&dom);
            show(&mut dom, 0);
            assert!(!expanded(&dom), "no toast up, nothing to expand");
            show(&mut dom, 2);
            assert!(!expanded(&dom), "the next stack arrives collapsed");
        }

        #[test]
        fn an_expanded_stack_collapses_once_one_toast_is_left_and_not_before() {
            let mut dom = mount();
            show(&mut dom, 3);
            expand(&dom);
            show(&mut dom, 2);
            assert!(expanded(&dom), "two toasts are still a stack");
            show(&mut dom, 1);
            assert!(!expanded(&dom), "a lone toast has nothing to expand");
        }
    }
}
