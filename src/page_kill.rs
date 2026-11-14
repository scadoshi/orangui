//! Recovery from a killed web page, for the apps on iOS.
//!
//! iOS kills a backgrounded WKWebView's web content process under memory
//! pressure. wry's navigation delegate takes WebKit's termination callback,
//! so WebKit does not reload the page, and dioxus 0.7 sets no handler of its
//! own: the page stays dead, its last frame on screen and every tap lost,
//! while the app runs on behind it.
//!
//! [`use_page_kill_recovery`] hears the kill by wrapping that callback on
//! wry's delegate class, calling wry's own first. Everything after runs
//! natively on the main run loop, never as a task of the app: a `VirtualDom`
//! that renders while its page is dead waits for good on edits the page will
//! never take, and its tasks with it. A second and a half after the kill it
//! looks whether a page has loaded since, so a dioxus that reloads the page
//! itself is left to it, and otherwise calls the host's rebuild: a new window
//! with a fresh app in it, the dead one closed. The host's session and
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
#[cfg(target_os = "ios")]
use std::rc::Rc;
use std::time::Duration;
#[cfg(any(target_os = "ios", test))]
use std::time::Instant;

#[cfg(target_os = "ios")]
use dioxus::prelude::use_hook;

/// How close a kill must follow the last one to count as a repeat: one
/// rebuild, then the overlay until the rebuilt page has lasted this long,
/// as WebKit does with its own reloads.
pub const REPEAT_WINDOW: Duration = Duration::from_secs(30);

/// Rebuilds the app when iOS kills its web page. Call it once, at the app's
/// root, so the rebuilt app calls it too; the newest call is the one a kill
/// reaches.
///
/// - `app_name`: the app's name, for the overlay's line ("Zwipe needs to
///   reload").
/// - `ios_webview`: returns the app's WKWebView, the one the overlay covers,
///   as [`use_os_back`](crate::use_os_back) takes it.
/// - `rebuild`: opens a new window with a fresh `VirtualDom` of the app
///   (with the root context its launch gave it) and closes this one. It runs
///   on the main thread outside the app's runtime, so it captures what it
///   needs at the call: a host on `dioxus/mobile` takes `window()` there,
///   and calls `new_window(..)` then `close()` on it, which the event loop
///   handles in that order.
pub fn use_page_kill_recovery(
    #[cfg_attr(
        not(target_os = "ios"),
        expect(unused_variables, reason = "only iOS kills the page's process")
    )]
    app_name: &'static str,
    #[cfg_attr(
        not(target_os = "ios"),
        expect(unused_variables, reason = "only iOS kills the page's process")
    )]
    ios_webview: fn() -> Option<NonNull<c_void>>,
    #[cfg_attr(
        not(target_os = "ios"),
        expect(unused_variables, reason = "only iOS kills the page's process")
    )]
    rebuild: impl Fn() + 'static,
) {
    #[cfg(target_os = "ios")]
    {
        let mut rebuild = Some(rebuild);
        use_hook(move || {
            ios::wrap_delegate();
            if let Some(rebuild) = rebuild.take() {
                ios::watch(app_name, ios_webview(), Rc::new(rebuild));
            }
        });
    }
}

/// What a kill leads to.
#[cfg(any(target_os = "ios", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// The first in a while: rebuild at once.
    Rebuild,
    /// A repeat: cover the page and wait for a tap on the button.
    Overlay(Button),
}

/// The overlay's one button.
#[cfg(any(target_os = "ios", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    /// Rebuilds the app.
    Reload,
    /// Quits the app, the last resort once a rebuild has not held.
    Restart,
}

/// What a kill at `now` leads to, given the streak before it (when the last
/// kill came and how many came in a row), and the streak's new length. A
/// kill more than [`REPEAT_WINDOW`] after the last one starts a new streak.
#[cfg(any(target_os = "ios", test))]
fn step_after(streak: Option<(Instant, u32)>, now: Instant) -> (Step, u32) {
    let count = match streak {
        Some((last, count)) if now.saturating_duration_since(last) < REPEAT_WINDOW => {
            count.saturating_add(1)
        }
        _ => 1,
    };
    let step = match count {
        1 => Step::Rebuild,
        2 => Step::Overlay(Button::Reload),
        _ => Step::Overlay(Button::Restart),
    };
    (step, count)
}

/// The delegate wrappers, the deferred check, the repeat guard and the
/// overlay.
#[cfg(target_os = "ios")]
mod ios {
    use std::cell::RefCell;
    use std::ffi::{CString, c_void};
    use std::panic::{self, AssertUnwindSafe};
    use std::ptr::NonNull;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, OnceLock};
    use std::time::Instant;

    use objc2::encode::{Encode, Encoding};
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, AnyObject, Imp, NSObject, Sel};
    use objc2::{AnyThread, MainThreadMarker, class, define_class, msg_send, sel};

    use super::{Button, Step, step_after};

    /// How long after a kill to look whether a page has loaded since: long
    /// enough for a dioxus that reloads a killed page itself.
    const GRACE_SECS: f64 = 1.5;

    /// The newest app's recovery: what the overlay says, the webview it
    /// covers and the host's rebuild. Main-thread state, as every callback
    /// that reads it runs there.
    struct Watch {
        app_name: &'static str,
        webview: Option<NonNull<c_void>>,
        rebuild: Rc<dyn Fn()>,
    }

    thread_local! {
        static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) };
        /// The object the deferred check and the overlay's tap call into.
        static TARGET: RefCell<Option<Retained<Target>>> = const { RefCell::new(None) };
        /// What the overlay's button does when tapped.
        static OVERLAY_BUTTON: RefCell<Option<Button>> = const { RefCell::new(None) };
    }

    /// The kills in a row and when the last one came.
    static STREAK: Mutex<Option<(Instant, u32)>> = Mutex::new(None);

    /// Whether a page has committed a navigation since the last kill.
    static NAVIGATED_SINCE_KILL: AtomicBool = AtomicBool::new(false);

    /// wry's own termination and commit methods, which the wrappers call
    /// first.
    static WRY_TERMINATED: OnceLock<Imp> = OnceLock::new();
    static WRY_COMMITTED: OnceLock<Imp> = OnceLock::new();

    /// Makes this app's recovery the one a kill reaches.
    pub(super) fn watch(
        app_name: &'static str,
        webview: Option<NonNull<c_void>>,
        rebuild: Rc<dyn Fn()>,
    ) {
        WATCH.with(|slot| {
            *slot.borrow_mut() = Some(Watch {
                app_name,
                webview,
                rebuild,
            });
        });
    }

    /// Runs `body` at an Objective-C entry point, where a panic must not
    /// unwind into the runtime: a panic is logged and stops there.
    fn guarded(entry: &str, body: impl FnOnce()) {
        if panic::catch_unwind(AssertUnwindSafe(body)).is_err() {
            dioxus::logger::tracing::error!("page kill: {entry} panicked; recovery skipped");
        }
    }

    /// Calls the newest app's rebuild.
    fn rebuild() {
        let rebuild = WATCH.with(|slot| slot.borrow().as_ref().map(|w| Rc::clone(&w.rebuild)));
        if let Some(rebuild) = rebuild {
            rebuild();
        }
    }

    /// What this kill leads to, counting it into the streak.
    fn next_step() -> Step {
        let now = Instant::now();
        let Ok(mut streak) = STREAK.lock() else {
            return Step::Rebuild;
        };
        let (step, count) = step_after(*streak, now);
        *streak = Some((now, count));
        step
    }

    /// The check a kill schedules: leaves a page that has loaded again to
    /// whoever loaded it, else rebuilds or covers the dead page.
    fn check() {
        if NAVIGATED_SINCE_KILL.load(Ordering::Acquire) {
            return;
        }
        match next_step() {
            Step::Rebuild => rebuild(),
            Step::Overlay(button) => {
                let watched =
                    WATCH.with(|slot| slot.borrow().as_ref().map(|w| (w.app_name, w.webview)));
                match watched {
                    Some((app_name, Some(webview))) => {
                        OVERLAY_BUTTON.with(|slot| *slot.borrow_mut() = Some(button));
                        cover(webview, app_name, button);
                    }
                    _ => rebuild(),
                }
            }
        }
    }

    /// The overlay's button: rebuilds, or quits on the last resort.
    fn tapped() {
        match OVERLAY_BUTTON.with(|slot| slot.borrow_mut().take()) {
            Some(Button::Reload) => rebuild(),
            Some(Button::Restart) => quit(),
            None => {}
        }
    }

    /// Quits the app on the user's tap, so the next launch starts clean.
    #[expect(
        clippy::exit,
        reason = "the overlay's last resort: a page that keeps dying, quit on the user's tap"
    )]
    fn quit() {
        std::process::exit(0);
    }

    /// The object the deferred check and the overlay's button call into,
    /// made once and kept for the app's life.
    fn target() -> Option<Retained<Target>> {
        TARGET.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_none() {
                *slot = Some(Target::new());
            }
            slot.clone()
        })
    }

    /// The wrapper put in place of wry's termination method: wry's first, so
    /// a handler wry calls still runs, then the check, a grace period later
    /// on the main run loop.
    unsafe extern "C-unwind" fn terminated(this: &AnyObject, cmd: Sel, webview: &AnyObject) {
        if let Some(&wry) = WRY_TERMINATED.get() {
            // SAFETY: `wry` is the method's own implementation, which takes
            // the receiver, the selector and the webview.
            let wry: unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject) =
                unsafe { std::mem::transmute(wry) };
            // SAFETY: called with the arguments the runtime handed us.
            unsafe { wry(this, cmd, webview) };
        }
        guarded("the termination wrapper", || {
            NAVIGATED_SINCE_KILL.store(false, Ordering::Release);
            if let Some(target) = target() {
                let nothing: Option<&AnyObject> = None;
                // SAFETY: NSObject's delayed perform, with a selector the
                // target implements, on the main thread's run loop.
                unsafe {
                    let (): () = msg_send![&*target, performSelector: sel!(check:), withObject: nothing, afterDelay: GRACE_SECS];
                }
            }
        });
    }

    /// The wrapper put in place of wry's commit method: wry's first, then a
    /// note that a page has loaded since the kill.
    unsafe extern "C-unwind" fn committed(
        this: &AnyObject,
        cmd: Sel,
        webview: &AnyObject,
        navigation: &AnyObject,
    ) {
        if let Some(&wry) = WRY_COMMITTED.get() {
            // SAFETY: `wry` is the method's own implementation, which takes
            // the receiver, the selector, the webview and the navigation.
            let wry: unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject, &AnyObject) =
                unsafe { std::mem::transmute(wry) };
            // SAFETY: called with the arguments the runtime handed us.
            unsafe { wry(this, cmd, webview, navigation) };
        }
        NAVIGATED_SINCE_KILL.store(true, Ordering::Release);
    }

    /// Wraps wry's termination and commit methods, once per app life.
    pub(super) fn wrap_delegate() {
        if WRY_TERMINATED.get().is_some() {
            return;
        }
        let Some(class) = AnyClass::get(c"WryNavigationDelegate") else {
            dioxus::logger::tracing::warn!(
                "page kill: no WryNavigationDelegate class; not watching"
            );
            return;
        };
        let (Some(terminate), Some(commit)) = (
            class.instance_method(sel!(webViewWebContentProcessDidTerminate:)),
            class.instance_method(sel!(webView:didCommitNavigation:)),
        ) else {
            dioxus::logger::tracing::warn!(
                "page kill: wry's delegate lacks a method; not watching"
            );
            return;
        };
        let on_commit: unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject, &AnyObject) =
            committed;
        // SAFETY: an `Imp` is any method implementation; the runtime calls
        // it with the method's own arguments, which `committed` takes.
        let on_commit: Imp = unsafe { std::mem::transmute(on_commit) };
        // SAFETY: the wrapper has the replaced method's signature.
        let wry = unsafe { commit.set_implementation(on_commit) };
        // Empty here: the guard at the top returns once they are set.
        let _: Result<(), _> = WRY_COMMITTED.set(wry);
        let on_terminate: unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject) = terminated;
        // SAFETY: as above, for `terminated` and the termination method.
        let on_terminate: Imp = unsafe { std::mem::transmute(on_terminate) };
        // SAFETY: the wrapper has the replaced method's signature.
        let wry = unsafe { terminate.set_implementation(on_terminate) };
        let _: Result<(), _> = WRY_TERMINATED.set(wry);
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

    /// The overlay's backdrop: gruvbox dark's `--bg-primary`, the launch color
    /// ([`crate::BOOT_BG`]).
    const BACKDROP: (u8, u8, u8) = (0x28, 0x28, 0x28);
    /// The overlay's text and button outline: gruvbox dark's `--text-primary`.
    const INK: (u8, u8, u8) = (0xeb, 0xdb, 0xb2);
    /// The line's and the button's type size, in points.
    const TYPE_PT: f64 = 17.0;
    /// The line's height, and how far above the middle it sits.
    const LINE_H: f64 = 32.0;
    const LINE_RISE: f64 = 64.0;
    /// The button's size, its top at the middle.
    const BUTTON_W: f64 = 140.0;
    const BUTTON_H: f64 = 48.0;
    /// The button's outline width and corner radius, in points.
    const BUTTON_BORDER: f64 = 1.0;
    const BUTTON_RADIUS: f64 = 10.0;

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
        struct Target;

        impl Target {
            /// The check a kill schedules on the main run loop.
            #[unsafe(method(check:))]
            fn check_later(&self, _nothing: Option<&AnyObject>) {
                guarded("the deferred check", check);
            }

            /// Target-action for the overlay's button.
            #[unsafe(method(tapped:))]
            fn tapped_button(&self, _sender: &AnyObject) {
                guarded("the overlay's button", tapped);
            }
        }
    );

    impl Target {
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
    fn color((r, g, b): (u8, u8, u8)) -> Retained<AnyObject> {
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
    /// one line and `button`, whose tap goes to [`tapped`]. Runs on the main
    /// thread, as the run loop's callbacks do.
    fn cover(webview: NonNull<c_void>, app_name: &str, button: Button) {
        if MainThreadMarker::new().is_none() {
            dioxus::logger::tracing::error!("page kill: the overlay draws on the main thread only");
            return;
        }
        let Some(target) = target() else {
            return;
        };
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
        let ink = color(INK);
        let font = mono_font(TYPE_PT);

        let cover = view(class!(UIView), bounds, FILL);
        set_object(&cover, sel!(setBackgroundColor:), &color(BACKDROP));

        let label = view(
            class!(UILabel),
            Rect::new(0.0, height / 2.0 - LINE_RISE, width, LINE_H),
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
        let frame = Rect::new(
            width / 2.0 - BUTTON_W / 2.0,
            height / 2.0,
            BUTTON_W,
            BUTTON_H,
        );
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
        let (): () = unsafe { msg_send![&*layer, setBorderWidth: BUTTON_BORDER] };
        // SAFETY: CALayer's corner radius setter takes a CGFloat.
        let (): () = unsafe { msg_send![&*layer, setCornerRadius: BUTTON_RADIUS] };
        // SAFETY: UIControl's target-action setter, with a target that
        // implements the action's selector.
        unsafe {
            let (): () = msg_send![&*reload, addTarget: &*target, action: sel!(tapped:), forControlEvents: TOUCH_UP_INSIDE];
        }
        set_object(&cover, sel!(addSubview:), &reload);
        set_object(webview, sel!(addSubview:), &cover);
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
    fn the_first_kill_rebuilds_a_repeat_offers_reload_then_restart() {
        let start = Instant::now();
        let (first, count) = step_after(None, start);
        assert_eq!((first, count), (Step::Rebuild, 1));
        let soon = start + Duration::from_secs(5);
        let (second, count) = step_after(Some((start, count)), soon);
        assert_eq!((second, count), (Step::Overlay(Button::Reload), 2));
        let (third, count) = step_after(Some((soon, count)), soon + Duration::from_secs(5));
        assert_eq!((third, count), (Step::Overlay(Button::Restart), 3));
    }

    #[test]
    fn a_kill_after_the_window_starts_a_new_streak() {
        let start = Instant::now();
        let later = start + REPEAT_WINDOW;
        assert_eq!(step_after(Some((start, 2)), later), (Step::Rebuild, 1));
    }
}
