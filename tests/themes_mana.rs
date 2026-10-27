//! The mana gate for `assets/themes.css`, per `context/development/styling.md`.
//!
//! Every theme block and the `:root` default carry the five `--mana-*` colors,
//! Magic's white, blue, black, red and green drawn from that theme's own
//! palette, each a hex. The `:root` default repeats gruvbox dark. Any two of a
//! block's five must sit at least 3 apart in OKLab; pairs under 8 are printed,
//! never failed, since a theme with no blue of its own (miasma) keeps the
//! colors it has.

#![expect(
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::many_single_char_names,
    reason = "a test: the close-pair report is its output, an overflow fails it, and the color math keeps the standard one-letter channel names"
)]

const CSS: &str = include_str!("../assets/themes.css");

const MANA: [&str; 5] = ["white", "blue", "black", "red", "green"];

/// Each theme block's name and body, `gruvbox-dark` and the text between its
/// braces, then `root` and the `:root` default's body.
fn blocks() -> Vec<(&'static str, &'static str)> {
    let mut out: Vec<(&str, &str)> = CSS
        .split(".theme-")
        .skip(1)
        .filter_map(|chunk| {
            let (name, rest) = chunk.split_once(" {")?;
            if !(name.ends_with("-dark") || name.ends_with("-light")) {
                return None;
            }
            let (body, _) = rest.split_once('}')?;
            Some((name, body))
        })
        .collect();
    if let Some((_, rest)) = CSS.split_once("\n:root {")
        && let Some((body, _)) = rest.split_once('}')
    {
        out.push(("root", body));
    }
    out
}

fn value<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    body.lines().find_map(|l| {
        let (k, v) = l.trim().split_once(':')?;
        (k.trim() == key).then(|| v.trim().trim_end_matches(';').trim())
    })
}

fn hex(s: &str) -> Option<[f64; 3]> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let ch = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
    Some([
        f64::from(ch(0)?) / 255.0,
        f64::from(ch(2)?) / 255.0,
        f64::from(ch(4)?) / 255.0,
    ])
}

fn lin(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn oklab(rgb: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(lin);
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    ]
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let [x, y, z] = oklab(a);
    let [p, q, r] = oklab(b);
    100.0 * ((x - p).powi(2) + (y - q).powi(2) + (z - r).powi(2)).sqrt()
}

#[test]
fn every_theme_has_five_distinct_mana_colors() {
    let blocks = blocks();
    assert!(
        blocks.len() >= 61,
        "expected the theme blocks and :root, found {}",
        blocks.len()
    );
    let mut failures = Vec::new();
    let mut close = Vec::new();
    for (name, body) in &blocks {
        let mut five = Vec::new();
        for color in MANA {
            match value(body, &format!("--mana-{color}")).and_then(hex) {
                Some(c) => five.push((color, c)),
                None => failures.push(format!("{name}: --mana-{color} missing or not a hex")),
            }
        }
        for (i, (a, x)) in five.iter().enumerate() {
            for (b, y) in five.iter().skip(i + 1) {
                let d = distance(*x, *y);
                if d < 3.0 {
                    failures.push(format!("{name}: {a} and {b} are {d:.1} apart"));
                } else if d < 8.0 {
                    close.push(format!("{name}: {a} and {b} are {d:.1} apart"));
                }
            }
        }
    }
    if !close.is_empty() {
        println!("mana pairs under 8 (reported, not enforced):");
        for c in &close {
            println!("  {c}");
        }
    }
    assert!(
        failures.is_empty(),
        "mana colors failed:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn root_mana_repeats_gruvbox_dark() {
    let blocks = blocks();
    let find = |n: &str| blocks.iter().find(|(name, _)| *name == n).map(|(_, b)| *b);
    let (Some(root), Some(gruvbox)) = (find("root"), find("gruvbox-dark")) else {
        panic!("no :root or gruvbox-dark block");
    };
    for color in MANA {
        let key = format!("--mana-{color}");
        assert_eq!(
            value(root, &key),
            value(gruvbox, &key),
            ":root {key} differs from gruvbox-dark"
        );
    }
}
