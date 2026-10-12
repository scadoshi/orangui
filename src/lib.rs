//! orangui: a Dioxus UI kit, so every app and site that uses it shares one look.
//!
//! The components depend only on base `dioxus` (no platform features) so any
//! Dioxus target can consume them; a host enables its own web, mobile or
//! desktop feature and cargo unifies. Styling ships alongside as string
//! constants to inline through `document::Style`: [`THEMES_CSS`] (the 31
//! palettes, each dark and light) and [`COMPONENTS_CSS`] (the components'
//! rules), then one opt-in sheet per kind of host.
//!
//! Three kits sit on the shared base. The app kit ([`BottomSheet`],
//! [`PageHeader`], [`ThemeSheet`], the hint pieces, the overlay back stack
//! and, behind the `toast` feature, `ToastStack`) is for apps and styled by
//! [`APP_CSS`]. The site kit ([`use_persisted_theme`], [`StatsStrip`],
//! [`GalleryFrame`], [`REVEAL_JS`], [`NAV_GLIDE_JS`], [`SCROLL_FADE_JS`]) is
//! for websites, with the gallery base in [`SITE_CSS`]. [`HeatGrid`] draws a
//! year of days for any surface that shows one.
//!
//! The theme model ([`ThemeConfig`], [`ALLOWED_THEMES`], [`COLORBLIND_THEMES`],
//! [`display_theme_name`]) lives here beside the palettes it names. A host that
//! stores a user's theme elsewhere keeps its own list and checks it against
//! this one in a test.
//!
//! **CSS cascade order matters:** load themes first, then components, then the
//! opt-in sheet, then the host's own stylesheet (`THEMES_CSS`, then
//! `COMPONENTS_CSS`, then `APP_CSS` or `SITE_CSS`, then host CSS) so component
//! rules resolve theme variables and host rules can override shared defaults at
//! equal specificity.

mod action_bar;
mod banner;
mod bottom_sheet;
mod button;
mod chart_math;
mod chip;
mod diagram;
mod entrance;
mod gallery;
mod heat_grid;
mod hint;
mod nav_bar;
mod nav_dropdown;
mod overlay_stack;
mod page_header;
mod page_meta;
mod palette_guard;
mod panel;
mod password_input;
mod stats_strip;
mod theme;
mod theme_picker;
mod theme_sheet;
mod theme_store;
mod theme_wipe;
#[cfg(feature = "toast")]
mod toast_stack;
mod toast_timing;

pub use action_bar::ActionBar;
pub use banner::{Banner, BannerStatus};
pub use bottom_sheet::BottomSheet;
pub use button::{Button, ButtonVariant};
pub use chart_math::{area, curve, peak_indices, tip_anchor};
pub use chip::Chip;
pub use diagram::{DIAGRAM_NODE_HEIGHT, DiagramArrow, DiagramDefs, DiagramNode, DiagramTone};
pub use entrance::{CountUp, Decode, Figure, Replay, with_separators};
pub use gallery::{GalleryFooter, GalleryFrame};
pub use heat_grid::{
    HEAT_CELL, HEAT_GAP, HEAT_ROWS, HEAT_STEP, HeatCell, HeatGrid, HeatHit, heat_span,
    use_scroll_to_end,
};
pub use hint::{HintBullet, HintBullets, HintKey, HintLine, InfoButton};
pub use nav_bar::{BRAND_RESET_JS, NavBar};
pub use nav_dropdown::NavDropdown;
pub use overlay_stack::{
    OverlayBackStack, use_overlay_back, use_overlay_back_action, use_overlay_back_stack,
};
pub use page_header::PageHeader;
pub use page_meta::{PageMeta, SiteMeta};
pub use palette_guard::{StatusUse, status_uses};
pub use panel::Panel;
pub use password_input::PasswordInput;
pub use stats_strip::StatsStrip;
pub use theme::{ALLOWED_THEMES, COLORBLIND_THEMES, ThemeConfig, display_theme_name};
pub use theme_picker::{DarkModeButton, ModeIcon, ThemePicker};
pub use theme_sheet::ThemeSheet;
pub use theme_store::use_persisted_theme;
pub use theme_wipe::{ThemeFollow, use_theme_follow, use_theme_wipe};
#[cfg(feature = "toast")]
pub use toast_stack::ToastStack;
pub use toast_timing::{TOAST_LONG, TOAST_NORMAL, TOAST_QUICK};

/// The shared component rules.
pub const COMPONENTS_CSS: &str = include_str!("../assets/components.css");
/// The 31 theme palettes, each dark and light.
pub const THEMES_CSS: &str = include_str!("../assets/themes.css");
/// The sites' scroll reveal: elements marked `data-reveal` (every [`Panel`])
/// that start below the fold fade up as they scroll in. Inline it with
/// `document::Script`; the host's stylesheet styles `.reveal-pending` and
/// `.reveal-in`.
pub const REVEAL_JS: &str = include_str!("../assets/reveal.js");
/// The sites' nav glide: the children of every element marked
/// `data-nav-glide` (the [`NavBar`] links and the [`ThemePicker`]) slide to
/// their new places when a neighbor changes width. Inline it with
/// `document::Script`.
pub const NAV_GLIDE_JS: &str = include_str!("../assets/nav-glide.js");
/// The sites' scroll fade for browsers without scroll timelines (Firefox,
/// older Safari): sets the edge fades on every `.scroll-fade-x`,
/// `.diagram-scroll` and `.scroll-fade-y` (the [`NavBar`] link list) from the
/// scroll position, which [`COMPONENTS_CSS`] animates where it can, and does
/// nothing elsewhere. Inline it with `document::Script`.
pub const SCROLL_FADE_JS: &str = include_str!("../assets/scroll-fade.js");
/// The sites' shared base for the gallery ([`GalleryFrame`],
/// [`GalleryFooter`]), for the websites only. Load it after
/// [`COMPONENTS_CSS`] and before the site's own stylesheet.
pub const SITE_CSS: &str = include_str!("../assets/site.css");
/// The app shell's rules (screen, page header, bottom sheet, settings rows,
/// hints), for the apps only. Load it after [`COMPONENTS_CSS`] and before the
/// app's own stylesheet. Sites leave it out: they style these class names
/// their own way.
pub const APP_CSS: &str = include_str!("../assets/app.css");
/// The toast stack's rules, for [`ToastStack`]. Load it after the app's own
/// stylesheet.
#[cfg(feature = "toast")]
pub const TOAST_CSS: &str = include_str!("../assets/toast.css");
