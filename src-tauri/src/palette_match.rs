//! Open Color / Tailwind サブセットへの ΔE2000 最近傍照合

use crate::color_theory::{delta_e_2000, lab_from_srgb, Lab};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct PaletteSwatchLab {
    pub name: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub lab: Lab,
}

#[derive(Clone, Debug)]
pub struct DominantPaletteMatch {
    pub dom_r: u8,
    pub dom_g: u8,
    pub dom_b: u8,
    pub pct: f32,
    pub swatch_name: String,
    pub sw_r: u8,
    pub sw_g: u8,
    pub sw_b: u8,
    pub delta_e: f64,
}

#[derive(Deserialize)]
struct OpenColorJson {
    white: String,
    black: String,
    gray: Vec<String>,
    red: Vec<String>,
    pink: Vec<String>,
    grape: Vec<String>,
    violet: Vec<String>,
    indigo: Vec<String>,
    blue: Vec<String>,
    cyan: Vec<String>,
    teal: Vec<String>,
    green: Vec<String>,
    lime: Vec<String>,
    yellow: Vec<String>,
    orange: Vec<String>,
}

#[derive(Deserialize)]
struct FlatPaletteJson {
    swatches: Vec<FlatSwatch>,
}

#[derive(Deserialize)]
struct FlatSwatch {
    name: String,
    hex: String,
}

fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

fn swatch_from_hex(name: String, hex: &str) -> Option<PaletteSwatchLab> {
    let (r, g, b) = parse_hex_rgb(hex)?;
    let lab = lab_from_srgb(r, g, b);
    Some(PaletteSwatchLab { name, r, g, b, lab })
}

fn flatten_open_color(j: OpenColorJson) -> Vec<PaletteSwatchLab> {
    let mut out = Vec::new();
    if let Some(s) = swatch_from_hex("white".to_string(), &j.white) {
        out.push(s);
    }
    if let Some(s) = swatch_from_hex("black".to_string(), &j.black) {
        out.push(s);
    }
    let hues = [
        ("gray", j.gray),
        ("red", j.red),
        ("pink", j.pink),
        ("grape", j.grape),
        ("violet", j.violet),
        ("indigo", j.indigo),
        ("blue", j.blue),
        ("cyan", j.cyan),
        ("teal", j.teal),
        ("green", j.green),
        ("lime", j.lime),
        ("yellow", j.yellow),
        ("orange", j.orange),
    ];
    for (hue, arr) in hues {
        for (i, hex) in arr.into_iter().enumerate() {
            if let Some(s) = swatch_from_hex(format!("{hue}-{i}"), &hex) {
                out.push(s);
            }
        }
    }
    out
}

static OPEN_COLOR_PALETTE: OnceLock<Vec<PaletteSwatchLab>> = OnceLock::new();
static TAILWIND_PALETTE: OnceLock<Vec<PaletteSwatchLab>> = OnceLock::new();

pub fn open_color_palette() -> &'static [PaletteSwatchLab] {
    OPEN_COLOR_PALETTE.get_or_init(|| {
        let raw = include_str!("../assets/open_color.json");
        let j: OpenColorJson = serde_json::from_str(raw).expect("parse assets/open_color.json");
        flatten_open_color(j)
    })
}

pub fn tailwind_subset_palette() -> &'static [PaletteSwatchLab] {
    TAILWIND_PALETTE.get_or_init(|| {
        let raw = include_str!("../assets/tailwind_subset.json");
        let j: FlatPaletteJson = serde_json::from_str(raw).expect("parse assets/tailwind_subset.json");
        j.swatches
            .into_iter()
            .filter_map(|s| swatch_from_hex(s.name, &s.hex))
            .collect()
    })
}

pub fn nearest_in_palette(r: u8, g: u8, b: u8, palette: &[PaletteSwatchLab]) -> Option<(&PaletteSwatchLab, f64)> {
    let lab = lab_from_srgb(r, g, b);
    let mut best_sw: &PaletteSwatchLab = palette.first()?;
    let mut best_de = delta_e_2000(lab, best_sw.lab);
    for sw in &palette[1..] {
        let de = delta_e_2000(lab, sw.lab);
        if de < best_de {
            best_de = de;
            best_sw = sw;
        }
    }
    Some((best_sw, best_de))
}

pub fn match_dominants(
    dominants: &[(u8, u8, u8, f32)],
    palette: &[PaletteSwatchLab],
) -> Vec<DominantPaletteMatch> {
    if palette.is_empty() {
        return Vec::new();
    }
    dominants
        .iter()
        .filter_map(|&(dr, dg, db, pct)| {
            let (sw, de) = nearest_in_palette(dr, dg, db, palette)?;
            Some(DominantPaletteMatch {
                dom_r: dr,
                dom_g: dg,
                dom_b: db,
                pct,
                swatch_name: sw.name.clone(),
                sw_r: sw.r,
                sw_g: sw.g,
                sw_b: sw.b,
                delta_e: de,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tailwind_nearest_blue_500_exact() {
        let p = tailwind_subset_palette();
        let (sw, de) = nearest_in_palette(59, 130, 246, p).expect("palette must not be empty");
        assert_eq!(sw.name, "blue-500");
        assert!(de < 1.0, "ΔE00 self-match should be tiny, got {}", de);
    }

    #[test]
    fn nearest_in_palette_empty_returns_none() {
        assert!(nearest_in_palette(255, 0, 0, &[]).is_none());
    }
}
