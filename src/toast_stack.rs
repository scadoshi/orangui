//! The app's toast stack, behind the `toast` feature. Styled by
//! `assets/toast.css` ([`TOAST_CSS`](crate::TOAST_CSS)).
//!
//! Toasts collapse into a stack and expand on tap, the way grouped
//! notifications do, so three of them cost about one toast of screen. A toast
//! stamps as it lands, and one pushed under a key already showing
//! ([`LiveToasts`]) changes in place.
//!
//! A host that tells the stack which screen is up (`ToastStack`'s `screen`) has each toast leave,
//! by its usual exit, once the screen it landed on is left, unless its title carries
//! [`CARRY_MARK`](orangui_core::toast::CARRY_MARK) ([`carry_title`](orangui_core::toast::carry_title)).

use std::collections::HashMap;
use std::iter;
use std::rc::Rc;
use std::time::Duration;

use dioxus::dioxus_core::DynamicNode;
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::point_interaction::{
    InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
};
use dioxus::html::{HasMouseData, Modifiers};
use dioxus::prelude::*;
use dioxus_primitives::toast::{
    Toast, ToastCloseButton, ToastContent, ToastDescription, ToastOptions, ToastProps,
    ToastPropsWithOwner, ToastProvider, ToastTitle, ToastType, Toasts,
};
use orangui_core::live_toast::{LiveText, live_key, live_title};
use orangui_core::toast::carried;

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
    /// The screen the app is on, as any text that changes when the user leaves it, such as the
    /// route's path. Each toast notes it as it lands and leaves once it changes, unless its title
    /// carries the carry mark. Without it every toast lives out its own life.
    #[props(default)]
    screen: Option<ReadSignal<String>>,
    children: Element,
) -> Element {
    // A leaving toast drops below the bottom edge.
    use_hook(|| {
        document::eval(crate::TOAST_LEAVE_JS);
    });
    use_context_provider(|| LiveToasts(Signal::new(HashMap::new())));
    use_context_provider(|| ToastScreen(screen));
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

/// The class of the wrapper around a toast's ×.
const CLOSE_CLASS: &str = "toast-close";

/// A toast's ×: its click closes the toast and goes no further, so it never reaches the toast's
/// tap and expands or collapses the stack. The wrapper has no box of its own (toast.css).
#[component]
fn CloseButton() -> Element {
    rsx! {
        span {
            class: CLOSE_CLASS,
            onclick: move |event: MouseEvent| event.stop_propagation(),
            ToastCloseButton {}
        }
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
    let mut props = props;
    let carry = carried(&props.title).map(str::to_owned);
    let carries = carry.is_some();
    if let Some(title) = carry {
        props.title = title;
    }
    use_leave_with_screen(props.on_close, carries);
    if let Some(name) = live_key(&props.title).map(str::to_owned) {
        return rsx! { LiveToast { name, toast: props } };
    }
    if props.children.is_none() {
        // The library's own layout, with the × in the wrapper that keeps its click from the tap.
        props.children = Some(rsx! {
            ToastContent {
                ToastTitle {}
                ToastDescription {}
            }
            CloseButton {}
        });
    }
    rsx! {
        div { class: "toast-tap", onclick: move |_| tap.toggle(),
            {Toast(props)}
            ToastIcons {}
        }
    }
}

/// The screen [`ToastStack`] was told about, if any, for each toast to leave with.
#[derive(Clone, Copy)]
struct ToastScreen(Option<ReadSignal<String>>);

/// Closes the toast through `on_close`, by the same exit as its ×, once the screen it landed on is
/// left, unless it `carries` over. A toast notes the screen as it mounts, on the render after it is
/// shown, so one shown in the same update as a screen change lands on the new screen.
fn use_leave_with_screen(on_close: Callback<MouseEvent>, carries: bool) {
    let ToastScreen(screen) = use_context::<ToastScreen>();
    let landed_on = use_hook(|| screen.map(|screen| screen.peek().clone()));
    let mut left = use_signal(|| false);
    use_effect(move || {
        let (Some(screen), Some(landed_on)) = (screen, landed_on.as_ref()) else {
            return;
        };
        if carries || *screen.read() == *landed_on || *left.peek() {
            return;
        }
        left.set(true);
        on_close.call(Event::new(Rc::new(MouseData::new(NoClick)), false));
    });
}

/// The click a toast's `on_close` takes when no one clicked: closing it because its screen was
/// left. Nothing reads it; the library's `on_close` only removes the toast.
struct NoClick;

impl InteractionLocation for NoClick {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::origin()
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::origin()
    }

    fn page_coordinates(&self) -> PagePoint {
        PagePoint::origin()
    }
}

impl InteractionElementOffset for NoClick {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::origin()
    }
}

impl ModifiersInteraction for NoClick {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}

impl PointerInteraction for NoClick {
    fn trigger_button(&self) -> Option<MouseButton> {
        None
    }

    fn held_buttons(&self) -> MouseButtonSet {
        MouseButtonSet::empty()
    }
}

impl HasMouseData for NoClick {
    fn as_any(&self) -> &dyn std::any::Any {
        self
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
        CloseButton {}
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

    mod close_button {
        use std::time::Duration;

        use dioxus::dioxus_core::NoOpMutations;
        use dioxus_primitives::toast::use_toast;

        use super::super::*;

        /// Pushes one plain toast and one keyed toast, neither timing out.
        #[component]
        fn Pusher() -> Element {
            let toasts = use_toast();
            let live = use_live_toasts();
            use_hook(move || {
                toasts.info("Saved".to_owned(), ToastOptions::default().permanent(true));
                live.show(
                    toasts,
                    "count",
                    "Pull-ups +1".to_owned(),
                    ToastType::Info,
                    ToastOptions::default().duration(Duration::from_secs(600)),
                );
            });
            rsx! {}
        }

        fn app() -> Element {
            rsx! {
                ToastStack { Pusher {} }
            }
        }

        fn render() -> String {
            // The library times each toast on tokio.
            let tokio = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();
            let _inside = tokio.enter();
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            for _ in 0..8 {
                dom.process_events();
                dom.render_immediate(&mut NoOpMutations);
            }
            dioxus_ssr::render(&dom)
        }

        #[test]
        fn every_toast_s_close_sits_in_the_wrapper_that_keeps_its_click_from_the_tap() {
            let html = render();
            let closes = html.matches("aria-label=\"close\"").count();
            assert_eq!(closes, 2, "a plain toast and a keyed one: {html}");
            let wrapped = html
                .matches(&format!("<span class=\"{CLOSE_CLASS}\"><button"))
                .count();
            assert_eq!(wrapped, closes, "each × in the wrapper: {html}");
            // Split, so only the wrapper's own line holds it whole.
            let stops = [
                "onclick: move |event: MouseEvent| event.",
                "stop_propagation(),",
            ]
            .concat();
            assert!(
                include_str!("toast_stack.rs").contains(&stops),
                "the wrapper stops the ×'s click"
            );
        }
    }

    mod screen_life {
        use std::cell::Cell;
        use std::time::Duration;

        use dioxus::dioxus_core::NoOpMutations;
        use dioxus_primitives::toast::use_toast;
        use orangui_core::toast::carry_title;

        use super::super::*;

        thread_local! {
            static PROBE: Cell<Option<(Toasts, Signal<String>)>> = const { Cell::new(None) };
            static WATCHES: Cell<bool> = const { Cell::new(true) };
        }

        /// The toaster and the screen the stack is told about.
        fn probe() -> (Toasts, Signal<String>) {
            PROBE.with(Cell::get).unwrap()
        }

        /// Hands the probe the toaster from under the stack.
        #[component]
        fn Grab(screen: Signal<String>) -> Element {
            let toasts = use_toast();
            PROBE.with(|p| p.set(Some((toasts, screen))));
            rsx! {}
        }

        fn app() -> Element {
            let screen = use_signal(|| "/decks".to_owned());
            let watched = WATCHES.with(Cell::get).then(|| ReadSignal::from(screen));
            rsx! {
                ToastStack { screen: watched, Grab { screen } }
            }
        }

        fn settle(dom: &mut VirtualDom) {
            for _ in 0..8 {
                dom.process_events();
                dom.render_immediate(&mut NoOpMutations);
            }
        }

        /// The titles of the toasts on the page.
        fn shown(dom: &VirtualDom) -> String {
            dioxus_ssr::render(dom)
        }

        fn long() -> ToastOptions {
            ToastOptions::default().duration(Duration::from_secs(600))
        }

        /// Mounts the stack on tokio, which times each toast, and runs `steps` against it.
        fn run(watches: bool, steps: impl FnOnce(&mut VirtualDom)) {
            let tokio = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();
            let _inside = tokio.enter();
            WATCHES.with(|w| w.set(watches));
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            settle(&mut dom);
            steps(&mut dom);
        }

        fn toast(dom: &mut VirtualDom, title: String) {
            let (toasts, _) = probe();
            dom.in_runtime(|| toasts.info(title, long()));
            settle(dom);
        }

        fn go(dom: &mut VirtualDom, to: &str) {
            let (_, mut screen) = probe();
            dom.in_runtime(|| screen.set(to.to_owned()));
            settle(dom);
        }

        #[test]
        fn a_toast_leaves_when_the_screen_changes() {
            run(true, |dom| {
                toast(dom, "Filter applied".to_owned());
                assert!(shown(dom).contains("Filter applied"), "{}", shown(dom));
                go(dom, "/decks/1");
                assert!(!shown(dom).contains("Filter applied"), "{}", shown(dom));
            });
        }

        #[test]
        fn a_carried_toast_stays_on_the_next_screen_without_its_mark() {
            run(true, |dom| {
                toast(dom, carry_title("Copied to clipboard"));
                go(dom, "/decks/1");
                let html = shown(dom);
                assert!(html.contains("Copied to clipboard"), "{html}");
                assert!(!html.contains("carry:"), "the mark is never drawn: {html}");
            });
        }

        #[test]
        fn a_toast_shown_as_the_screen_changes_lives_on_the_new_screen() {
            run(true, |dom| {
                let (toasts, mut screen) = probe();
                dom.in_runtime(|| {
                    toasts.info("Deck opened".to_owned(), long());
                    screen.set("/decks/2".to_owned());
                });
                settle(dom);
                assert!(shown(dom).contains("Deck opened"), "{}", shown(dom));
            });
        }

        #[test]
        fn a_stack_told_no_screen_keeps_every_toast() {
            run(false, |dom| {
                toast(dom, "Filter applied".to_owned());
                go(dom, "/decks/1");
                assert!(shown(dom).contains("Filter applied"), "{}", shown(dom));
            });
        }
    }
}
