//! The ink gate for `assets/themes.css`, per `context/development/styling.md`.
//!
//! Every theme block carries an `--ink-<tone>` for each chip tone. The ink reads at 4.5:1 or better
//! on everything a chip of that tone sits on: its 12% tint over `--bg-primary` and over `--bg-sink`,
//! and either background bare. Inks are made for 4.6:1, a margin for the grounds a page paints
//! between these, so where the tone already reads at 4.6 the ink is the tone itself.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: a bad slice or an overflow fails it"
)]

const CSS: &str = include_str!("../assets/themes.css");

/// The tones a chip, a tag or a stat takes its color from.
const TONES: [&str; 14] = [
    "accent-primary",
    "accent-secondary",
    "accent-tertiary",
    "palette-1",
    "palette-2",
    "palette-3",
    "palette-4",
    "palette-5",
    "palette-6",
    "text-muted",
    "color-money",
    "color-success",
    "color-warning",
    "color-error",
];

const TARGET: f64 = 4.5;

/// What the inks are made for: the target with a margin.
const MADE_FOR: f64 = 4.6;

struct Block {
    name: String,
    vars: Vec<(String, String)>,
}

fn blocks() -> Vec<Block> {
    let mut out = Vec::new();
    let mut rest = CSS;
    while let Some(start) = rest.find(".theme-") {
        let after = &rest[start + ".theme-".len()..];
        let Some(brace) = after.find('{') else { break };
        let name = after[..brace].trim().to_string();
        let body_start = brace + 1;
        let Some(end) = after[body_start..].find('}') else {
            break;
        };
        let body = &after[body_start..body_start + end];
        let vars = body
            .lines()
            .filter_map(|l| {
                let l = l.trim().strip_prefix("--")?;
                let (k, v) = l.split_once(':')?;
                Some((
                    k.trim().to_string(),
                    v.trim().trim_end_matches(';').trim().to_string(),
                ))
            })
            .collect();
        out.push(Block { name, vars });
        rest = &after[body_start + end..];
    }
    out
}

impl Block {
    fn raw(&self, key: &str) -> &str {
        match self.vars.iter().find(|(k, _)| k == key) {
            Some((_, value)) => value,
            None => panic!("{} has no --{key}", self.name),
        }
    }

    fn color(&self, key: &str) -> [f64; 3] {
        let value = self.raw(key);
        match hex(value) {
            Some(color) => color,
            None => panic!("{}'s --{key} is not a hex color: {value}", self.name),
        }
    }
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

fn luminance(rgb: [f64; 3]) -> f64 {
    let [r, g, b] = rgb.map(lin);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// `color-mix(in srgb, a share, b)`.
fn mix(a: [f64; 3], b: [f64; 3], share: f64) -> [f64; 3] {
    [0, 1, 2].map(|i| a[i] * share + b[i] * (1.0 - share))
}

/// What a chip of `tone` sits on in this block: its tint over each background, and each bare.
/// `--bg-sink` is mixed as the sheet's derived rule mixes it.
fn grounds(block: &Block, tone: [f64; 3]) -> [[f64; 3]; 4] {
    let bg = block.color("bg-primary");
    let sink = if block.name.ends_with("-dark") {
        mix(bg, [0.0; 3], 0.85)
    } else {
        mix(bg, [1.0; 3], 0.65)
    };
    [mix(tone, bg, 0.12), mix(tone, sink, 0.12), bg, sink]
}

fn worst(block: &Block, text: [f64; 3], tone: [f64; 3]) -> f64 {
    grounds(block, tone)
        .into_iter()
        .map(|ground| contrast(text, ground))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn every_tone_has_an_ink_that_reads_on_its_tint() {
    let blocks = blocks();
    assert!(blocks.len() >= 100, "found {} theme blocks", blocks.len());
    let mut failures = Vec::new();
    for block in &blocks {
        for tone_name in TONES {
            let tone = block.color(tone_name);
            let ink = block.color(&format!("ink-{tone_name}"));
            let reads = worst(block, ink, tone);
            if reads < TARGET {
                failures.push(format!("{} --ink-{tone_name} reads {reads:.2}", block.name));
            }
            let same = block
                .raw(tone_name)
                .eq_ignore_ascii_case(block.raw(&format!("ink-{tone_name}")));
            if worst(block, tone, tone) >= MADE_FOR && !same {
                failures.push(format!(
                    "{} --ink-{tone_name} differs from a tone that already reads",
                    block.name
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_root_defaults_carry_every_ink() {
    let root = CSS
        .find(":root {")
        .and_then(|start| CSS[start..].find('}').map(|end| &CSS[start..start + end]))
        .unwrap_or_default();
    for tone in TONES {
        assert!(
            root.contains(&format!("--ink-{tone}:")),
            ":root has no --ink-{tone}"
        );
    }
}
