//! The launch guard for an app in a native WebView. Dioxus adds every head
//! element (each `document::Style`, `document::Link` and `document::Script`)
//! after the first render, so without it the WebView can paint the first
//! screen as raw HTML before its styles land.
//!
//! [`BOOT_HEAD`] goes into the static page through the WebView config, with
//! [`BOOT_BG`] as the WebView's own color; it paints the page dark and hides
//! `#main`, the root Dioxus renders into. [`BootReveal`] goes last among the
//! host's head elements and shows `#main` once every stylesheet has loaded:
//!
//! ```ignore
//! dioxus::LaunchBuilder::new()
//!     .with_cfg(
//!         Config::new()
//!             .with_background_color(orangui::BOOT_BG)
//!             .with_custom_head(orangui::BOOT_HEAD.to_string()),
//!     )
//!     .launch(App);
//! ```

use dioxus::prelude::*;

/// The WebView's color before any HTML paints: gruvbox dark's
/// `--bg-primary`, the default theme.
pub const BOOT_BG: (u8, u8, u8, u8) = (0x28, 0x28, 0x28, 0xff);

/// The static head: the page in [`BOOT_BG`], and `#main` hidden until
/// [`BootReveal`] fades it in.
pub const BOOT_HEAD: &str = "<style>html,body{background-color:#282828;}#main{opacity:0;transition:opacity 0.15s ease-in;}</style>";

/// Shows `#main` once the linked stylesheets have loaded.
const BOOT_REVEAL_JS: &str = include_str!("../assets/boot-reveal.js");

/// Shows the app once its styles are in. Mount it after the host's last
/// `document::Style`, `document::Link` and `document::Stylesheet`.
#[component]
pub fn BootReveal() -> Element {
    rsx! {
        document::Script { {BOOT_REVEAL_JS} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_boot_color_is_the_default_themes_background() {
        let block = crate::THEMES_CSS
            .split(".theme-gruvbox-dark {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .unwrap();
        let bg = block
            .lines()
            .find_map(|line| line.trim().strip_prefix("--bg-primary:"))
            .unwrap()
            .trim()
            .trim_end_matches(';');
        let (r, g, b, a) = BOOT_BG;
        assert_eq!(bg, format!("#{r:02x}{g:02x}{b:02x}"));
        assert_eq!(a, 0xff);
        assert!(BOOT_HEAD.contains(&format!("background-color:{bg};")));
    }

    #[test]
    fn the_head_hides_main_and_the_reveal_shows_it() {
        assert!(BOOT_HEAD.contains("#main{opacity:0;"));
        assert!(BOOT_REVEAL_JS.contains(r#"getElementById("main")"#));
        assert!(BOOT_REVEAL_JS.contains(r#"style.opacity = "1""#));
    }
}
