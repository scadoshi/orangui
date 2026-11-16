//! Nav menu motion: a menu comes out of what opened it quickly and leaves faster, moving only
//! opacity and transform so nothing around it reflows. The desktop dropdown fades and scales in
//! from its trigger (150ms ease-out, out in 100ms ease-in); the folded phone menu lays over the
//! page and slides and fades the same way (180ms in, 120ms out). Under reduced motion both keep
//! only a quick fade.

const COMPONENTS: &str = include_str!("../assets/components.css");
const NAV_FOLD: &str = include_str!("../assets/nav_fold.css");

/// Every rule in `sheet` as (the selector, its body, the at-rules around it), comments removed.
fn rules(sheet: &str) -> Vec<(String, String, Vec<String>)> {
    let mut clean = String::new();
    let mut rest = sheet;
    while let Some(start) = rest.find("/*") {
        let (before, after) = rest.split_at(start);
        clean.push_str(before);
        rest = after
            .find("*/")
            .map_or("", |end| after.get(end.saturating_add(2)..).unwrap_or(""));
    }
    clean.push_str(rest);
    let mut found = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut text = String::new();
    for ch in clean.chars() {
        match ch {
            '{' => {
                open.push(text.split_whitespace().collect::<Vec<_>>().join(" "));
                text.clear();
            }
            '}' => {
                if let Some(selector) = open.pop() {
                    let body = text.trim().to_owned();
                    if !body.is_empty() && !selector.starts_with('@') {
                        found.push((selector, body, open.clone()));
                    }
                }
                text.clear();
            }
            other => text.push(other),
        }
    }
    found
}

/// The body of the rule for exactly `selector` outside any at-rule but the kit's own.
fn body<'a>(found: &'a [(String, String, Vec<String>)], selector: &str, reduced: bool) -> &'a str {
    found
        .iter()
        .find(|(s, _, around)| {
            s.split(',').any(|one| one.trim() == selector)
                && around.iter().any(|a| a.contains("prefers-reduced-motion")) == reduced
        })
        .map_or("", |(_, b, _)| b.as_str())
}

#[test]
fn the_dropdown_fades_in_from_its_trigger_and_out_faster() {
    let found = rules(COMPONENTS);
    let closed = body(&found, ".nav-dropdown-content", false);
    assert!(!closed.contains("display: none"), "{closed}");
    for part in ["visibility: hidden;", "opacity: 0;", "transform:", "100ms"] {
        assert!(closed.contains(part), "closed lacks {part}: {closed}");
    }
    let shown = body(&found, ".nav-dropdown-open .nav-dropdown-content", false);
    for part in [
        "visibility: visible;",
        "opacity: 1;",
        "transform: none;",
        "150ms",
    ] {
        assert!(shown.contains(part), "open lacks {part}: {shown}");
    }
}

#[test]
fn the_phone_menu_moves_without_reflowing_the_page() {
    let found = rules(NAV_FOLD);
    // A layout property may only step (0s), never ease over frames.
    for (selector, body, _) in &found {
        let Some(at) = body.find("transition:") else {
            continue;
        };
        let list = body
            .get(at.saturating_add("transition:".len())..)
            .unwrap_or("");
        let list = list.split(';').next().unwrap_or("");
        for item in list.split(',') {
            let words: Vec<&str> = item.split_whitespace().collect();
            let (Some(property), Some(duration)) = (words.first(), words.get(1)) else {
                continue;
            };
            let layout = [
                "grid-template-rows",
                "margin-top",
                "padding",
                "height",
                "max-height",
            ];
            assert!(
                !layout.contains(property) || *duration == "0s",
                "{selector} animates {property} over {duration}"
            );
        }
    }
    let panel = body(&found, ".nav-panel", false);
    for part in [
        "position: absolute;",
        "top: 100%;",
        "opacity: 0;",
        "transform:",
        "120ms",
    ] {
        assert!(panel.contains(part), "closed panel lacks {part}: {panel}");
    }
    let shown = body(&found, ".nav-panel-open", false);
    for part in ["opacity: 1;", "transform: none;", "180ms"] {
        assert!(shown.contains(part), "open panel lacks {part}: {shown}");
    }
}

#[test]
fn reduced_motion_keeps_only_a_quick_fade() {
    let dropdown = rules(COMPONENTS);
    let fold = rules(NAV_FOLD);
    for (found, selector) in [(&dropdown, ".nav-dropdown-content"), (&fold, ".nav-panel")] {
        let calm = body(found, selector, true);
        assert!(calm.contains("transform: none;"), "{selector}: {calm}");
        assert!(calm.contains("opacity"), "{selector} keeps a fade: {calm}");
    }
}
