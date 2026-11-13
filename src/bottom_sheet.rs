//! Slide-up bottom sheet, for the apps and the sites' theme picker. Styled
//! by `assets/components.css`.
//!
//! Dragging the header down pulls the sheet with the pointer. Letting go
//! past a quarter of the sheet's height, or with a downward flick, closes
//! it the way a backdrop tap does; anything less springs it back
//! (`orangui_core::sheet_drag` holds the thresholds).

use std::fmt;

use dioxus::prelude::*;
use orangui_core::sheet_drag;

use crate::overlay_stack::{OverlayLift, use_overlay_back_action};
use crate::{ActionBar, Button, ButtonVariant, entrance};

/// One pointer pulling the sheet by its header.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    pointer: i32,
    start: f64,
    offset: f64,
    last: f64,
    last_ms: f64,
    velocity: f64,
}

/// A pointer down on a sheet pulled from anywhere on it, whose way is not read yet: where it went
/// down, along the pull and across it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Arm {
    pointer: i32,
    along: f64,
    across: f64,
}

/// How long the sheet takes to slide away, matching the `.bottom-sheet`
/// transform transition in components.css.
const SLIDE_MS: u32 = 300;

/// Milliseconds on a monotonic clock: the page's on the web, the process's
/// elsewhere.
fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or(0.0, |p| p.now())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        use std::time::Instant;

        static START: OnceLock<Instant> = OnceLock::new();
        START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
    }
}

/// A sheet's pull by its header, shared by [`BottomSheet`] and any other sheet that slides up
/// from the bottom: [`start`](Self::start) on the header's pointer down (or
/// [`arm`](Self::arm) on the pointer down anywhere on a sheet pulled sideways),
/// [`follow`](Self::follow), [`release`](Self::release) and [`cancel`](Self::cancel) on the
/// sheet's and its backdrop's pointer move, up and cancel, [`mounted`](Self::mounted) on the
/// sheet's mount, and [`style`](Self::style) and [`class`](Self::class) on the sheet. Letting
/// go past a quarter of the sheet's height, or with a downward flick, calls the sheet's dismiss
/// (`orangui_core::sheet_drag` holds the thresholds); anything less springs it back.
#[derive(Clone, Copy, PartialEq)]
pub struct SheetDrag {
    open: ReadSignal<bool>,
    dismiss: Callback<()>,
    toward: Pull,
    drag: Signal<Option<Drag>>,
    armed: Signal<Option<Arm>>,
    sheet: Signal<Option<MountedEvent>>,
    extent: Signal<f64>,
}

/// Which way a sheet is pulled to close: a bottom sheet down, a drawer along the right edge to
/// the right, one along the left edge to the left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Pull {
    /// Down, by the pointer's y, against the sheet's height.
    #[default]
    Down,
    /// To the right, by the pointer's x, against the sheet's width.
    Right,
    /// To the left, by the pointer's x reversed, against the sheet's width.
    Left,
}

impl Pull {
    /// The class a sheet carries while pulled this way, with its leading space: it moves the
    /// sheet by `--sheet-drag` with no easing.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Down => " sheet-dragging",
            Self::Right => " sheet-dragging-right",
            Self::Left => " sheet-dragging-left",
        }
    }

    /// The pointer's coordinate across the pull.
    fn across(self, evt: &PointerEvent) -> f64 {
        let at = evt.client_coordinates();
        match self {
            Self::Down => at.x,
            Self::Right | Self::Left => at.y,
        }
    }

    /// The pointer's coordinate along the pull, growing the way the sheet closes.
    fn along(self, evt: &PointerEvent) -> f64 {
        let at = evt.client_coordinates();
        match self {
            Self::Down => at.y,
            Self::Right => at.x,
            Self::Left => -at.x,
        }
    }
}

impl fmt::Debug for SheetDrag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SheetDrag").finish_non_exhaustive()
    }
}

/// The pull of a sheet shown while `open`, which calls `dismiss` when let go far or fast enough.
#[must_use]
pub fn use_sheet_drag(open: ReadSignal<bool>, dismiss: Callback<()>) -> SheetDrag {
    use_sheet_drag_toward(open, dismiss, Pull::Down)
}

/// [`use_sheet_drag`] pulled `toward` a way of its own, such as a drawer to the right.
#[must_use]
pub fn use_sheet_drag_toward(
    open: ReadSignal<bool>,
    dismiss: Callback<()>,
    toward: Pull,
) -> SheetDrag {
    SheetDrag {
        open,
        dismiss,
        toward,
        drag: use_signal(|| None),
        armed: use_signal(|| None),
        sheet: use_signal(|| None),
        extent: use_signal(|| 0.0),
    }
}

impl SheetDrag {
    /// Starts a pull at the pointer, measuring the sheet along the pull (its height, or its
    /// width for a pull to either side), while the sheet is open and no other pull holds it.
    pub fn start(self, evt: &PointerEvent) {
        if !(self.open)() || self.drag.peek().is_some() {
            return;
        }
        self.begin(evt.pointer_id(), self.toward.along(evt));
    }

    /// Readies a pull at the pointer for a sheet pulled from anywhere on it, whose content
    /// scrolls the other way: the pull starts once the pointer moves past
    /// [`sheet_drag::ARM_SLOP_PX`] more along the pull than across it
    /// ([`sheet_drag::intent`]), and a move across it leaves the sheet to scroll. Nothing on the
    /// sheet changes until the pull starts, so a scroll the browser takes over runs undisturbed.
    pub fn arm(mut self, evt: &PointerEvent) {
        if !*self.open.peek() || self.drag.peek().is_some() {
            return;
        }
        self.armed.set(Some(Arm {
            pointer: evt.pointer_id(),
            along: self.toward.along(evt),
            across: self.toward.across(evt),
        }));
    }

    /// Holds the pull of `pointer` from `at` along it and measures the sheet.
    fn begin(mut self, pointer: i32, at: f64) {
        self.drag.set(Some(Drag {
            pointer,
            start: at,
            offset: 0.0,
            last: at,
            last_ms: now_ms(),
            velocity: 0.0,
        }));
        if let Some(el) = self.sheet.peek().clone() {
            let mut extent = self.extent;
            let toward = self.toward;
            spawn(async move {
                if let Ok(rect) = el.get_client_rect().await {
                    extent.set(match toward {
                        Pull::Down => rect.height(),
                        Pull::Right | Pull::Left => rect.width(),
                    });
                }
            });
        }
    }

    /// Reads an armed pointer's way ([`arm`](Self::arm)) as it moves: past the slop along the
    /// pull it starts the pull from where it went down, across it the arm drops.
    fn read_arm(mut self, evt: &PointerEvent) {
        let Some(arm) = *self.armed.peek() else {
            return;
        };
        if arm.pointer != evt.pointer_id() {
            return;
        }
        let along = self.toward.along(evt) - arm.along;
        let across = self.toward.across(evt) - arm.across;
        match sheet_drag::intent(along, across) {
            None => {}
            Some(sheet_drag::Intent::Scroll) => self.armed.set(None),
            Some(sheet_drag::Intent::Pull) => {
                self.armed.set(None);
                if *self.open.peek() && self.drag.peek().is_none() {
                    self.begin(arm.pointer, arm.along);
                }
            }
        }
    }

    /// Moves the sheet with the pointer holding the pull.
    pub fn follow(mut self, evt: &PointerEvent) {
        if self.armed.peek().is_some() {
            self.read_arm(evt);
        }
        let Some(mut d) = *self.drag.peek() else {
            return;
        };
        if d.pointer != evt.pointer_id() {
            return;
        }
        let at = self.toward.along(evt);
        let now = now_ms();
        d.velocity = sheet_drag::velocity(at - d.last, now - d.last_ms);
        d.offset = sheet_drag::offset(d.start, at);
        d.last = at;
        d.last_ms = now;
        self.drag.set(Some(d));
    }

    /// Ends the pull as the pointer lets go: past the thresholds the sheet is dismissed, else it
    /// springs back.
    pub fn release(self, evt: &PointerEvent) {
        if let Some(d) = self.end(evt) {
            let velocity = sheet_drag::release_velocity(d.velocity, now_ms() - d.last_ms);
            if sheet_drag::closes(d.offset, *self.extent.peek(), velocity) {
                self.dismiss.call(());
            }
        }
    }

    /// Ends the pull with the sheet springing back, as the browser takes the pointer.
    pub fn cancel(self, evt: &PointerEvent) {
        let _ = self.end(evt);
    }

    /// Keeps the sheet's element, measured as each pull starts.
    pub fn mounted(mut self, evt: MountedEvent) {
        self.sheet.set(Some(evt));
    }

    /// The sheet's style while a pull holds it: the pull's distance as `--sheet-drag`, which
    /// [`class`](Self::class) moves the sheet by. `None` otherwise.
    #[must_use]
    pub fn style(self) -> Option<String> {
        (self.drag)().map(|d| Self::style_for(d.offset))
    }

    /// The class the sheet carries while a pull holds it, with its leading space
    /// ([`Pull::class`]): it follows the pointer with no easing, and its going lets the sheet's
    /// transition slide it home or away. Empty otherwise.
    #[must_use]
    pub fn class(self) -> &'static str {
        if (self.drag)().is_some() {
            self.toward.class()
        } else {
            ""
        }
    }

    /// The inline style of a pull `offset` pixels along its way. Only the distance goes inline: Dioxus
    /// keeps an inline property that a new `style` value leaves out, so an inline transform
    /// would outlast the pull on a sheet whose style carries its layer.
    #[must_use]
    pub fn style_for(offset: f64) -> String {
        format!("--sheet-drag: {offset}px;")
    }

    /// Drops the pull the pointer of `evt` holds, handing it back, and any arm.
    fn end(mut self, evt: &PointerEvent) -> Option<Drag> {
        if self.armed.peek().is_some() {
            self.armed.set(None);
        }
        let d = (*self.drag.peek())?;
        if d.pointer != evt.pointer_id() {
            return None;
        }
        self.drag.set(None);
        Some(d)
    }
}

/// A bottom sheet with backdrop, title, content slot, and footer.
///
/// `footer` overrides the default single "Close" button (e.g. a Back/Save
/// pair). `closable` puts a small × in the title bar's left corner instead,
/// and with no `footer` the sheet then has no action bar at all.
/// `on_dismiss` fires when the backdrop is tapped, the × is pressed, a drag
/// lets go past its threshold or the OS back gesture lands, and owns the
/// close: a sheet that sets it closes itself, which lets the theme sheet
/// hold still through the wipe that restores a discarded pick. `hidden`
/// drops the sheet and backdrop with no slide, for a sheet that leaves
/// inside that wipe so the new snapshot is taken without it.
///
/// While the sheet slides away it carries `bottom-sheet-leaving`, so a rule
/// that keeps something painted only for a shown sheet, such as the folded
/// nav panel around a [`ThemePicker`](crate::ThemePicker)'s sheet, can hold
/// it through the slide.
///
/// Registers with the [`OverlayBackStack`](crate::OverlayBackStack) when the
/// host provides one above it, so the OS back gesture closes it, and draws at
/// its place there: a sheet opened later sits over one opened earlier. Inside
/// an [`OverlayLift`]'s container it reports that z-index while it shows, so
/// the container rises with it. `dim` says whether its backdrop dims the
/// page ([`SheetDim`]).
#[component]
pub fn BottomSheet(
    mut open: Signal<bool>,
    title: String,
    children: Element,
    footer: Option<Element>,
    on_dismiss: Option<EventHandler<()>>,
    /// Drops the sheet with no slide away and no `bottom-sheet-leaving` mark.
    #[props(default)]
    hidden: bool,
    /// A hint's open signal. Given one, the sheet's header carries the same
    /// "?" the page headers do; without one, the corner stays empty.
    hint: Option<Signal<bool>>,
    /// An action in the title bar's right corner, before the "?": a chip
    /// that opens a sibling flow, such as a sheet's Forgot password.
    #[props(default)]
    trailing: Option<Element>,
    /// A × in the title bar that dismisses the sheet. Without a `footer`, it
    /// replaces the default "Close" bar.
    #[props(default)]
    closable: bool,
    /// Whether the backdrop dims what is under the sheet.
    #[props(default)]
    dim: SheetDim,
) -> Element {
    // The OS back gesture closes the sheet the way a backdrop tap does.
    let dismiss = use_callback(move |()| {
        let mut open = open;
        match on_dismiss {
            Some(h) => h.call(()),
            None => open.set(false),
        }
    });
    let layer = use_overlay_back_action(open.into(), dismiss);

    // The first render carries `transition: none` through the premount class,
    // dropped once mounted. Without it iOS WebKit replays the transform
    // transition on insert and a freshly mounted sheet visibly slides away.
    // The flag has to flip after WebKit's first post-insert paint, so it
    // waits a couple of frames. It is a class rather than an inline style
    // because clearing an inline `transition: none` does not reliably take in
    // that WebView, and the rule would linger and kill every slide.
    let mut mounted = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            entrance::sleep_ms(50).await;
            mounted.set(true);
        });
    });

    // The slide away, numbered so a close that follows another within the
    // slide is not cut short by the first one's timer. A sheet dropped with
    // `hidden` leaves with no slide and marks nothing.
    let mut leaving = use_signal(|| 0u32);
    let mut was_open = use_signal(|| false);
    use_effect(use_reactive!(|hidden| {
        let is_open = open();
        let closing = *was_open.peek() && !is_open;
        was_open.set(is_open);
        if !closing || hidden {
            return;
        }
        let slide = leaving.peek().wrapping_add(1).max(1);
        leaving.set(slide);
        spawn(async move {
            entrance::sleep_ms(SLIDE_MS).await;
            if *leaving.peek() == slide {
                leaving.set(0);
            }
        });
    }));

    // The way it pulls: down below the kit's wide window, else by the form its frame gives it.
    let form = try_use_context::<SheetForm>().map_or(SheetWide::Sheet, |SheetForm(form)| form);
    let wide = crate::page_keys::use_media(KIT_WIDE_QUERY);
    let pull = sheet_pull(form, wide());
    let drag = use_sheet_drag_toward(open.into(), dismiss, pull.unwrap_or_default());
    let pulls = if pull.is_some() { " sheet-pulls" } else { "" };

    // The sheet's layer holds through its slide away, so it leaves from where
    // it was rather than under what it covered.
    let mut held = use_signal(|| None::<u32>);
    use_effect(move || {
        if let Some(z) = layer() {
            held.set(Some(z));
        } else if held.peek().is_some() {
            spawn(async move {
                entrance::sleep_ms(SLIDE_MS).await;
                if layer.peek().is_none() {
                    held.set(None);
                }
            });
        }
    });
    let lift = try_use_context::<OverlayLift>();
    use_effect(move || {
        if let Some(OverlayLift(mut lifted)) = lift {
            lifted.set(held());
        }
    });
    let layer_style = held().map(|z| format!("--overlay-z: {z};"));
    let drag_style = drag.style();
    let pulled = drag.class();
    let sheet_style = match (layer_style.clone(), drag_style) {
        (Some(layer), Some(drag)) => Some(format!("{layer} {drag}")),
        (layer, drag) => layer.or(drag),
    };
    let dim_class = match dim {
        SheetDim::Always => "",
        SheetDim::Phone => " sheet-dim-phone",
    };
    // A sheet that dims the page is modal; one that previews its change leaves the page in use.
    let modal = (dim == SheetDim::Always).then_some("true");

    rsx! {
        div {
            class: if hidden {
                "sheet-backdrop snap{dim_class}"
            } else if open() {
                "sheet-backdrop show{dim_class}"
            } else {
                "sheet-backdrop{dim_class}"
            },
            style: layer_style,
            onclick: move |_| dismiss.call(()),
            onpointermove: move |evt| drag.follow(&evt),
            onpointerup: move |evt| drag.release(&evt),
            onpointercancel: move |evt| drag.cancel(&evt),
        }
        div {
            class: if hidden {
                "bottom-sheet snap"
            } else if open() {
                "bottom-sheet show{pulls}{pulled}"
            } else if leaving() > 0 {
                "bottom-sheet bottom-sheet-leaving"
            } else if mounted() {
                "bottom-sheet"
            } else {
                "bottom-sheet bottom-sheet-premount"
            },
            style: sheet_style,
            role: "dialog",
            "aria-label": "{title}",
            "aria-modal": modal,
            onmounted: move |evt| drag.mounted(evt),
            onpointermove: move |evt| drag.follow(&evt),
            onpointerup: move |evt| drag.release(&evt),
            onpointercancel: move |evt| drag.cancel(&evt),
            onlostpointercapture: move |evt| drag.cancel(&evt),
            if let Some(toward) = pull {
                SheetHandle { toward, drag }
            }
            div {
                class: "sheet-header",
                onpointerdown: move |evt| {
                    if pull.is_some() {
                        drag.start(&evt);
                    }
                },
                if closable {
                    button {
                        r#type: "button",
                        class: "sheet-close",
                        aria_label: "Close",
                        onpointerdown: move |evt| evt.stop_propagation(),
                        onclick: move |_| dismiss.call(()),
                        "\u{00d7}"
                    }
                }
                span { class: "sheet-title", "{title}" }
                {trailing}
                if let Some(mut hint) = hint {
                    Button {
                        variant: ButtonVariant::Action,
                        class: "page-header-corner",
                        onclick: move |_| hint.set(true),
                        "?"
                    }
                }
            }
            div { class: "sheet-content",
                div { class: "flex-col", style: "gap: 0.5rem;", {children} }
            }
            if let Some(f) = footer {
                ActionBar { {f} }
            } else if !closable {
                ActionBar {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| dismiss.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}

/// Whether a [`BottomSheet`]'s backdrop dims what is under it. Either way the
/// backdrop takes a click outside the sheet, which dismisses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetDim {
    /// Dims at every width: a sheet that asks for a decision.
    #[default]
    Always,
    /// Dims only below the kit's modal width (60rem): a sheet that previews
    /// its change live, such as the themes, leaves the page in full view on a
    /// wide window.
    Phone,
}

/// Where a [`BottomSheet`] sits on a wide window. Below the kit's bar
/// breakpoint every form is the bottom sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetWide {
    /// The bottom sheet at every width.
    #[default]
    Sheet,
    /// A centered modal, for a short form or list.
    Modal,
    /// A drawer: an inset panel along the right edge under the bar, for a
    /// sheet that edits what stays in view beside it. The page dims only a
    /// little. `--sheet-drawer-top` on the root sets where it starts.
    Drawer,
}

impl SheetWide {
    /// The frame's classes for this form.
    #[must_use]
    pub fn class(self) -> &'static str {
        match self {
            Self::Sheet => "sheet-form",
            Self::Modal => "sheet-form sheet-form-modal",
            Self::Drawer => "sheet-form sheet-form-drawer",
        }
    }
}

/// The form a [`SheetFrame`] gives the sheet inside it on a wide window, which decides the way
/// the sheet pulls there ([`sheet_pull`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheetForm(pub SheetWide);

/// The query for the kit's wide window, where a framed sheet takes its wide form.
pub const KIT_WIDE_QUERY: &str = "(min-width: 60rem)";

/// The way a sheet in `form` pulls to close, `wide` the kit's wide window ([`KIT_WIDE_QUERY`]):
/// below it every sheet is a bottom sheet pulled down; on it a bottom sheet still pulls down, a
/// drawer toward its edge (the right), and a centered modal doesn't pull.
#[must_use]
pub fn sheet_pull(form: SheetWide, wide: bool) -> Option<Pull> {
    if !wide {
        return Some(Pull::Down);
    }
    match form {
        SheetWide::Sheet => Some(Pull::Down),
        SheetWide::Drawer => Some(Pull::Right),
        SheetWide::Modal => None,
    }
}

/// Installs, once a page, the pointer capture for pulls: a pointer that goes down on a pulling
/// sheet's title bar or handle (not on a button, link or field in it) is captured to the sheet,
/// so every move and the release reach the sheet's own handlers wherever the pointer goes: over
/// something drawn above the sheet, or out of the window or frame. A touch is captured to what it
/// touched already. Losing the capture ends the pull (`onlostpointercapture`).
const CAPTURE_JS: &str = r"
if (!window.__oranguiSheetCapture) {
    window.__oranguiSheetCapture = true;
    document.addEventListener('pointerdown', (e) => {
        const zone = e.target.closest && e.target.closest('.bottom-sheet.sheet-pulls > .sheet-header, .bottom-sheet.sheet-pulls > .sheet-handle');
        if (!zone || e.target.closest('button, a, input, select, textarea')) return;
        zone.parentElement.setPointerCapture(e.pointerId);
    }, true);
}
";

/// The handle a sheet that pulls shows where to drag it: a short bar at the top of a bottom
/// sheet, a short upright bar on the inner edge of a drawer. Decorative to a screen reader; a
/// pull starts from it as from the title bar, with a mouse as well as a touch.
#[component]
pub fn SheetHandle(toward: Pull, drag: SheetDrag) -> Element {
    use_hook(|| {
        document::eval(CAPTURE_JS);
    });
    let side = match toward {
        Pull::Down => "sheet-handle-down",
        Pull::Right => "sheet-handle-right",
        Pull::Left => "sheet-handle-left",
    };
    rsx! {
        div {
            class: "sheet-handle {side}",
            "aria-hidden": "true",
            onpointerdown: move |evt| drag.start(&evt),
        }
    }
}

/// Wraps one [`BottomSheet`] (or a component that renders one) so a wide
/// window draws it as `wide`. The frame has no box of its own; the sheet and
/// its backdrop stay fixed to the window.
#[component]
pub fn SheetFrame(#[props(default)] wide: SheetWide, children: Element) -> Element {
    use_context_provider(|| SheetForm(wide));
    rsx! {
        div { class: wide.class(), {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pull_from_the_title_bar_or_handle_captures_the_pointer_to_the_sheet() {
        assert!(
            CAPTURE_JS.contains("e.target.closest('.bottom-sheet.sheet-pulls > .sheet-header, .bottom-sheet.sheet-pulls > .sheet-handle')"),
            "{CAPTURE_JS}"
        );
        assert!(
            CAPTURE_JS.contains("e.target.closest('button, a, input, select, textarea')) return;"),
            "a control in the title bar keeps its click: {CAPTURE_JS}"
        );
        assert!(
            CAPTURE_JS.contains("zone.parentElement.setPointerCapture(e.pointerId)"),
            "{CAPTURE_JS}"
        );
    }

    #[test]
    fn an_open_sheet_never_hands_a_pull_to_the_browser_s_refresh() {
        let css = crate::COMPONENTS_CSS;
        assert!(
            css.contains("html:has(.bottom-sheet.show),\nhtml:has(.bottom-sheet.show) body,\n.bottom-sheet .sheet-content {\n    overscroll-behavior-y: contain;\n}"),
            "a pull down on an open sheet at the top of its content reloads nothing"
        );
    }

    #[test]
    fn a_sheet_pulls_down_on_a_phone_and_by_its_form_on_a_wide_window() {
        for form in [SheetWide::Sheet, SheetWide::Modal, SheetWide::Drawer] {
            assert_eq!(
                sheet_pull(form, false),
                Some(Pull::Down),
                "{form:?} below the wide window"
            );
        }
        assert_eq!(sheet_pull(SheetWide::Sheet, true), Some(Pull::Down));
        assert_eq!(
            sheet_pull(SheetWide::Drawer, true),
            Some(Pull::Right),
            "a drawer toward its edge"
        );
        assert_eq!(
            sheet_pull(SheetWide::Modal, true),
            None,
            "a centered modal holds still"
        );
    }

    #[test]
    fn the_pull_beats_a_framed_sheet_s_resting_place_and_shows_a_grab() {
        let css = crate::COMPONENTS_CSS;
        for rule in [
            ".sheet-form > .bottom-sheet.show.sheet-dragging {",
            ".sheet-form > .bottom-sheet.show.sheet-dragging-right {",
            ".sheet-form > .bottom-sheet.show.sheet-dragging-left {",
            ".sheet-handle-down {",
            ".sheet-handle-right {",
            ".sheet-handle-left {",
        ] {
            assert!(css.contains(rule), "{rule}");
        }
        assert!(css.contains("cursor: grab;") && css.contains("cursor: grabbing;"));
    }
}
