//! One radius: every container (a panel, a sheet, a dialog, a toast, the dock) rounds at the panel
//! radius with the panel padding inside, and every chip and button at the chip radius. A box that
//! sits in a container's corner, nearer than the padding (a dock button, the toast's close), takes
//! the container's radius less the border and its inset instead, so the two curves match. Clickable
//! and read-only chips share the corner and differ by border, hover, press and fill. No rule sets a radius of its own, except the
//! shapes that are no container, chip or button: dots, bars, handles, focus rings and round
//! buttons.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: walks CSS text by byte offsets it finds itself"
)]

/// Every sheet orangui ships.
const SHEETS: [(&str, &str); 8] = [
    ("app.css", include_str!("../assets/app.css")),
    ("components.css", include_str!("../assets/components.css")),
    ("dialog.css", include_str!("../assets/dialog.css")),
    ("nav_fold.css", include_str!("../assets/nav_fold.css")),
    ("screen.css", include_str!("../assets/screen.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("themes.css", include_str!("../assets/themes.css")),
    ("toast.css", include_str!("../assets/toast.css")),
];

const COMPONENTS: &str = include_str!("../assets/components.css");
const TOAST: &str = include_str!("../assets/toast.css");

/// The radii a rule may set: the panel's, a chip's, a bottom sheet's top corners, and the dock's.
/// A box in a container's corner may also take [`nested`], the container's radius less the border
/// and its inset.
const ALLOWED: [&str; 4] = [
    "var(--panel-radius)",
    "var(--chip-radius)",
    "var(--panel-radius) var(--panel-radius) 0 0",
    "var(--dock-radius)",
];

/// Whether `radius` is the nested formula: a container's radius token less the 1px border and an
/// inset token, floored so it never goes square.
fn nested(radius: &str) -> bool {
    radius
        .strip_prefix("max(0.15rem, calc(var(--")
        .and_then(|rest| rest.split_once(") - 1px - var(--"))
        .is_some_and(|(outer, inset)| {
            ["panel-radius", "dock-radius"].contains(&outer) && inset.ends_with(")))")
        })
}

/// Rules whose shape is no container, chip or button, each with its own radius.
const OUT_OF_SCOPE: [&str; 5] = [
    // The hamburger's bars.
    ".nav-toggle-bar",
    // A theme's color dot.
    ".theme-dot",
    // A bottom sheet's drag handle, a pill.
    ".sheet-handle",
    // A bare link's focus ring, which follows its text.
    ":where(a):not([class]):focus-visible",
    // The gallery's round arrows.
    ".gallery-nav",
];

/// `css` without its comments, the same length so offsets still line up.
fn without_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("*/")
            .map_or(rest.len(), |e| start + e + 2);
        out.extend(
            rest[start..end]
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' }),
        );
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Every style rule in `css` as its selector and its declarations, at-rules looked inside.
fn rules(css: &str) -> Vec<(String, String)> {
    let css = without_comments(css);
    let mut out = Vec::new();
    let mut heads: Vec<String> = Vec::new();
    let mut start = 0;
    for (at, c) in css.char_indices() {
        match c {
            '{' => {
                heads.push(css[start..at].trim().to_owned());
                start = at + 1;
            }
            '}' => {
                if let Some(head) = heads.pop()
                    && !head.starts_with('@')
                {
                    out.push((
                        head.split_whitespace().collect::<Vec<_>>().join(" "),
                        css[start..at].to_owned(),
                    ));
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    out
}

/// The value `body` gives `property`, if it sets it.
fn value(body: &str, property: &str) -> Option<String> {
    body.split(';').find_map(|decl| {
        let (name, value) = decl.split_once(':')?;
        (name.trim() == property).then(|| value.split_whitespace().collect::<Vec<_>>().join(" "))
    })
}

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule(css: &str, selector: &str) -> String {
    rules(css)
        .into_iter()
        .find(|(head, _)| head == selector)
        .map_or_else(|| panic!("a rule for {selector}"), |(_, body)| body)
}

#[test]
fn the_panel_s_radius_and_padding_and_the_chip_s_radius_are_tokens() {
    let root = rule(COMPONENTS, ":root");
    assert_eq!(value(&root, "--panel-radius").as_deref(), Some("0.75rem"));
    assert_eq!(
        value(&root, "--panel-pad").as_deref(),
        Some("var(--space-4)")
    );
    assert_eq!(value(&root, "--chip-radius").as_deref(), Some("0.375rem"));
}

#[test]
fn no_rule_sets_a_radius_of_its_own() {
    let mut own = Vec::new();
    for (file, css) in SHEETS {
        for (selector, body) in rules(css) {
            let Some(radius) = value(&body, "border-radius") else {
                continue;
            };
            if !ALLOWED.contains(&radius.as_str())
                && !nested(&radius)
                && !OUT_OF_SCOPE.contains(&selector.as_str())
            {
                own.push(format!("{file}: {selector} {{ border-radius: {radius} }}"));
            }
        }
    }
    assert!(
        own.is_empty(),
        "radii outside the rule:\n{}",
        own.join("\n")
    );
}

#[test]
fn the_old_corner_tokens_are_gone() {
    for (file, css) in SHEETS {
        for token in ["--radius-readonly", "--radius-control", "--corner-radius"] {
            assert!(!css.contains(token), "{file} still names {token}");
        }
    }
}

#[test]
fn read_only_and_clickable_chips_share_the_chip_radius() {
    for selector in [
        ".chip",
        ".chip-unselected",
        ":where(.tag)",
        ":where(.stat-chip)",
        ".status-tag",
        ".btn",
        ".btn-xs",
        ".btn-action",
    ] {
        assert_eq!(
            value(&rule(COMPONENTS, selector), "border-radius").as_deref(),
            Some("var(--chip-radius)"),
            "{selector}"
        );
    }
}

#[test]
fn containers_take_the_panel_radius_and_padding() {
    let panel = rule(COMPONENTS, ".panel-card");
    assert_eq!(
        value(&panel, "border-radius").as_deref(),
        Some("var(--panel-radius)")
    );
    assert_eq!(
        value(&panel, "padding").as_deref(),
        Some("var(--panel-pad)")
    );
    let toast = rule(TOAST, ".toast-container [role=\"alertdialog\"]");
    assert_eq!(
        value(&toast, "border-radius").as_deref(),
        Some("var(--panel-radius)")
    );
    assert_eq!(
        value(&toast, "padding").as_deref(),
        Some("var(--panel-pad)")
    );
}

#[test]
fn the_dock_is_a_container_and_its_buttons_nest_in_its_corners() {
    let dock = rule(COMPONENTS, ":where(.action-dock)");
    assert_eq!(
        value(&dock, "--dock-radius").as_deref(),
        Some("var(--panel-radius)")
    );
    let button = rule(COMPONENTS, ":where(.action-dock-btn)");
    assert_eq!(
        value(&button, "border-radius").as_deref(),
        Some("max(0.15rem, calc(var(--dock-radius) - 1px - var(--dock-inset)))"),
        "a button sits the inset in from the dock's corner"
    );
}

#[test]
fn a_menu_s_rows_run_edge_to_edge_and_its_corners_clip_their_hover() {
    let menu = rule(COMPONENTS, ".nav-dropdown-content");
    assert_eq!(
        value(&menu, "padding").as_deref(),
        Some("var(--space-2) 0"),
        "a row's own padding is the menu's inset, so its hover fill meets the edges"
    );
    assert_eq!(
        value(&menu, "overflow-y").as_deref(),
        Some("auto"),
        "the menu clips its rows to its rounded corners"
    );
}
