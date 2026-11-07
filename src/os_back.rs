//! The OS back intent for the apps: the iOS left-edge swipe and Android's
//! back gesture and button.
//!
//! An app's router keeps its history in memory, so the OS back never reaches
//! it on its own. [`use_os_back`] bridges it: the intent closes the top-most
//! overlay on the [`OverlayBackStack`](crate::OverlayBackStack) first, and
//! with none open it calls the host's `go_back`. On Android, a `go_back`
//! that finds nothing to go back to finishes the Activity, which leaves the
//! app.
//!
//! **iOS.** A `UIScreenEdgePanGestureRecognizer` on the app's WKWebView sends
//! on a channel as the pan begins, so the swipe acts as fast as tapping Back.
//! WKWebView's own back-forward gesture, fed by a `history.pushState` trap,
//! waits for a native page transition that fails to animate and takes about
//! three seconds to settle. The recognizer runs alongside any other pan, so a
//! list still gliding from a flick does not swallow the swipe.
//!
//! **Android.** The host patches its generated `MainActivity` so an
//! `OnBackPressedCallback` dispatches a DOM event on the window, and the hook
//! listens for that event. The patch stays in the host's build scripts,
//! beside its package name.
//!
//! **iOS layout.** The same hook also calls [`fill_window`] once with the
//! WKWebView it receives, so the page covers the whole window on iOS 18,
//! where UIKit otherwise insets the page by the safe area. A host that does
//! not use the hook calls [`fill_window`] itself.
//!
//! On every other target the hook does nothing, so a browser keeps its own
//! back button.

use std::ffi::c_void;
use std::ptr::NonNull;
#[cfg(target_os = "ios")]
use std::sync::atomic::{AtomicBool, Ordering};

use dioxus::prelude::*;

#[cfg(any(target_os = "ios", target_os = "android"))]
use crate::OverlayBackStack;

/// Bridges the OS back intent into the app. Call it once, in a layout that
/// wraps every route, below the provided
/// [`OverlayBackStack`](crate::OverlayBackStack).
///
/// - `android_event`: the name of the DOM event the host's patched
///   `MainActivity` dispatches on the window for the back intent, such as
///   `"app:back"`. A plain name: it is written into a script as is.
/// - `ios_webview`: returns the app's WKWebView, which the edge-swipe
///   recognizer attaches to. A host on `dioxus/mobile` takes it from
///   `dioxus::mobile::window().webview` through wry's `WebViewExtIOS`; `None`
///   leaves the swipe off.
/// - `go_back`: navigates back and returns `true`, or returns `false` when
///   there is nothing to go back to. A router host checks
///   `can_go_back` before `go_back`.
pub fn use_os_back(
    android_event: &'static str,
    ios_webview: fn() -> Option<NonNull<c_void>>,
    go_back: Callback<(), bool>,
) {
    #[cfg(target_os = "ios")]
    {
        let _ = android_event;
        let mut overlays: OverlayBackStack = use_context();
        use_effect(move || {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
            spawn(async move {
                while rx.recv().await.is_some() {
                    if !overlays.close_top() {
                        let _ = go_back.call(());
                    }
                }
            });
            ios::route_edge_back(tx);
            if let Some(webview) = ios_webview() {
                if !EDGE_BACK_INSTALLED.swap(true, Ordering::AcqRel) {
                    ios::install_edge_back(webview);
                }
                if !FILLED_WINDOW.swap(true, Ordering::AcqRel) {
                    ios::fill_window(webview);
                }
            }
        });
    }

    #[cfg(target_os = "android")]
    {
        let _ = ios_webview;
        let mut overlays: OverlayBackStack = use_context();
        use_future(move || async move {
            let mut eval = document::eval(&format!(
                "if (window.__oranguiBack) window.removeEventListener('{android_event}', window.__oranguiBack);\
                 window.__oranguiBack = () => dioxus.send(1);\
                 window.addEventListener('{android_event}', window.__oranguiBack);"
            ));
            while eval.recv::<i32>().await.is_ok() {
                if !overlays.close_top() && !go_back.call(()) {
                    android::finish_activity();
                }
            }
        });
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        let _ = (android_event, ios_webview, go_back);
    }
}

/// Whether [`use_os_back`] has already laid the page out over the window:
/// the hook mounts once per layout, and the layout call runs once per app
/// life.
#[cfg(target_os = "ios")]
static FILLED_WINDOW: AtomicBool = AtomicBool::new(false);

/// Whether the edge-swipe recognizer is on the WKWebView: one per app life,
/// however often the layout that calls [`use_os_back`] mounts.
#[cfg(target_os = "ios")]
static EDGE_BACK_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Lays the page out over the whole window instead of the safe area, on iOS.
///
/// On iOS 18, UIKit adds the safe-area insets to the WKWebView's scroll view
/// as content insets, so WebKit lays the page out in the window less the
/// status bar strip and the home indicator strip, and the webview's own
/// background shows in the band left under the page. This sets the scroll
/// view's `contentInsetAdjustmentBehavior` to never and forces a layout pass,
/// so the page covers the whole window. iOS 26 lays the page out that way on
/// its own, and the call is harmless there.
///
/// `env(safe-area-inset-*)` keeps reporting the insets afterward, so the
/// bands that want them still get them, and the host's viewport meta needs
/// `viewport-fit=cover` for the page to extend under them at all.
///
/// [`use_os_back`] calls this once with the WKWebView it receives, so a host
/// on the hook needs nothing more. A host that does not use the hook calls it
/// once at mount, on the main thread, with the pointer `dioxus/mobile` hands
/// out through wry's `WebViewExtIOS`. On every other target it does nothing.
pub fn fill_window(webview: NonNull<c_void>) {
    #[cfg(target_os = "ios")]
    ios::fill_window(webview);
    #[cfg(not(target_os = "ios"))]
    let _ = webview;
}

/// The left-edge swipe: a `UIScreenEdgePanGestureRecognizer` on the WKWebView
/// whose target sends on the mounted layout's channel, and the whole-window
/// layout.
#[cfg(target_os = "ios")]
mod ios {
    use std::ffi::c_void;
    use std::ptr::NonNull;
    use std::sync::Mutex;

    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyObject, Bool, NSObject};
    use objc2::{AnyThread, MainThreadMarker, class, define_class, msg_send, sel};
    use tokio::sync::mpsc::UnboundedSender;

    /// `UIGestureRecognizerStateBegan`: the target fires once, as the pan
    /// starts.
    const STATE_BEGAN: isize = 1;
    /// `UIRectEdgeLeft` (1 << 1): only pans from the left screen edge.
    const EDGE_LEFT: usize = 2;
    /// `UIScrollViewContentInsetAdjustmentNever`, from UIKit's
    /// `UIScrollViewContentInsetAdjustmentBehavior`.
    const ADJUSTMENT_NEVER: isize = 2;

    /// Where an edge swipe goes: the channel of the layout mounted now.
    static EDGE_BACK_TX: Mutex<Option<UnboundedSender<()>>> = Mutex::new(None);

    /// Sends the edge swipe to `tx` from now on, in place of the channel of a
    /// layout that has since unmounted.
    pub(super) fn route_edge_back(tx: UnboundedSender<()>) {
        if let Ok(mut slot) = EDGE_BACK_TX.lock() {
            *slot = Some(tx);
        }
    }

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; no Drop impl.
        #[unsafe(super(NSObject))]
        #[name = "OranguiEdgeBackTarget"]
        struct EdgeBackTarget;

        impl EdgeBackTarget {
            /// Target-action for the edge pan. `sender` is the recognizer.
            #[unsafe(method(handleEdgePan:))]
            fn handle_edge_pan(&self, sender: &AnyObject) {
                // SAFETY: `state` is a valid selector on UIGestureRecognizer.
                let state: isize = unsafe { msg_send![sender, state] };
                if state == STATE_BEGAN
                    && let Ok(slot) = EDGE_BACK_TX.lock()
                    && let Some(tx) = slot.as_ref()
                {
                    let _ = tx.send(());
                }
            }

            /// Delegate: lets the edge pan run alongside another pan. A gliding
            /// list's scroll view claims the next touch at once to catch the
            /// scroll, and one recognizer winning would cancel the edge swipe.
            #[unsafe(method(gestureRecognizer:shouldRecognizeSimultaneouslyWithGestureRecognizer:))]
            fn should_recognize_simultaneously(&self, _edge: &AnyObject, other: &AnyObject) -> Bool {
                // SAFETY: `isKindOfClass:` is a valid selector on every NSObject.
                let is_pan: Bool = unsafe { msg_send![other, isKindOfClass: class!(UIPanGestureRecognizer)] };
                is_pan
            }
        }
    );

    impl EdgeBackTarget {
        fn new() -> Retained<Self> {
            let this = Self::alloc().set_ivars(());
            // SAFETY: the designated initializer of an NSObject subclass.
            unsafe { msg_send![super(this), init] }
        }
    }

    /// Attaches the edge-back recognizer to `webview`, the app's WKWebView.
    /// Runs on the main thread, as UI effects do.
    pub(super) fn install_edge_back(webview: NonNull<c_void>) {
        if MainThreadMarker::new().is_none() {
            dioxus::logger::tracing::error!(
                "os back: the edge swipe installs on the main thread only; skipped"
            );
            return;
        }
        // SAFETY: the host hands over its live WKWebView, which the view
        // hierarchy keeps alive for the app's lifetime.
        let webview: &AnyObject = unsafe { webview.cast::<AnyObject>().as_ref() };
        let target = EdgeBackTarget::new();
        // SAFETY: `alloc` is valid on the UIScreenEdgePanGestureRecognizer class.
        let alloc: Allocated<AnyObject> =
            unsafe { msg_send![class!(UIScreenEdgePanGestureRecognizer), alloc] };
        // SAFETY: the designated initializer, with a live target and a selector it implements.
        let recognizer: Retained<AnyObject> =
            unsafe { msg_send![alloc, initWithTarget: &*target, action: sel!(handleEdgePan:)] };
        // SAFETY: `setEdges:` takes a UIRectEdge, an NSUInteger, on the main thread.
        let (): () = unsafe { msg_send![&*recognizer, setEdges: EDGE_LEFT] };
        // SAFETY: the target implements the delegate method this recognizer calls.
        let (): () = unsafe { msg_send![&*recognizer, setDelegate: &*target] };
        // SAFETY: `addGestureRecognizer:` on the live WKWebView, on the main thread.
        let (): () = unsafe { msg_send![webview, addGestureRecognizer: &*recognizer] };
        // A recognizer holds its target weakly, so the target is leaked to
        // outlive this call: one per app launch.
        std::mem::forget(target);
    }

    /// Turns the scroll view's safe-area inset adjustment off on `webview`,
    /// the app's WKWebView, and forces a layout pass so WebKit reads the
    /// insets back. Runs on the main thread, as UI effects do.
    pub(super) fn fill_window(webview: NonNull<c_void>) {
        if MainThreadMarker::new().is_none() {
            dioxus::logger::tracing::error!(
                "os back: the window layout runs on the main thread only; skipped"
            );
            return;
        }
        // SAFETY: the pointer is the app's live WKWebView, which outlives
        // every call made from the page it hosts.
        let webview: &AnyObject = unsafe { webview.cast::<AnyObject>().as_ref() };
        // SAFETY: `scrollView` is a WKWebView getter returning its UIScrollView.
        let scroll: Retained<AnyObject> = unsafe { msg_send![webview, scrollView] };
        // SAFETY: the setter takes the enum as an NSInteger, on the main thread.
        let (): () =
            unsafe { msg_send![&*scroll, setContentInsetAdjustmentBehavior: ADJUSTMENT_NEVER] };
        // SAFETY: `setNeedsLayout` takes nothing, on the main thread.
        let (): () = unsafe { msg_send![webview, setNeedsLayout] };
        // SAFETY: `layoutIfNeeded` takes nothing, on the main thread; WebKit
        // reads the insets back on this layout pass.
        let (): () = unsafe { msg_send![webview, layoutIfNeeded] };
    }
}

/// Leaving the app from a root screen: finish the Activity through JNI.
#[cfg(target_os = "android")]
mod android {
    use jni::JavaVM;
    use jni::objects::JObject;

    /// Finishes the current Activity, logging a failure.
    pub(super) fn finish_activity() {
        if let Err(e) = try_finish() {
            dioxus::logger::tracing::error!("os back: could not finish the activity: {e}");
        }
    }

    fn try_finish() -> jni::errors::Result<()> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context holds the process's JavaVM.
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        // SAFETY: ndk-context's context pointer is the Activity.
        let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
        env.call_method(&activity, "finish", "()V", &[])?;
        Ok(())
    }
}
