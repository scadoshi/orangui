//! A centered dialog for the apps: a title, a body and a row of actions over
//! a dim. Styled by `assets/app.css`.
//!
//! A `position: fixed` overlay inside a screen's scroll container gets
//! trapped by it in a mobile WebView, and the dim stops at the content
//! column. So dialogs draw at the app root: the host provides a
//! [`DialogHost`] from [`use_dialog_host`] and mounts [`DialogHostView`]
//! beside its router, and a dialog anywhere below describes itself into the
//! host with [`AlertDialog`] or [`use_hosted_dialog`].
//!
//! Closing fades the dim and the dialog out before they leave the DOM. Every
//! label is the host's: the kit draws the buttons it is given.
//!
//! The spec is built once, when the dialog opens. An action that has to
//! change while it shows (a countdown, a field that must not be empty, a
//! request in flight) is a [`DialogAction::Button`] built from an
//! [`ActionButton`], whose label and disabled state are signals the host
//! owns.

use std::cell::RefCell;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::use_overlay_back;

/// One button in a dialog's action row. `Dismiss`, `Confirm` and `Link`
/// close the dialog; a [`Button`](DialogAction::Button) closes it or not, as
/// its [`ActionButton`] says.
#[derive(Clone, PartialEq, Debug)]
pub enum DialogAction {
    /// Closes the dialog and does nothing else: Cancel, Got it.
    Dismiss(String),
    /// Closes the dialog, then runs `on_confirm`. `danger` draws it in the
    /// error color, for an action that cannot be undone.
    Confirm {
        /// The button's text.
        label: String,
        /// Marks the action as destructive.
        danger: bool,
        /// Runs after the dialog starts closing.
        on_confirm: Callback<()>,
    },
    /// A link styled as a button, opened in a new tab (an app's WebView
    /// hands a `_blank` https link to the system browser), and the dialog
    /// closes.
    Link {
        /// The link's text.
        label: String,
        /// Where it goes.
        href: String,
    },
    /// A button the host drives: disabled from a signal, relabeled from a
    /// signal, running async work under a busy label, or leaving the dialog
    /// open. See [`ActionButton`].
    Button(ActionButton),
}

impl From<ActionButton> for DialogAction {
    fn from(button: ActionButton) -> Self {
        DialogAction::Button(button)
    }
}

/// The future an [`ActionButton::busy`] handler returns: `true` closes the
/// dialog, `false` keeps it open.
type BusyFuture = Pin<Box<dyn Future<Output = bool>>>;

/// What clicking an [`ActionButton`] does.
#[derive(Clone)]
enum Run {
    /// Starts closing the dialog, then runs the handler.
    Close(Rc<RefCell<dyn FnMut()>>),
    /// Runs the handler; the dialog stays open.
    StayOpen(Rc<RefCell<dyn FnMut()>>),
    /// Shows the busy label and holds the dialog open while the future runs,
    /// then closes it if the future returns `true`.
    Busy {
        label: String,
        work: Rc<RefCell<dyn FnMut() -> BusyFuture>>,
    },
}

impl PartialEq for Run {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Run::Close(a), Run::Close(b)) | (Run::StayOpen(a), Run::StayOpen(b)) => {
                Rc::ptr_eq(a, b)
            }
            (Run::Busy { label: la, work: a }, Run::Busy { label: lb, work: b }) => {
                la == lb && Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}

/// A dialog action whose state can change while the dialog shows. Build one
/// with [`ActionButton::new`] and the methods below, and pass it as
/// `DialogAction::Button(..)` or `.into()`.
///
/// With no handler it closes the dialog, like [`DialogAction::Dismiss`].
/// The label and the disabled state are read from signals the host owns, so
/// they update in place; the dialog reads them when it draws, never at open.
///
/// ```
/// # use dioxus::prelude::*;
/// # use orangui::{ActionButton, DialogAction};
/// # fn _f(seconds: Signal<u8>, mut flipped: Signal<bool>) {
/// // Disabled through a countdown, relabeled each second.
/// let label = use_memo(move || match seconds() {
///     0 => "Delete".to_string(),
///     n => format!("Delete ({n})"),
/// });
/// let delete: DialogAction = ActionButton::new("Delete")
///     .danger()
///     .live_label(label)
///     .disabled(use_memo(move || seconds() > 0))
///     .on_click(|| { /* runs as the dialog closes */ })
///     .into();
/// // A toggle that leaves the dialog open.
/// let flip: DialogAction = ActionButton::new("Flip")
///     .stay_open(move || flipped.toggle())
///     .into();
/// # }
/// ```
#[derive(Clone, PartialEq)]
pub struct ActionButton {
    label: String,
    live_label: Option<ReadSignal<String>>,
    danger: bool,
    disabled: Option<ReadSignal<bool>>,
    run: Option<Run>,
}

impl fmt::Debug for ActionButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActionButton")
            .field("label", &self.label)
            .field("live_label", &self.live_label.is_some())
            .field("danger", &self.danger)
            .field("disabled", &self.disabled.is_some())
            .field("run", &self.run.is_some())
            .finish()
    }
}

impl ActionButton {
    /// A button with this text that closes the dialog.
    pub fn new(label: impl Into<String>) -> Self {
        ActionButton {
            label: label.into(),
            live_label: None,
            danger: false,
            disabled: None,
            run: None,
        }
    }

    /// Draws the button in the error color, for an action that cannot be
    /// undone.
    #[must_use]
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Takes the text from `label` while the dialog shows, in place of the
    /// text given to [`new`](Self::new), which stays as the fallback once the
    /// signal is gone.
    ///
    /// ```
    /// # use dioxus::prelude::*;
    /// # use orangui::ActionButton;
    /// # fn _f(seconds: Signal<u8>) {
    /// let label = use_memo(move || format!("Delete ({})", seconds()));
    /// let delete = ActionButton::new("Delete").live_label(label);
    /// # }
    /// ```
    #[must_use]
    pub fn live_label(mut self, label: impl Into<ReadSignal<String>>) -> Self {
        self.live_label = Some(label.into());
        self
    }

    /// Disables the button while `disabled` is true. A `Signal<bool>` passes
    /// as it is; a condition over other state passes as a `use_memo`.
    ///
    /// ```
    /// # use dioxus::prelude::*;
    /// # use orangui::ActionButton;
    /// # fn _f(name: Signal<String>) {
    /// let save = ActionButton::new("Save")
    ///     .disabled(use_memo(move || name().trim().is_empty()));
    /// # }
    /// ```
    #[must_use]
    pub fn disabled(mut self, disabled: impl Into<ReadSignal<bool>>) -> Self {
        self.disabled = Some(disabled.into());
        self
    }

    /// Starts closing the dialog, then runs `handler`, as
    /// [`DialogAction::Confirm`] does.
    ///
    /// ```
    /// # use dioxus::prelude::*;
    /// # use orangui::ActionButton;
    /// # fn _f(mut saved: Signal<bool>) {
    /// let save = ActionButton::new("Save").on_click(move || saved.set(true));
    /// # }
    /// ```
    #[must_use]
    pub fn on_click(mut self, handler: impl FnMut() + 'static) -> Self {
        self.run = Some(Run::Close(Rc::new(RefCell::new(handler))));
        self
    }

    /// Runs `handler` and leaves the dialog open, for a toggle inside it.
    ///
    /// ```
    /// # use dioxus::prelude::*;
    /// # use orangui::ActionButton;
    /// # fn _f(mut flipped: Signal<bool>) {
    /// let flip = ActionButton::new("Flip").stay_open(move || flipped.toggle());
    /// # }
    /// ```
    #[must_use]
    pub fn stay_open(mut self, handler: impl FnMut() + 'static) -> Self {
        self.run = Some(Run::StayOpen(Rc::new(RefCell::new(handler))));
        self
    }

    /// Runs `work` with the dialog held open: the button shows `busy_label`,
    /// and every action and the dim are inert until the future finishes (the
    /// OS back gesture still closes it). It resolves to `true` to close the
    /// dialog, or `false` to keep it open with its body as it was, for a
    /// request that failed.
    ///
    /// ```
    /// # use dioxus::prelude::*;
    /// # use orangui::ActionButton;
    /// # async fn clone_deck(_name: String) -> Result<(), ()> { Ok(()) }
    /// # fn _f(name: Signal<String>) {
    /// let save = ActionButton::new("Save")
    ///     .disabled(use_memo(move || name().trim().is_empty()))
    ///     .busy("Cloning...", move || async move {
    ///         clone_deck(name()).await.is_ok()
    ///     });
    /// # }
    /// ```
    #[must_use]
    pub fn busy<F>(
        mut self,
        busy_label: impl Into<String>,
        mut work: impl FnMut() -> F + 'static,
    ) -> Self
    where
        F: Future<Output = bool> + 'static,
    {
        let work = move || -> BusyFuture { Box::pin(work()) };
        self.run = Some(Run::Busy {
            label: busy_label.into(),
            work: Rc::new(RefCell::new(work)),
        });
        self
    }

    /// The text and whether the button takes clicks, given what the dialog's
    /// actions are doing.
    fn face(&self, activity: Activity) -> (String, bool) {
        let label = match (&self.run, activity) {
            (Some(Run::Busy { label, .. }), Activity::ThisRunning) => label.clone(),
            _ => self
                .live_label
                .and_then(|l| l.try_read().ok().map(|l| l.clone()))
                .unwrap_or_else(|| self.label.clone()),
        };
        let disabled = self
            .disabled
            .is_some_and(|d| d.try_read().is_ok_and(|d| *d));
        (label, !(disabled || activity != Activity::Idle))
    }
}

/// What a dialog's actions are doing, from one button's point of view.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Activity {
    /// Nothing is running.
    Idle,
    /// This button's own busy work is running.
    ThisRunning,
    /// Another action's busy work is running.
    OtherRunning,
}

/// What a dialog draws: its title, its body, and its actions left to right.
#[derive(Clone, PartialEq, Debug)]
pub struct DialogSpec {
    /// The heading.
    pub title: String,
    /// The body, inside the scrolling `.alert-dialog-description`.
    pub body: Element,
    /// The action row.
    pub actions: Vec<DialogAction>,
}

/// The dialog on screen, with the owner that put it there.
#[derive(Clone)]
struct Shown {
    spec: DialogSpec,
    owner: u64,
    /// The owner's close: sets its open signal false.
    close: Callback<()>,
    /// Fading out; inert until the animation ends and the slot clears.
    leaving: bool,
    /// The index of the action whose busy work is running.
    running: Option<usize>,
}

/// The app root's dialog slot. A `Copy` handle; create it with
/// [`use_dialog_host`], provide it with `use_context_provider` above the
/// router, and mount [`DialogHostView`] beside the router.
#[derive(Clone, Copy)]
pub struct DialogHost {
    slot: Signal<Option<Shown>>,
}

/// Creates the dialog slot.
#[must_use]
pub fn use_dialog_host() -> DialogHost {
    DialogHost {
        slot: use_signal(|| None),
    }
}

impl fmt::Debug for DialogHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DialogHost").finish_non_exhaustive()
    }
}

impl DialogHost {
    /// Starts the fade out of `owner`'s dialog, if it is the one showing and
    /// is not already leaving. Returns whether it started.
    fn leave(self, owner: u64) -> bool {
        let mut slot = self.slot;
        let showing = slot
            .peek()
            .as_ref()
            .is_some_and(|s| s.owner == owner && !s.leaving);
        if showing && let Some(shown) = slot.write().as_mut() {
            shown.leaving = true;
        }
        showing
    }

    /// Runs a click on `owner`'s action `index`, which [`ActionButton`]'s
    /// `run` describes: `None` closes the dialog, the rest as each says.
    fn activate(self, owner: u64, close: Callback<()>, index: usize, run: Option<&Run>) {
        match run {
            None => {
                if self.leave(owner) {
                    close.call(());
                }
            }
            Some(Run::Close(handler)) => {
                if self.leave(owner) {
                    close.call(());
                    (handler.borrow_mut())();
                }
            }
            Some(Run::StayOpen(handler)) => (handler.borrow_mut())(),
            Some(Run::Busy { work, .. }) => {
                let work = (work.borrow_mut())();
                self.set_running(owner, Some(index));
                spawn(async move {
                    let done = work.await;
                    self.set_running(owner, None);
                    if done && self.leave(owner) {
                        close.call(());
                    }
                });
            }
        }
    }

    /// Marks `owner`'s action `index` as running busy work, or clears it with
    /// `None`. Does nothing once another dialog holds the slot.
    fn set_running(self, owner: u64, index: Option<usize>) {
        let mut slot = self.slot;
        let mine = slot.peek().as_ref().is_some_and(|s| s.owner == owner);
        if mine && let Some(shown) = slot.write().as_mut() {
            shown.running = index;
        }
    }
}

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// Keeps the host's slot in step with `open`: when it turns true the dialog
/// `build` returns is shown, when it turns false the dialog fades out, and
/// another owner's dialog is never touched. Registers `open` with the
/// [`OverlayBackStack`](crate::OverlayBackStack), so the OS back gesture
/// closes the dialog first.
///
/// The spec is built when `open` turns true. A body that changes while it is
/// showing does so through its own components and signals, and an action
/// through the signals its [`ActionButton`] reads.
pub fn use_hosted_dialog(open: Signal<bool>, build: impl Fn() -> DialogSpec + 'static) {
    let host = use_context::<DialogHost>();
    let owner = use_hook(|| NEXT_OWNER.fetch_add(1, Ordering::Relaxed));
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    use_overlay_back(open);
    use_drop(move || {
        host.leave(owner);
    });
    use_effect(move || {
        if open() {
            let mut slot = host.slot;
            slot.set(Some(Shown {
                spec: build(),
                owner,
                close,
                leaving: false,
                running: None,
            }));
        } else {
            host.leave(owner);
        }
    });
}

/// A dialog drawn by the app root's [`DialogHostView`]. `open` is the
/// caller's, so it keeps the state that drives the dialog; the dialog sets it
/// false when any action, the dim or the OS back gesture closes it.
/// `children` is the body. Renders nothing in place.
#[component]
pub fn AlertDialog(
    open: Signal<bool>,
    title: String,
    actions: Vec<DialogAction>,
    children: Element,
) -> Element {
    let spec = DialogSpec {
        title,
        body: children,
        actions,
    };
    let mut latest = use_hook(|| CopyValue::new(spec.clone()));
    latest.set(spec);
    use_hosted_dialog(open, move || latest.cloned());
    rsx! {}
}

/// Draws the dialog in the [`DialogHost`], if any, above everything.
#[component]
pub fn DialogHostView() -> Element {
    let host = use_context::<DialogHost>();
    let Some(shown) = host.slot.read().clone() else {
        return rsx! {};
    };
    let Shown {
        spec,
        owner,
        close,
        leaving,
        running,
    } = shown;
    let state = if leaving { "closed" } else { "open" };
    let busy = running.is_some();
    let dismiss = move || {
        if !busy && host.leave(owner) {
            close.call(());
        }
    };
    rsx! {
        div {
            class: "alert-dialog-overlay",
            "data-state": state,
            onclick: move |_| dismiss(),
        }
        div {
            class: "alert-dialog",
            role: "dialog",
            aria_modal: "true",
            "data-state": state,
            onanimationend: move |evt| {
                if evt.animation_name() == "alert-dialog-out" {
                    let mut slot = host.slot;
                    if slot.peek().as_ref().is_some_and(|s| s.owner == owner && s.leaving) {
                        slot.set(None);
                    }
                }
            },
            h2 { class: "alert-dialog-title", "{spec.title}" }
            hr { class: "alert-dialog-rule" }
            div { class: "alert-dialog-description", {spec.body} }
            hr { class: "alert-dialog-rule" }
            div { class: "alert-dialog-actions",
                for (index , action) in spec.actions.into_iter().enumerate() {
                    match action {
                        DialogAction::Dismiss(label) => rsx! {
                            button {
                                class: "alert-dialog-cancel",
                                r#type: "button",
                                disabled: busy,
                                onclick: move |_| dismiss(),
                                "{label}"
                            }
                        },
                        DialogAction::Confirm { label, danger, on_confirm } => rsx! {
                            button {
                                class: if danger { "alert-dialog-action-danger" } else { "alert-dialog-action" },
                                r#type: "button",
                                disabled: busy,
                                onclick: move |_| {
                                    if host.leave(owner) {
                                        close.call(());
                                        on_confirm.call(());
                                    }
                                },
                                "{label}"
                            }
                        },
                        DialogAction::Link { label, href } => rsx! {
                            a {
                                class: "alert-dialog-cancel",
                                href,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                aria_disabled: if busy { "true" },
                                onclick: move |evt| {
                                    if busy {
                                        evt.prevent_default();
                                    }
                                    dismiss();
                                },
                                "{label}"
                            }
                        },
                        DialogAction::Button(button) => {
                            let this_running = running == Some(index);
                            let activity = match running {
                                None => Activity::Idle,
                                Some(i) if i == index => Activity::ThisRunning,
                                Some(_) => Activity::OtherRunning,
                            };
                            let (label, enabled) = button.face(activity);
                            let class = match (&button.run, button.danger) {
                                (None, false) => "alert-dialog-cancel",
                                (_, true) => "alert-dialog-action-danger",
                                (Some(_), false) => "alert-dialog-action",
                            };
                            let run = button.run;
                            rsx! {
                                button {
                                    class,
                                    r#type: "button",
                                    disabled: !enabled,
                                    aria_busy: if this_running { "true" },
                                    onclick: move |_| {
                                        if enabled {
                                            host.activate(owner, close, index, run.as_ref());
                                        }
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use dioxus::dioxus_core::NoOpMutations;

    use super::*;
    use crate::use_overlay_back_stack;

    /// The owner's state, reached from the test between renders.
    #[derive(Clone, Copy)]
    struct Probe {
        host: DialogHost,
        open: Signal<bool>,
        ready: Signal<bool>,
        seconds: Signal<u8>,
        flips: Signal<u32>,
        confirmed: Signal<u32>,
    }

    thread_local! {
        static PROBE: Cell<Option<Probe>> = const { Cell::new(None) };
        static OUTCOME: Cell<bool> = const { Cell::new(true) };
    }

    fn probe() -> Probe {
        PROBE.with(Cell::get).unwrap()
    }

    fn app() -> Element {
        let stack = use_overlay_back_stack();
        use_context_provider(|| stack);
        let host = use_dialog_host();
        use_context_provider(|| host);
        rsx! {
            Owner {}
            DialogHostView {}
        }
    }

    #[component]
    fn Owner() -> Element {
        let host = use_context::<DialogHost>();
        let open = use_signal(|| true);
        let ready = use_signal(|| false);
        let seconds = use_signal(|| 3u8);
        let mut flips = use_signal(|| 0u32);
        let mut confirmed = use_signal(|| 0u32);
        let label = use_memo(move || match seconds() {
            0 => "Delete".to_string(),
            n => format!("Delete ({n})"),
        });
        let blocked = use_memo(move || !ready());
        PROBE.with(|p| {
            p.set(Some(Probe {
                host,
                open,
                ready,
                seconds,
                flips,
                confirmed,
            }));
        });
        rsx! {
            AlertDialog {
                open,
                title: "Title",
                actions: vec![
                    DialogAction::Dismiss("Cancel".into()),
                    ActionButton::new("Delete")
                        .danger()
                        .live_label(label)
                        .disabled(blocked)
                        .on_click(move || confirmed += 1)
                        .into(),
                    ActionButton::new("Flip").stay_open(move || flips += 1).into(),
                    ActionButton::new("Save")
                        .busy("Saving...", || async { OUTCOME.with(Cell::get) })
                        .into(),
                ],
                "Body"
            }
        }
    }

    fn mount() -> VirtualDom {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        dom
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..8 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
    }

    /// The `<button>` tag whose text is `text`.
    #[expect(
        clippy::string_slice,
        reason = "a test helper over ASCII markup; a bad slice fails the test"
    )]
    fn button(html: &str, text: &str) -> String {
        let end = html.find(&format!(">{text}</button>")).unwrap();
        let start = html[..end].rfind("<button").unwrap();
        html[start..end].to_string()
    }

    /// Clicks action `index` as [`DialogHostView`] does.
    fn click(dom: &VirtualDom, index: usize) {
        let p = probe();
        dom.in_scope(ScopeId::APP, || {
            let shown = p.host.slot.peek().clone().unwrap();
            let run = match shown.spec.actions.get(index) {
                Some(DialogAction::Button(b)) => b.run.clone(),
                _ => None,
            };
            p.host
                .activate(shown.owner, shown.close, index, run.as_ref());
        });
    }

    fn leaving(dom: &VirtualDom) -> bool {
        let p = probe();
        dom.in_runtime(|| p.host.slot.peek().as_ref().is_some_and(|s| s.leaving))
    }

    #[test]
    fn label_and_disabled_follow_the_hosts_signals() {
        let mut dom = mount();
        let html = dioxus_ssr::render(&dom);
        assert!(button(&html, "Delete (3)").contains("disabled"));
        assert!(!button(&html, "Flip").contains("disabled"));

        let p = probe();
        dom.in_runtime(|| {
            let (mut seconds, mut ready) = (p.seconds, p.ready);
            seconds.set(0);
            ready.set(true);
        });
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        let delete = button(&html, "Delete");
        assert!(!delete.contains("disabled"));
        assert!(delete.contains("alert-dialog-action-danger"));
    }

    #[test]
    fn on_click_closes_then_runs() {
        let mut dom = mount();
        click(&dom, 1);
        settle(&mut dom);
        let p = probe();
        assert!(leaving(&dom));
        dom.in_runtime(|| {
            assert_eq!(*p.confirmed.peek(), 1);
            assert!(!*p.open.peek());
        });
    }

    #[test]
    fn stay_open_runs_and_keeps_the_dialog() {
        let mut dom = mount();
        click(&dom, 2);
        settle(&mut dom);
        let p = probe();
        assert!(!leaving(&dom));
        dom.in_runtime(|| {
            assert_eq!(*p.flips.peek(), 1);
            assert!(*p.open.peek());
        });
    }

    #[test]
    fn busy_shows_its_label_and_holds_every_action() {
        let mut dom = mount();
        let p = probe();
        dom.in_runtime(|| {
            let owner = p.host.slot.peek().as_ref().unwrap().owner;
            p.host.set_running(owner, Some(3));
        });
        dom.render_immediate(&mut NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let save = button(&html, "Saving...");
        assert!(save.contains("disabled"));
        assert!(save.contains("aria-busy"));
        assert!(button(&html, "Cancel").contains("disabled"));
        assert!(button(&html, "Flip").contains("disabled"));
    }

    #[test]
    fn busy_work_that_succeeds_closes() {
        OUTCOME.with(|o| o.set(true));
        let mut dom = mount();
        click(&dom, 3);
        settle(&mut dom);
        let p = probe();
        assert!(leaving(&dom));
        dom.in_runtime(|| assert!(!*p.open.peek()));
    }

    #[test]
    fn busy_work_that_fails_stays_open() {
        OUTCOME.with(|o| o.set(false));
        let mut dom = mount();
        click(&dom, 3);
        settle(&mut dom);
        let p = probe();
        assert!(!leaving(&dom));
        dom.in_runtime(|| {
            assert!(*p.open.peek());
            assert!(p.host.slot.peek().as_ref().unwrap().running.is_none());
        });
        let html = dioxus_ssr::render(&dom);
        assert!(!button(&html, "Save").contains("disabled"));
        assert!(html.contains("Body"));
    }

    #[test]
    fn face_without_signals() {
        let plain = ActionButton::new("Go");
        assert_eq!(plain.face(Activity::Idle), ("Go".to_string(), true));
        assert_eq!(
            plain.face(Activity::OtherRunning),
            ("Go".to_string(), false)
        );
        let busy = ActionButton::new("Save").busy("Saving...", || async { true });
        assert_eq!(
            busy.face(Activity::ThisRunning),
            ("Saving...".to_string(), false)
        );
        assert_eq!(busy.face(Activity::Idle), ("Save".to_string(), true));
    }

    #[test]
    fn a_clone_is_equal_and_a_rebuild_is_not() {
        let a = ActionButton::new("Flip").stay_open(|| {});
        assert_eq!(a, a.clone());
        assert_ne!(a, ActionButton::new("Flip").stay_open(|| {}));
        assert_eq!(ActionButton::new("Ok"), ActionButton::new("Ok"));
    }
}
