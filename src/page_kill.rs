//! Recovery from a killed web page, for the apps on iOS.
//!
//! iOS kills a backgrounded WKWebView's web content process under memory
//! pressure. wry's navigation delegate takes WebKit's termination callback,
//! so WebKit does not reload the page, and dioxus 0.7 sets no handler of its
//! own: the page stays dead, its last frame on screen and every tap lost,
//! while the app runs on behind it.
//!
//! [`use_page_kill_recovery`] hears the kill by wrapping that callback on
//! wry's delegate class, calling wry's own first. It then gives the page
//! about a second and a half to answer, so a dioxus that reloads the page
//! itself is left to it, and otherwise asks the host to rebuild: a new
//! window with a fresh app in it, the dead one closed. The host's session and
//! settings come back from storage as at a launch; what a screen held in
//! memory does not.
//!
//! A second kill within [`REPEAT_WINDOW`] of the first does not rebuild on
//! its own, so a page that keeps dying cannot loop: a native overlay covers
//! the dead page with one line and a Reload button, which rebuilds. A third
//! in a row turns the button into Restart, which quits the app so the next
//! tap on its icon launches it clean.
//!
//! Android needs none of this: wry's web view client leaves
//! `onRenderProcessGone` to the system, which ends the app with its renderer,
//! and the app launches fresh. On every target but iOS the hook does nothing.

use std::ffi::c_void;
use std::ptr::NonNull;
use std::time::Duration;

use dioxus::prelude::*;

/// How close a kill must follow the last one to count as a repeat: one
/// rebuild, then the overlay until the rebuilt page has lasted this long,
/// as WebKit does with its own reloads.
pub const REPEAT_WINDOW: Duration = Duration::from_secs(30);

/// Rebuilds the app when iOS kills its web page. Call it once, at the app's
/// root, so the rebuilt app calls it too.
///
/// - `app_name`: the app's name, for the overlay's line ("Zwipe needs to
///   reload").
/// - `ios_webview`: returns the app's WKWebView, the one the overlay covers,
///   as [`use_os_back`](crate::use_os_back) takes it.
/// - `rebuild`: opens a new window with a fresh `VirtualDom` of the app
///   (with the root context its launch gave it) and closes this one. A host
///   on `dioxus/mobile` calls `window().new_window(..)` and then
///   `window().close()`.
pub fn use_page_kill_recovery(
    app_name: &'static str,
    ios_webview: fn() -> Option<NonNull<c_void>>,
    rebuild: Callback<()>,
) {
    #[cfg(target_os = "ios")]
    {
        use_hook(ios::wrap_termination);
        use_future(move || async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
            ios::route_kills(tx);
            while rx.recv().await.is_some() {
                if page_answers().await {
                    continue;
                }
                match ios::next_step() {
                    ios::Step::Rebuild => rebuild.call(()),
                    ios::Step::Overlay(button) => {
                        let Some(webview) = ios_webview() else {
                            rebuild.call(());
                            continue;
                        };
                        let (tap_tx, mut tap_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
                        ios::cover(webview, app_name, button, tap_tx);
                        if tap_rx.recv().await.is_some() {
                            match button {
                                ios::Button::Reload => rebuild.call(()),
                                ios::Button::Restart => ios::quit(),
                            }
                        }
                    }
                }
            }
        });
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (app_name, ios_webview, rebuild);
    }
}

/// Whether the page answers within the grace a self-healing host needs.
#[cfg(target_os = "ios")]
async fn page_answers() -> bool {
    /// Long enough for a dioxus that reloads a killed page itself.
    const GRACE: Duration = Duration::from_millis(1500);
    let mut eval = document::eval("dioxus.send(true);");
    matches!(
        tokio::time::timeout(GRACE, eval.recv::<bool>()).await,
        Ok(Ok(true))
    )
}

/// The termination wrapper, the repeat guard and the overlay.
#[cfg(target_os = "ios")]
mod ios {
    use std::ffi::{CString, c_void};
    use std::ptr::NonNull;
    use std::sync::{Mutex, OnceLock};
    use std::time::Instant;

    use objc2::encode::{Encode, Encoding};
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, AnyObject, Imp, NSObject, Sel};
    use objc2::{AnyThread, MainThreadMarker, class, define_class, msg_send, sel};
    use tokio::sync::mpsc::UnboundedSender;

    use super::REPEAT_WINDOW;

    /// What a kill leads to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Step {
        /// The first in a while: rebuild at once.
        Rebuild,
        /// A repeat: cover the page and wait for a tap on `Button`.
        Overlay(Button),
    }

    /// The overlay's one button.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Button {
        /// Rebuilds the app.
        Reload,
        /// Quits the app, the last resort once a rebuild has not held.
        Restart,
    }

    /// The kills in a row and when the last one came.
    static STREAK: Mutex<Option<(Instant, u32)>> = Mutex::new(None);

    /// Where a kill goes: the channel of the app mounted now.
    static KILL_TX: Mutex<Option<UnboundedSender<()>>> = Mutex::new(None);

    /// Where the overlay's tap goes.
    static TAP_TX: Mutex<Option<UnboundedSender<()>>> = Mutex::new(None);

    /// wry's own termination method, which the wrapper calls first.
    static WRY_TERMINATED: OnceLock<Imp> = OnceLock::new();

    /// Sends kills to `tx` from now on, in place of the channel of an app
    /// that has since been rebuilt.
    pub(super) fn route_kills(tx: UnboundedSender<()>) {
        if let Ok(mut slot) = KILL_TX.lock() {
            *slot = Some(tx);
        }
    }

    /// What this kill leads to, counting it into the streak.
    pub(super) fn next_step() -> Step {
        let now = Instant::now();
        let Ok(mut streak) = STREAK.lock() else {
            return Step::Rebuild;
        };
        let count = match *streak {
            Some((last, count)) if now.duration_since(last) < REPEAT_WINDOW => {
                count.saturating_add(1)
            }
            _ => 1,
        };
        *streak = Some((now, count));
        match count {
            1 => Step::Rebuild,
            2 => Step::Overlay(Button::Reload),
            _ => Step::Overlay(Button::Restart),
        }
    }

    /// Quits the app on the user's tap, so the next launch starts clean.
    #[expect(
        clippy::exit,
        reason = "the overlay's last resort: a page that keeps dying, quit on the user's tap"
    )]
    pub(super) fn quit() {
        std::process::exit(0);
    }

    /// The wrapper put in place of wry's termination method: wry's first, so
    /// a handler wry calls still runs, then the kill on to the app.
    unsafe extern "C-unwind" fn terminated(this: &AnyObject, cmd: Sel, webview: &AnyObject) {
        if let Some(&wry) = WRY_TERMINATED.get() {
            // SAFETY: `wry` is the method's own implementation, which takes
            // the receiver, the selector and the webview.
            let wry: unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject) =
                unsafe { std::mem::transmute(wry) };
            // SAFETY: called with the arguments the runtime handed us.
            unsafe { wry(this, cmd, webview) };
        }
        if let Ok(slot) = KILL_TX.lock()
            && let Some(tx) = slot.as_ref()
        {
            let _ = tx.send(());
        }
    }

    /// Wraps wry's termination method, once per app life.
    pub(super) fn wrap_termination() {
        if WRY_TERMINATED.get().is_some() {
            return;
        }
        let Some(class) = AnyClass::get(c"WryNavigationDelegate") else {
            dioxus::logger::tracing::warn!(
                "page kill: no WryNavigationDelegate class; not watching"
            );
            return;
        };
        let Some(method) = class.instance_method(sel!(webViewWebContentProcessDidTerminate:))
        else {
            dioxus::logger::tracing::warn!(
                "page kill: wry has no termination method; not watching"
            );
            return;
        };
        let wrapper: unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject) = terminated;
        // SAFETY: an `Imp` is any method implementation; the runtime calls
        // it with the method's own arguments, which `terminated` takes.
        let wrapper: Imp = unsafe { std::mem::transmute(wrapper) };
        // SAFETY: the wrapper has the replaced method's signature.
        let wry = unsafe { method.set_implementation(wrapper) };
        let _ = WRY_TERMINATED.set(wry);
    }

    /// A `CGPoint`, for the frames UIKit takes.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Point {
        x: f64,
        y: f64,
    }

    /// A `CGSize`.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Size {
        width: f64,
        height: f64,
    }

    /// A `CGRect`.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Rect {
        origin: Point,
        size: Size,
    }

    // SAFETY: the layout and encoding of CoreGraphics' CGPoint on 64-bit.
    unsafe impl Encode for Point {
        const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
    }

    // SAFETY: the layout and encoding of CoreGraphics' CGSize on 64-bit.
    unsafe impl Encode for Size {
        const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
    }

    // SAFETY: the layout and encoding of CoreGraphics' CGRect on 64-bit.
    unsafe impl Encode for Rect {
        const ENCODING: Encoding = Encoding::Struct("CGRect", &[Point::ENCODING, Size::ENCODING]);
    }

    impl Rect {
        const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
            Self {
                origin: Point { x, y },
                size: Size { width, height },
            }
        }
    }

    /// `UIViewAutoresizing`: flexible width and height.
    const FILL: usize = 2 | 16;
    /// `UIViewAutoresizing`: flexible width, top and bottom margins.
    const ROW: usize = 2 | 8 | 32;
    /// `UIViewAutoresizing`: flexible margins on every side.
    const CENTERED: usize = 1 | 4 | 8 | 32;
    /// `UIControlEventTouchUpInside`.
    const TOUCH_UP_INSIDE: usize = 1 << 6;
    /// `UIButtonTypeSystem`.
    const BUTTON_SYSTEM: isize = 1;
    /// `NSTextAlignmentCenter`.
    const ALIGN_CENTER: isize = 1;

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; no Drop impl.
        #[unsafe(super(NSObject))]
        #[name = "OranguiPageKillTarget"]
        struct TapTarget;

        impl TapTarget {
            /// Target-action for the overlay's button.
            #[unsafe(method(tapped:))]
            fn tapped(&self, _sender: &AnyObject) {
                if let Ok(slot) = TAP_TX.lock()
                    && let Some(tx) = slot.as_ref()
                {
                    let _ = tx.send(());
                }
            }
        }
    );

    impl TapTarget {
        fn new() -> Retained<Self> {
            let this = Self::alloc().set_ivars(());
            // SAFETY: the designated initializer of an NSObject subclass.
            unsafe { msg_send![super(this), init] }
        }
    }

    /// An `NSString` holding `text`.
    fn ns_string(text: &str) -> Option<Retained<AnyObject>> {
        let text = CString::new(text).ok()?;
        // SAFETY: a valid NUL-terminated UTF-8 string, copied by NSString.
        unsafe { msg_send![class!(NSString), stringWithUTF8String: text.as_ptr()] }
    }

    /// A `UIColor` from 8-bit channels, opaque.
    fn color(r: u8, g: u8, b: u8) -> Retained<AnyObject> {
        let channel = |c: u8| f64::from(c) / 255.0;
        // SAFETY: the class factory takes four CGFloat channels in 0..=1.
        unsafe {
            msg_send![class!(UIColor), colorWithRed: channel(r), green: channel(g), blue: channel(b), alpha: 1.0_f64]
        }
    }

    /// The monospaced system font at `size` points.
    fn mono_font(size: f64) -> Retained<AnyObject> {
        // SAFETY: the class factory takes a point size and a weight
        // (UIFontWeightRegular is 0).
        unsafe { msg_send![class!(UIFont), monospacedSystemFontOfSize: size, weight: 0.0_f64] }
    }

    /// Sends `setter` with one object argument to `target`, on the main
    /// thread.
    fn set_object(target: &AnyObject, setter: Sel, value: &AnyObject) {
        // SAFETY: every caller names a UIKit setter that takes one object,
        // and calls it on the main thread; the return value is ignored.
        let _: *mut AnyObject =
            unsafe { msg_send![target, performSelector: setter, withObject: value] };
    }

    /// Covers `webview`, the dead page's WKWebView, with the app's backdrop,
    /// one line and `button`; its tap goes to `tap`. Runs on the main thread,
    /// as UI effects do.
    pub(super) fn cover(
        webview: NonNull<c_void>,
        app_name: &str,
        button: Button,
        tap: UnboundedSender<()>,
    ) {
        if MainThreadMarker::new().is_none() {
            dioxus::logger::tracing::error!("page kill: the overlay draws on the main thread only");
            return;
        }
        if let Ok(mut slot) = TAP_TX.lock() {
            *slot = Some(tap);
        }
        // SAFETY: the host hands over its live WKWebView.
        let webview: &AnyObject = unsafe { webview.cast::<AnyObject>().as_ref() };
        // SAFETY: `bounds` is a UIView getter returning a CGRect.
        let bounds: Rect = unsafe { msg_send![webview, bounds] };
        let (width, height) = (bounds.size.width, bounds.size.height);
        let (line, title) = match button {
            Button::Reload => (format!("{app_name} needs to reload"), "Reload"),
            Button::Restart => (format!("{app_name} needs to restart"), "Restart"),
        };
        let (Some(line), Some(title)) = (ns_string(&line), ns_string(title)) else {
            return;
        };
        let ink = color(0xeb, 0xdb, 0xb2);
        let font = mono_font(17.0);

        let cover = view(class!(UIView), bounds, FILL);
        set_object(&cover, sel!(setBackgroundColor:), &color(0x28, 0x28, 0x28));

        let label = view(
            class!(UILabel),
            Rect::new(0.0, height / 2.0 - 64.0, width, 32.0),
            ROW,
        );
        set_object(&label, sel!(setText:), &line);
        set_object(&label, sel!(setTextColor:), &ink);
        set_object(&label, sel!(setFont:), &font);
        // SAFETY: UILabel's alignment setter takes an NSTextAlignment.
        let (): () = unsafe { msg_send![&*label, setTextAlignment: ALIGN_CENTER] };
        set_object(&cover, sel!(addSubview:), &label);

        // SAFETY: the class factory for a system button.
        let reload: Retained<AnyObject> =
            unsafe { msg_send![class!(UIButton), buttonWithType: BUTTON_SYSTEM] };
        let frame = Rect::new(width / 2.0 - 70.0, height / 2.0, 140.0, 48.0);
        // SAFETY: UIView's frame setter takes a CGRect.
        let (): () = unsafe { msg_send![&*reload, setFrame: frame] };
        // SAFETY: UIView's autoresizing setter takes a UIViewAutoresizing.
        let (): () = unsafe { msg_send![&*reload, setAutoresizingMask: CENTERED] };
        // SAFETY: UIButton's title setter takes a string and a UIControlState.
        let (): () = unsafe { msg_send![&*reload, setTitle: &*title, forState: 0_usize] };
        // SAFETY: UIButton's title color setter takes a color and a state.
        let (): () = unsafe { msg_send![&*reload, setTitleColor: &*ink, forState: 0_usize] };
        // SAFETY: `titleLabel` is a UIButton getter returning its label.
        let title_label: Retained<AnyObject> = unsafe { msg_send![&*reload, titleLabel] };
        set_object(&title_label, sel!(setFont:), &font);
        // SAFETY: `layer` is a UIView getter returning its CALayer.
        let layer: Retained<AnyObject> = unsafe { msg_send![&*reload, layer] };
        // SAFETY: `CGColor` is a UIColor getter returning a CGColorRef.
        let ink_cg: *const c_void = unsafe { msg_send![&*ink, CGColor] };
        // SAFETY: CALayer's border color setter takes a CGColorRef.
        let (): () = unsafe { msg_send![&*layer, setBorderColor: ink_cg] };
        // SAFETY: CALayer's border width setter takes a CGFloat.
        let (): () = unsafe { msg_send![&*layer, setBorderWidth: 1.0_f64] };
        // SAFETY: CALayer's corner radius setter takes a CGFloat.
        let (): () = unsafe { msg_send![&*layer, setCornerRadius: 10.0_f64] };
        let target = TapTarget::new();
        // SAFETY: UIControl's target-action setter, with a target that
        // implements the action's selector.
        unsafe {
            let (): () = msg_send![&*reload, addTarget: &*target, action: sel!(tapped:), forControlEvents: TOUCH_UP_INSIDE];
        }
        set_object(&cover, sel!(addSubview:), &reload);
        set_object(webview, sel!(addSubview:), &cover);
        // A control holds its target weakly, so the target outlives this
        // call: one per overlay, and an overlay ends the app or its window.
        std::mem::forget(target);
    }

    /// A new view of `class` at `frame`, resizing by `mask`.
    fn view(class: &AnyClass, frame: Rect, mask: usize) -> Retained<AnyObject> {
        // SAFETY: `alloc` is valid on every UIView subclass.
        let alloc: Allocated<AnyObject> = unsafe { msg_send![class, alloc] };
        // SAFETY: UIView's designated initializer takes a CGRect.
        let view: Retained<AnyObject> = unsafe { msg_send![alloc, initWithFrame: frame] };
        // SAFETY: UIView's autoresizing setter takes a UIViewAutoresizing.
        let (): () = unsafe { msg_send![&*view, setAutoresizingMask: mask] };
        view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeat_is_webkits_thirty_seconds() {
        assert_eq!(REPEAT_WINDOW, Duration::from_secs(30));
    }
}
