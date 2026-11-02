//! Links that open inside the app: [`open_url`] and [`use_in_app_links`].
//!
//! On iOS an `http` or `https` URL opens in an `SFSafariViewController` sheet
//! over the app, with a Done button that returns to the same screen. On
//! Android it opens as a Custom Tab, an `ACTION_VIEW` intent carrying the
//! Custom Tabs session extra; a browser without Custom Tabs support opens it
//! normally. Every other scheme (`mailto:`, `tel:`) goes to the OS:
//! `UIApplication.openURL` on iOS, a plain `ACTION_VIEW` on Android. Desktop
//! hands every URL to the default browser, and the web opens a new tab.
//!
//! Dioxus sends a click on an `<a href>` that no handler claimed to the
//! `webbrowser` crate, which leaves the app on mobile and drops `mailto:`
//! there. [`use_in_app_links`] routes those clicks through [`open_url`]
//! instead: links in `dangerous_inner_html`, [`DialogAction::Link`](crate::DialogAction::Link)
//! buttons and `target: "_blank"` anchors alike.

use std::error::Error;
use std::fmt;

use dioxus::prelude::*;

/// Why [`open_url`] could not open a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OpenUrlError {
    /// Called off the main thread, where `UIKit` must run.
    NotMainThread,
    /// The platform could not parse the URL.
    InvalidUrl {
        /// The URL as given.
        url: String,
    },
    /// The platform refused or failed, with its own message.
    Platform(String),
}

impl fmt::Display for OpenUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenUrlError::NotMainThread => write!(f, "links open only from the main thread"),
            OpenUrlError::InvalidUrl { url } => write!(f, "not a url: {url}"),
            OpenUrlError::Platform(message) => write!(f, "could not open the url: {message}"),
        }
    }
}

impl Error for OpenUrlError {}

/// Opens `url` in the in-app browser when it is a web page, or with the OS
/// otherwise. On iOS, a sheet that can't be presented falls back to the OS.
///
/// # Errors
///
/// [`OpenUrlError`] when the URL couldn't be opened at all.
pub fn open_url(url: &str) -> Result<(), OpenUrlError> {
    platform::open(url)
}

/// Routes every link click Dioxus leaves unclaimed through [`open_url`], for
/// the life of the calling component. Call it once, from the app root. A
/// failure is logged. Router links prevent the default in Rust first, so they
/// never reach it. On the web it does nothing: the browser opens links itself.
pub fn use_in_app_links() {
    #[cfg(not(target_arch = "wasm32"))]
    use_future(|| async {
        let mut eval = document::eval(ROUTE_LINKS_JS);
        while let Ok(href) = eval.recv::<String>().await {
            if let Err(e) = open_url(&href) {
                dioxus::logger::tracing::error!("in-app links: {e}");
            }
        }
    });
}

/// Replaces the interpreter's link hook with one that sends each `href` back
/// over the eval's channel. It matches the original: the nearest `<a>`, the
/// default prevented, an empty `href` ignored.
#[cfg(not(target_arch = "wasm32"))]
const ROUTE_LINKS_JS: &str = r#"
const interpreter = window.interpreter;
if (interpreter) {
    interpreter.handleClickNavigate = (event, target) => {
        const anchor = target.closest("a");
        if (!anchor) return;
        event.preventDefault();
        const href = anchor.getAttribute("href");
        if (href) dioxus.send(href);
    };
}
"#;

/// Whether `url` belongs in the in-app browser, which only loads web pages:
/// its scheme is `http` or `https`, in any case.
#[cfg_attr(
    not(any(target_os = "ios", target_os = "android", test)),
    expect(dead_code, reason = "only the mobile platforms have an in-app browser")
)]
fn opens_in_app(url: &str) -> bool {
    url.split_once(':').is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    })
}

#[cfg(target_os = "ios")]
mod platform {
    use std::ffi::c_void;

    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::{Encode, Encoding, MainThreadMarker, msg_send};
    use objc2_foundation::{NSDictionary, NSString, NSURL};

    use super::OpenUrlError;

    // Links SafariServices, which holds SFSafariViewController. The class is
    // looked up by name at runtime, so no symbols are declared.
    #[link(name = "SafariServices", kind = "framework")]
    unsafe extern "C" {}

    /// A null completion block, so the crate needs no block dependency.
    #[repr(transparent)]
    struct NullBlock(*const c_void);

    // SAFETY: `#[repr(transparent)]` over a pointer, the layout of an optional
    // block reference, which `completion:` and `completionHandler:` take.
    unsafe impl Encode for NullBlock {
        const ENCODING: Encoding = Encoding::Block;
    }

    pub(super) fn open(url: &str) -> Result<(), OpenUrlError> {
        // UIKit runs on the main thread, where a tap handler already is.
        MainThreadMarker::new().ok_or(OpenUrlError::NotMainThread)?;
        let app = app().ok_or_else(|| OpenUrlError::Platform("no UIApplication".into()))?;
        let url_object = NSURL::URLWithString(&NSString::from_str(url)).ok_or_else(|| {
            OpenUrlError::InvalidUrl {
                url: url.to_string(),
            }
        })?;

        if super::opens_in_app(url) {
            match present_safari(app, &url_object) {
                Ok(()) => return Ok(()),
                Err(e) => dioxus::logger::tracing::warn!(
                    "in-app browser unavailable, opening externally: {e}"
                ),
            }
        }
        open_external(app, &url_object);
        Ok(())
    }

    fn app() -> Option<&'static AnyObject> {
        let class = AnyClass::get(c"UIApplication")?;
        // SAFETY: `sharedApplication` takes no arguments and returns the
        // process's UIApplication, which lives as long as the process.
        let app: *mut AnyObject = unsafe { msg_send![class, sharedApplication] };
        // SAFETY: the pointer is either null or the live shared application.
        unsafe { app.as_ref() }
    }

    /// Presents an `SFSafariViewController` for `url` over the topmost view
    /// controller of the key window. `url` must be `http` or `https`; the
    /// controller raises an exception for any other scheme.
    fn present_safari(app: &AnyObject, url: &NSURL) -> Result<(), &'static str> {
        // SAFETY: `keyWindow` is a nullable UIWindow getter, read on the main
        // thread.
        let window: Option<Retained<AnyObject>> = unsafe { msg_send![app, keyWindow] };
        let window = window.ok_or("no key window")?;

        // SAFETY: `rootViewController` is a nullable UIViewController getter.
        let root: Option<Retained<AnyObject>> = unsafe { msg_send![&*window, rootViewController] };
        let mut presenter = root.ok_or("no root view controller")?;

        // A controller already presenting something can't present again, so
        // walk up to the one on top.
        loop {
            // SAFETY: `presentedViewController` is a nullable getter.
            let on_top: Option<Retained<AnyObject>> =
                unsafe { msg_send![&*presenter, presentedViewController] };
            match on_top {
                Some(next) => presenter = next,
                None => break,
            }
        }

        let class = AnyClass::get(c"SFSafariViewController").ok_or("no SFSafariViewController")?;
        // SAFETY: `alloc` on a class object returns an uninitialized instance.
        let allocated: Allocated<AnyObject> = unsafe { msg_send![class, alloc] };
        // SAFETY: `initWithURL:` takes an NSURL, and `opens_in_app` limited it
        // to the http(s) schemes the initializer accepts.
        let safari: Option<Retained<AnyObject>> = unsafe { msg_send![allocated, initWithURL: url] };
        let safari = safari.ok_or("SFSafariViewController init returned nil")?;

        // SAFETY: a null completion block is allowed, and this runs on the
        // main thread.
        unsafe {
            let () = msg_send![
                &*presenter,
                presentViewController: &*safari,
                animated: true,
                completion: NullBlock(std::ptr::null())
            ];
        }
        Ok(())
    }

    /// Hands `url` to the OS with `UIApplication.openURL`.
    fn open_external(app: &AnyObject, url: &NSURL) {
        let options: Retained<NSDictionary<AnyObject, AnyObject>> = NSDictionary::new();
        let handler = NullBlock(std::ptr::null());
        // SAFETY: an empty options dictionary and a null completion handler
        // are both allowed.
        unsafe {
            let () = msg_send![app, openURL: url, options: &*options, completionHandler: handler];
        }
    }
}

#[cfg(target_os = "android")]
mod platform {
    use jni::JNIEnv;
    use jni::objects::JObject;

    use super::OpenUrlError;

    /// Local references `open` creates: the URL string, the `Uri`, the action
    /// string, the `Intent`, and for web pages the `Bundle`, its key and the
    /// `Intent` that `putExtras` returns, plus the `Activity` class and the
    /// `Intent` that `addFlags` returns. A hint to the VM, not a cap.
    const LOCAL_FRAME_CAPACITY: i32 = 14;

    /// Intent extra that marks an `ACTION_VIEW` as a Custom Tabs request. The
    /// value is a `Bundle` holding a session binder, which may be null.
    const CUSTOM_TABS_SESSION: &str = "android.support.customtabs.extra.SESSION";

    /// `Intent.FLAG_ACTIVITY_NEW_TASK`, which `startActivity` requires when the
    /// context is not an `Activity`.
    const FLAG_ACTIVITY_NEW_TASK: i32 = 0x1000_0000;

    /// Fires the intent from its own local reference frame, so the references
    /// it creates are freed on return even on an already attached thread.
    pub(super) fn open(url: &str) -> Result<(), OpenUrlError> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Context jobject
        // for the running app.
        let vm =
            unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| platform_error(&e))?;
        let mut env = vm.attach_current_thread().map_err(|e| platform_error(&e))?;
        // SAFETY: the same ndk-context guarantee covers the Context jobject,
        // which stays alive for the running app. It may be the Application
        // rather than an Activity.
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };

        let result = fire(&mut env, &context, url);
        // A failed call leaves its Java exception pending, and the next JNI
        // call anywhere in the process aborts on it.
        if result.is_err()
            && let Err(e) = env.exception_clear()
        {
            dioxus::logger::tracing::error!("could not clear a java exception: {e}");
        }
        result.map_err(|e| platform_error(&e))
    }

    fn platform_error(e: &jni::errors::Error) -> OpenUrlError {
        OpenUrlError::Platform(e.to_string())
    }

    fn fire(env: &mut JNIEnv<'_>, context: &JObject<'_>, url: &str) -> jni::errors::Result<()> {
        env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| -> jni::errors::Result<()> {
            // Uri uri = Uri.parse(url);
            let url_jstring = env.new_string(url)?;
            let uri = env
                .call_static_method(
                    "android/net/Uri",
                    "parse",
                    "(Ljava/lang/String;)Landroid/net/Uri;",
                    &[(&url_jstring).into()],
                )?
                .l()?;

            // Intent intent = new Intent(Intent.ACTION_VIEW, uri);
            let action_view = env
                .get_static_field(
                    "android/content/Intent",
                    "ACTION_VIEW",
                    "Ljava/lang/String;",
                )?
                .l()?;
            let intent = env.new_object(
                "android/content/Intent",
                "(Ljava/lang/String;Landroid/net/Uri;)V",
                &[(&action_view).into(), (&uri).into()],
            )?;

            if super::opens_in_app(url)
                && let Err(e) = add_custom_tabs_session(env, &intent)
            {
                dioxus::logger::tracing::warn!(
                    "custom tabs extra failed, opening a plain intent: {e}"
                );
                // A failed JNI call leaves its Java exception pending, and
                // the next call would fail on it.
                env.exception_clear()?;
            }

            // An Application context can start an activity only into a new task.
            if !env.is_instance_of(context, "android/app/Activity")? {
                env.call_method(
                    &intent,
                    "addFlags",
                    "(I)Landroid/content/Intent;",
                    &[FLAG_ACTIVITY_NEW_TASK.into()],
                )?;
            }

            // context.startActivity(intent);
            env.call_method(
                context,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[(&intent).into()],
            )?;
            Ok(())
        })
    }

    /// Adds the Custom Tabs session extra to `intent`. `putExtras` is the last
    /// call, so on error the intent is left as it was.
    fn add_custom_tabs_session(
        env: &mut JNIEnv<'_>,
        intent: &JObject<'_>,
    ) -> jni::errors::Result<()> {
        // Bundle extras = new Bundle();
        let extras = env.new_object("android/os/Bundle", "()V", &[])?;

        // extras.putBinder(CUSTOM_TABS_SESSION, null);
        let key = env.new_string(CUSTOM_TABS_SESSION)?;
        env.call_method(
            &extras,
            "putBinder",
            "(Ljava/lang/String;Landroid/os/IBinder;)V",
            &[(&key).into(), (&JObject::null()).into()],
        )?;

        // intent.putExtras(extras);
        env.call_method(
            intent,
            "putExtras",
            "(Landroid/os/Bundle;)Landroid/content/Intent;",
            &[(&extras).into()],
        )?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use super::OpenUrlError;

    pub(super) fn open(url: &str) -> Result<(), OpenUrlError> {
        let window = web_sys::window().ok_or_else(|| OpenUrlError::Platform("no window".into()))?;
        window
            .open_with_url_and_target(url, "_blank")
            .map(|_| ())
            .map_err(|e| OpenUrlError::Platform(format!("{e:?}")))
    }
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
mod platform {
    use super::OpenUrlError;

    pub(super) fn open(url: &str) -> Result<(), OpenUrlError> {
        webbrowser::open(url).map_err(|e| OpenUrlError::Platform(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_schemes_open_in_app() {
        for url in [
            "https://zwipe.net/guides/filtering",
            "http://example.com",
            "HTTPS://ZWIPE.NET",
            "Http://example.com/path?q=1",
        ] {
            assert!(opens_in_app(url), "input {url:?}");
        }
    }

    #[test]
    fn other_schemes_go_to_the_os() {
        for url in [
            "mailto:support@zwipe.net",
            "tel:+15555550100",
            "sms:+15555550100",
            "httpx://example.com",
            "ftp://example.com",
            "https",
            "",
            "zwipe.net/https:",
        ] {
            assert!(!opens_in_app(url), "input {url:?}");
        }
    }

    #[test]
    fn errors_read_lowercase_without_a_period() {
        for error in [
            OpenUrlError::NotMainThread,
            OpenUrlError::InvalidUrl { url: "x".into() },
            OpenUrlError::Platform("no window".into()),
        ] {
            let message = error.to_string();
            assert!(!message.ends_with('.'), "message {message:?}");
            assert_eq!(message, message.to_lowercase(), "message {message:?}");
        }
    }
}
