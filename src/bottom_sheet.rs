//! Slide-up bottom sheet, for the apps and the sites' theme picker. Styled
//! by `assets/components.css`.
//!
//! Dragging the header down pulls the sheet with the pointer. Letting go
//! past a quarter of the sheet's height, or with a downward flick, closes
//! it the way a backdrop tap does; anything less springs it back
//! (`orangui_core::sheet_drag` holds the thresholds).

use dioxus::prelude::*;
use orangui_core::sheet_drag;

use crate::overlay_stack::{OverlayLift, use_overlay_back_action};
use crate::{ActionBar, Button, ButtonVariant, entrance};

/// One pointer pulling the sheet by its header.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    pointer: i32,
    start_y: f64,
    offset: f64,
    last_y: f64,
    last_ms: f64,
    velocity: f64,
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

    // The drag in progress, and the sheet's height, measured as one starts.
    let mut drag = use_signal(|| None::<Drag>);
    let mut sheet_el = use_signal(|| None::<MountedEvent>);
    let mut height = use_signal(|| 0.0_f64);

    let start_drag = move |evt: PointerEvent| {
        if !open() || drag.peek().is_some() {
            return;
        }
        let y = evt.client_coordinates().y;
        drag.set(Some(Drag {
            pointer: evt.pointer_id(),
            start_y: y,
            offset: 0.0,
            last_y: y,
            last_ms: now_ms(),
            velocity: 0.0,
        }));
        if let Some(el) = sheet_el.peek().clone() {
            spawn(async move {
                if let Ok(rect) = el.get_client_rect().await {
                    height.set(rect.height());
                }
            });
        }
    };
    let move_drag = move |evt: PointerEvent| {
        let Some(mut d) = *drag.peek() else {
            return;
        };
        if d.pointer != evt.pointer_id() {
            return;
        }
        let y = evt.client_coordinates().y;
        let now = now_ms();
        d.velocity = sheet_drag::velocity(y - d.last_y, now - d.last_ms);
        d.offset = sheet_drag::offset(d.start_y, y);
        d.last_y = y;
        d.last_ms = now;
        drag.set(Some(d));
    };
    let mut end_drag = move |evt: PointerEvent, cancelled: bool| {
        let Some(d) = *drag.peek() else {
            return;
        };
        if d.pointer != evt.pointer_id() {
            return;
        }
        drag.set(None);
        let velocity = sheet_drag::release_velocity(d.velocity, now_ms() - d.last_ms);
        if !cancelled && sheet_drag::closes(d.offset, *height.peek(), velocity) {
            dismiss.call(());
        }
    };

    // While a drag holds the sheet it follows the pointer with no easing;
    // dropping the style lets the class's transition slide it home or away.
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
    let drag_style =
        drag().map(|d| format!("transform: translateY({}px); transition: none;", d.offset));
    let sheet_style = match (layer_style.clone(), drag_style) {
        (Some(layer), Some(drag)) => Some(format!("{layer} {drag}")),
        (layer, drag) => layer.or(drag),
    };
    let dim_class = match dim {
        SheetDim::Always => "",
        SheetDim::Phone => " sheet-dim-phone",
    };

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
            onpointermove: move_drag,
            onpointerup: move |evt| end_drag(evt, false),
            onpointercancel: move |evt| end_drag(evt, true),
        }
        div {
            class: if hidden {
                "bottom-sheet snap"
            } else if open() {
                "bottom-sheet show"
            } else if leaving() > 0 {
                "bottom-sheet bottom-sheet-leaving"
            } else if mounted() {
                "bottom-sheet"
            } else {
                "bottom-sheet bottom-sheet-premount"
            },
            style: sheet_style,
            onmounted: move |evt| sheet_el.set(Some(evt)),
            onpointermove: move_drag,
            onpointerup: move |evt| end_drag(evt, false),
            onpointercancel: move |evt| end_drag(evt, true),
            div { class: "sheet-header", onpointerdown: start_drag,
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

/// Wraps one [`BottomSheet`] (or a component that renders one) so a wide
/// window draws it as `wide`. The frame has no box of its own; the sheet and
/// its backdrop stay fixed to the window.
#[component]
pub fn SheetFrame(#[props(default)] wide: SheetWide, children: Element) -> Element {
    rsx! {
        div { class: wide.class(), {children} }
    }
}
