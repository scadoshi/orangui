//! One radius: every container (a panel, a sheet, a dialog, a toast) rounds at the panel radius
//! with the panel padding inside, and every chip and button rounds at the panel radius minus that
//! padding, so a chip in a container's corner nests in it. Clickable and read-only chips share the
//! corner and differ by border, hover, press and fill. No rule sets a radius of its own, except the
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

/// The radii a rule may set: the panel's, a chip's, a bottom sheet's top corners, the dock's
/// (built back up from a chip's, so its buttons land on the chip radius), and the toast's close
/// button, which sits a smaller inset in from the toast's edge and nests at that inset.
const ALLOWED: [&str; 5] = [
    "var(--panel-radius)",
    "var(--chip-radius)",
    "var(--panel-radius) var(--panel-radius) 0 0",
    "var(--dock-radius)",
    "max(0.15rem, calc(var(--panel-radius) - 1px - var(--space-2)))",
];

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
    assert_eq!(value(&root, "--panel-radius").as_deref(), Some("1.5rem"));
    assert_eq!(
        value(&root, "--panel-pad").as_deref(),
        Some("var(--space-4)")
    );
    assert_eq!(
        value(&root, "--chip-radius").as_deref(),
        Some("calc(var(--panel-radius) - var(--panel-pad))"),
        "a chip's corner nests in a panel's"
    );
}

#[test]
fn no_rule_sets_a_radius_of_its_own() {
    let mut own = Vec::new();
    for (file, css) in SHEETS {
        for (selector, body) in rules(css) {
            let Some(radius) = value(&body, "border-radius") else {
                continue;
            };
            if !ALLOWED.contains(&radius.as_str()) && !OUT_OF_SCOPE.contains(&selector.as_str()) {
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
    assert!(
        rules(COMPONENTS).iter().all(|(head, body)| head
            != ":where(a.tag, button.tag, a.stat-chip, button.stat-chip)"
            || value(body, "border-radius").is_none()),
        "a linked chip keeps the chip's corner"
    );
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
fn the_dock_s_buttons_land_on_the_chip_radius() {
    let dock = rule(COMPONENTS, ":where(.action-dock)");
    assert_eq!(
        value(&dock, "--dock-radius").as_deref(),
        Some("calc(var(--chip-radius) + 1px + var(--dock-inset))"),
        "the dock's corner is a button's plus the border and the inset"
    );
    let button = rule(COMPONENTS, ":where(.action-dock-btn)");
    assert_eq!(
        value(&button, "border-radius").as_deref(),
        Some("var(--chip-radius)")
    );
}
