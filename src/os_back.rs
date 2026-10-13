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
//! On every other target the hook does nothing, so a browser keeps its own
//! back button.

use std::{ffi::c_void, ptr::NonNull};

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
            if let Some(webview) = ios_webview() {
                ios::install_edge_back(webview, tx);
            }
        });
    }

    #[cfg(target_os = "android")]
    {
        let _ = ios_webview;
        let mut overlays: OverlayBackStack = use_context();
        use_future(move || async move {
            let mut eval = document::eval(&format!(
                "window.addEventListener('{android_event}', () => dioxus.send(1));"
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

/// The left-edge swipe: a `UIScreenEdgePanGestureRecognizer` on the WKWebView
/// whose target sends on a channel.
#[cfg(target_os = "ios")]
mod ios {
    use std::{ffi::c_void, ptr::NonNull};

    use objc2::{
        AnyThread, DefinedClass, MainThreadMarker, class, define_class, msg_send,
        rc::{Allocated, Retained},
        runtime::{AnyObject, Bool, NSObject},
        sel,
    };
    use tokio::sync::mpsc::UnboundedSender;

    /// `UIGestureRecognizerStateBegan`: the target fires once, as the pan
    /// starts.
    const STATE_BEGAN: isize = 1;
    /// `UIRectEdgeLeft` (1 << 1): only pans from the left screen edge.
    const EDGE_LEFT: usize = 2;

    /// The Rust state the Objective-C target carries.
    struct Ivars {
        tx: UnboundedSender<()>,
    }

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; no Drop impl.
        #[unsafe(super(NSObject))]
        #[name = "OranguiEdgeBackTarget"]
        #[ivars = Ivars]
        struct EdgeBackTarget;

        impl EdgeBackTarget {
            /// Target-action for the edge pan. `sender` is the recognizer.
            #[unsafe(method(handleEdgePan:))]
            fn handle_edge_pan(&self, sender: &AnyObject) {
                // SAFETY: `state` is a valid selector on UIGestureRecognizer.
                let state: isize = unsafe { msg_send![sender, state] };
                if state == STATE_BEGAN {
                    let _ = self.ivars().tx.send(());
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
        fn new(tx: UnboundedSender<()>) -> Retained<Self> {
            let this = Self::alloc().set_ivars(Ivars { tx });
            // SAFETY: the designated initializer of an NSObject subclass.
            unsafe { msg_send![super(this), init] }
        }
    }

    /// Attaches the edge-back recognizer to `webview`, the app's WKWebView.
    /// Runs on the main thread, as UI effects do.
    pub fn install_edge_back(webview: NonNull<c_void>, tx: UnboundedSender<()>) {
        if MainThreadMarker::new().is_none() {
            dioxus::logger::tracing::error!(
                "os back: the edge swipe installs on the main thread only; skipped"
            );
            return;
        }
        // SAFETY: the host hands over its live WKWebView, which the view
        // hierarchy keeps alive for the app's lifetime.
        let webview: &AnyObject = unsafe { webview.cast::<AnyObject>().as_ref() };
        let target = EdgeBackTarget::new(tx);
        // SAFETY: UIKit gesture setup with valid selectors and argument types,
        // on the main thread.
        unsafe {
            let alloc: Allocated<AnyObject> =
                msg_send![class!(UIScreenEdgePanGestureRecognizer), alloc];
            let recognizer: Retained<AnyObject> =
                msg_send![alloc, initWithTarget: &*target, action: sel!(handleEdgePan:)];
            let _: () = msg_send![&*recognizer, setEdges: EDGE_LEFT];
            let _: () = msg_send![&*recognizer, setDelegate: &*target];
            let _: () = msg_send![webview, addGestureRecognizer: &*recognizer];
        }
        // A recognizer holds its target weakly, so the target is leaked to
        // outlive this call: one per app launch.
        std::mem::forget(target);
    }
}

/// Leaving the app from a root screen: finish the Activity through JNI.
#[cfg(target_os = "android")]
mod android {
    use jni::{JavaVM, objects::JObject};

    /// Finishes the current Activity, logging a failure.
    pub fn finish_activity() {
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
