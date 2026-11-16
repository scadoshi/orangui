//! Shadow levels: three elevations shared by every theme, each larger and softer than the one
//! below, not just darker, and all cast from straight above. `sm` is for what sits on the page
//! (chips, buttons, inputs, cards, bars), `md` for what floats over it (the dock, toasts,
//! banners, dropdowns) and `lg` for what covers it (sheets, drawers, modals and dialogs). A dark
//! theme keeps them faint, since a black shadow on a near-black page reads as a smudge.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: slices the stylesheets after matches it found and steps past them"
)]

const THEMES: &str = include_str!("../assets/themes.css");
const COMPONENTS: &str = include_str!("../assets/components.css");
const TOAST: &str = include_str!("../assets/toast.css");
const DIALOG: &str = include_str!("../assets/dialog.css");
const APP: &str = include_str!("../assets/app.css");
const SITE: &str = include_str!("../assets/site.css");
const NAV_FOLD: &str = include_str!("../assets/nav_fold.css");

/// The selector of the block every dark theme, and the default, takes its derived tokens from.
const DARK_BLOCK: &str =
    ":root,\n[class*=\"theme-\"][class*=\"-dark\"],\n[class*=\"theme-\"][class*=\"-light\"] {";
/// The selector of the block a light theme overrides them in, after its comment (the dark block's
/// selector list ends on the same line).
const LIGHT_BLOCK: &str = "*/\n[class*=\"theme-\"][class*=\"-light\"] {\n    --bg-sink";

/// The body of the first rule in `sheet` whose selector line, indented or not, is `selector`.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let open = format!("{selector} {{");
    let mut at = 0;
    for line in sheet.split_inclusive('\n') {
        if line.trim() == open {
            let rest = &sheet[at + line.len()..];
            return rest.split_once('}').map_or("", |(body, _)| body);
        }
        at += line.len();
    }
    ""
}

/// The body of the block whose text starts with `opening`, which includes its `{`.
fn block(opening: &str) -> &'static str {
    let brace = opening.find('{').map_or(0, |at| at + 1);
    THEMES
        .find(opening)
        .and_then(|at| THEMES[at + brace..].split_once('}'))
        .map_or("", |(body, _)| body)
}

/// The value `token` is set to in `body`.
fn token<'body>(body: &'body str, token: &str) -> &'body str {
    let name = format!("{token}:");
    body.split(';')
        .find_map(|declaration| declaration.trim().strip_prefix(name.as_str()))
        .map_or("", str::trim)
}

/// One layer of a shadow: its offsets and blur in px, and its alpha.
#[derive(Debug)]
struct Layer {
    x: f64,
    y: f64,
    blur: f64,
    alpha: f64,
}

/// `value`'s layers, each written `0 2px 4px rgb(0 0 0 / 0.2)`.
fn layers(value: &str) -> Vec<Layer> {
    value
        .split("),")
        .map(|layer| {
            let (lengths, color) = layer.split_once("rgb(").unwrap_or_default();
            let px: Vec<f64> = lengths
                .split_whitespace()
                .map(|length| length.trim_end_matches("px").parse().unwrap_or(f64::NAN))
                .collect();
            let alpha = color.split_once('/').map_or(f64::NAN, |(_, a)| {
                a.trim_end_matches(')').trim().parse().unwrap_or(f64::NAN)
            });
            assert_eq!(px.len(), 3, "x, y and blur: {layer:?}");
            Layer {
                x: px[0],
                y: px[1],
                blur: px[2],
                alpha,
            }
        })
        .collect()
}

/// The three levels, small to large, as `body` sets them.
fn scale(body: &str) -> [Vec<Layer>; 3] {
    ["--shadow-sm", "--shadow-md", "--shadow-lg"].map(|name| {
        let value = token(body, name);
        assert!(!value.is_empty(), "{name} is set in {body:?}");
        layers(value)
    })
}

/// The largest of `pick` over `layers`.
fn most(layers: &[Layer], pick: fn(&Layer) -> f64) -> f64 {
    layers.iter().map(pick).fold(0.0, f64::max)
}

#[test]
fn every_theme_takes_one_shared_scale() {
    for name in ["--shadow-sm:", "--shadow-md:", "--shadow-lg:"] {
        assert_eq!(
            THEMES.matches(name).count(),
            2,
            "{name} is set once for dark themes and once for light ones, never in a theme's own block"
        );
    }
}

#[test]
fn each_level_is_larger_and_softer_from_one_light() {
    for (mode, body) in [("dark", block(DARK_BLOCK)), ("light", block(LIGHT_BLOCK))] {
        let levels = scale(body);
        for pair in levels.windows(2) {
            let (lower, upper) = (&pair[0], &pair[1]);
            assert!(
                upper.len() > lower.len(),
                "{mode}: more layers a level up: {levels:?}"
            );
            assert!(
                most(upper, |l| l.y) > most(lower, |l| l.y),
                "{mode}: a longer offset a level up: {levels:?}"
            );
            assert!(
                most(upper, |l| l.blur) >= 2.0 * most(lower, |l| l.blur),
                "{mode}: at least twice the blur a level up: {levels:?}"
            );
        }
        for layer in levels.iter().flatten() {
            assert_eq!(layer.x, 0.0, "{mode}: cast from straight above: {layer:?}");
            assert!(
                layer.blur >= layer.y,
                "{mode}: soft, its blur at least its offset: {layer:?}"
            );
        }
    }
}

#[test]
fn dark_shadows_stay_faint_and_light_ones_fainter() {
    let dark = scale(block(DARK_BLOCK));
    let light = scale(block(LIGHT_BLOCK));
    for layer in dark.iter().flatten() {
        assert!(
            layer.alpha <= 0.3,
            "dark: faint on a near-black page: {layer:?}"
        );
    }
    for (d, l) in dark.iter().zip(&light) {
        assert!(
            most(l, |x| x.alpha) < most(d, |x| x.alpha),
            "a light theme needs less to show: {l:?} against {d:?}"
        );
    }
}

#[test]
fn each_surface_takes_its_level() {
    let surfaces: [(&str, &'static str, &str, &str); 22] = [
        ("components", COMPONENTS, ".chip", "sm"),
        ("components", COMPONENTS, ".btn", "sm"),
        ("app", APP, ".input", "sm"),
        ("app", APP, ".settings-list", "sm"),
        ("components", COMPONENTS, ".panel-card", "sm"),
        ("components", COMPONENTS, ".release-card", "sm"),
        ("components", COMPONENTS, ".stats-strip .stat", "sm"),
        ("components", COMPONENTS, ".nav-wrapper", "sm"),
        ("site", SITE, ".gallery-nav", "sm"),
        ("site", SITE, ".site-footer", "sm"),
        ("components", COMPONENTS, ".action-dock", "md"),
        (
            "toast",
            TOAST,
            ".toast-container [role=\"alertdialog\"]",
            "md",
        ),
        ("components", COMPONENTS, ".banner", "md"),
        ("components", COMPONENTS, ".nav-dropdown-content", "md"),
        ("components", COMPONENTS, ".release-card:hover", "md"),
        ("components", COMPONENTS, ".bottom-sheet", "lg"),
        (
            "components",
            COMPONENTS,
            ".sheet-form-drawer > .bottom-sheet",
            "lg",
        ),
        (
            "components",
            COMPONENTS,
            ".nav-wrapper:has(.nav-panel-open)",
            "lg",
        ),
        ("dialog", DIALOG, ".alert-dialog", "lg"),
        ("components", COMPONENTS, ".panel-action", "sm"),
        ("components", COMPONENTS, ".mode-toggle", "sm"),
        (
            "nav_fold",
            NAV_FOLD,
            ".nav-panel .nav-links a.nav-dropdown-item:visited",
            "sm",
        ),
    ];
    for (file, sheet, selector, level) in surfaces {
        let body = rule(sheet, selector);
        assert!(
            body.contains(&format!("box-shadow: var(--shadow-{level});")),
            "{file}: {selector} sits at {level}: {body:?}"
        );
    }
}

#[test]
fn no_surface_stacks_a_level_on_itself() {
    for sheet in [COMPONENTS, TOAST, DIALOG, APP, SITE, NAV_FOLD] {
        for level in ["sm", "md", "lg"] {
            let doubled = format!("var(--shadow-{level}), var(--shadow-{level})");
            assert!(
                !sheet.contains(&doubled),
                "the next level up, not {doubled}"
            );
        }
    }
}
