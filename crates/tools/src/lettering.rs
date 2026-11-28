//! `cargo xtask lettering`: bakes the chapter title card's lettering
//! (ticket 0812, ADR-0128): Cinzel, from `assets-src/fonts/`, into two
//! glyph sheets in `assets/title/` that `trpg_content::lettering` reads:
//! `chapter_name` (Cinzel Bold, 40 px) and `chapter_line` (Cinzel Medium,
//! 20 px). Each is a PNG of white glyphs whose alpha is the font's
//! coverage, 16 cells a row, and a `FontAtlasDef` beside it with each
//! glyph's cell and its inked columns (`ink`), which the card lays a line
//! out by.
//!
//! A stand-in for the engine's proportional fonts (engine tickets 2224
//! and 2334), which bake TTF files themselves: when they land, the card
//! moves to them and this tool goes.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use fontdue::{Font, FontSettings};
use shuyi_cli::font_atlas::{BitmapFont, ink_of};
use shuyi_cli::prelude::encode_png;
use shuyi_platform::prelude::text::FontAtlasDef;
use trpg_content::lettering::LETTERING_GLYPHS;
use trpg_content::title::TITLE_DIR;

/// A sheet to bake: its file stem, the font file and the size in pixels.
pub struct Bake {
    /// The file stem in `assets/title/`.
    pub stem: &'static str,
    /// The font, in `assets-src/fonts/`.
    pub font: &'static str,
    /// The size, in pixels to the em.
    pub px: f32,
}

/// The two sheets (*tunable*: the sizes are the mockups Nick picked from).
pub const BAKES: [Bake; 2] = [
    Bake {
        stem: "chapter_name",
        font: "Cinzel-Bold.ttf",
        px: 40.0,
    },
    Bake {
        stem: "chapter_line",
        font: "Cinzel-Medium.ttf",
        px: 20.0,
    },
];

/// Cells a row in a sheet.
pub const COLUMNS: u32 = 16;

/// The least coverage (of 255) a pixel counts as ink at: fainter edges
/// don't widen a glyph's columns.
pub const INK: u8 = 96;

/// A baked sheet: its atlas and its RGBA pixels.
pub struct Sheet {
    /// Each glyph's cell, and its ink.
    pub def: FontAtlasDef,
    /// The image's width and height.
    pub size: (u32, u32),
    /// The image's RGBA8 pixels, row-major.
    pub rgba: Vec<u8>,
}

fn to_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

fn to_usize(v: u32) -> usize {
    usize::try_from(v).unwrap_or(usize::MAX)
}

/// Bakes `font` (a TTF's bytes) at `px` pixels to the em.
pub fn bake(font: &[u8], px: f32) -> Result<Sheet, String> {
    let font = Font::from_bytes(font, FontSettings::default()).map_err(str::to_owned)?;
    let rastered: Vec<(char, fontdue::Metrics, Vec<u8>)> = LETTERING_GLYPHS
        .chars()
        .map(|c| {
            let (m, coverage) = font.rasterize(c, px);
            (c, m, coverage)
        })
        .collect();
    // Every glyph's left edge at the cell's, so none is cut; the baseline
    // the same row in every cell.
    let left = rastered
        .iter()
        .map(|(_, m, _)| m.xmin)
        .min()
        .unwrap_or(0)
        .min(0);
    // The baseline is as far down as the tallest glyph's top.
    let glyph_h = |m: &fontdue::Metrics| i32::try_from(m.height).unwrap_or(0);
    let baseline = rastered
        .iter()
        .map(|(_, m, _)| m.ymin + glyph_h(m))
        .max()
        .unwrap_or(0)
        .max(1);
    let right = rastered
        .iter()
        .map(|(_, m, _)| m.xmin - left + i32::try_from(m.width).unwrap_or(0))
        .max()
        .unwrap_or(1);
    let below = rastered
        .iter()
        .map(|(_, m, _)| -m.ymin)
        .max()
        .unwrap_or(0)
        .max(0);
    let cell_w = u32::try_from(right.max(1)).map_err(|e| e.to_string())?;
    let cell_h = u32::try_from(baseline + below).map_err(|e| e.to_string())?;
    let rows = to_u32(rastered.len()).div_ceil(COLUMNS);
    let (width, height) = (cell_w * COLUMNS, cell_h * rows);
    let mut rgba = vec![0; to_usize(width * height * 4)];
    let mut glyphs = BTreeMap::new();
    let mut ink = BitmapFont {
        cell_w,
        cell_h,
        glyphs: BTreeMap::new(),
    };
    for (index, (c, m, coverage)) in rastered.iter().enumerate() {
        let index = to_u32(index);
        glyphs.insert(*c, index);
        let (cx, cy) = (index % COLUMNS * cell_w, index / COLUMNS * cell_h);
        // fontdue's `ymin` is the bitmap's bottom above the baseline.
        let top = baseline - (m.ymin + glyph_h(m));
        let mut on = vec![false; to_usize(cell_w * cell_h)];
        for (i, &a) in coverage.iter().enumerate() {
            let (gx, gy) = (to_u32(i % m.width.max(1)), to_u32(i / m.width.max(1)));
            let x = u32::try_from(m.xmin - left).unwrap_or(0) + gx;
            let Ok(y) = u32::try_from(top + i32::try_from(gy).unwrap_or(0)) else {
                continue;
            };
            if a == 0 || x >= cell_w || y >= cell_h {
                continue;
            }
            let at = to_usize(((cy + y) * width + cx + x) * 4);
            rgba[at..at + 4].copy_from_slice(&[255, 255, 255, a]);
            on[to_usize(y * cell_w + x)] = a >= INK;
        }
        ink.glyphs.insert(*c, on);
    }
    let font_ink = ink_of(&ink, &glyphs);
    let mut def = FontAtlasDef::new(cell_w, cell_h, glyphs, COLUMNS);
    def.ink = font_ink;
    Ok(Sheet {
        def,
        size: (width, height),
        rgba,
    })
}

/// The `.ron` text of `def`, baked from `font`.
pub fn ron_text(def: &FontAtlasDef, font: &str) -> Result<String, String> {
    let config = ron::ser::PrettyConfig::default()
        .new_line("\n")
        .compact_arrays(true);
    let body = ron::ser::to_string_pretty(def, config).map_err(|e| e.to_string())?;
    Ok(format!(
        "// Generated by `cargo xtask lettering` from assets-src/fonts/{font}. Do not edit;\n\
         // see assets/title/README.md. Maps each glyph to its cell in the PNG beside it.\n\
         {body}\n"
    ))
}

/// The files of `sheet`, made from its font under repo root `root`: the
/// `.ron` text and the PNG.
pub fn files(root: &Path, sheet: &Bake) -> Result<(String, Vec<u8>), String> {
    let path = root.join("assets-src/fonts").join(sheet.font);
    let font = fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let baked = bake(&font, sheet.px)?;
    let png = encode_png(baked.size.0, baked.size.1, &baked.rgba)?;
    Ok((ron_text(&baked.def, sheet.font)?, png))
}

/// Runs the command: writes every sheet under repo root `root`.
pub fn run(root: &Path) -> Result<String, String> {
    let dir = root.join("assets").join(TITLE_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let mut wrote = Vec::new();
    for bake in &BAKES {
        let (ron, png) = files(root, bake)?;
        for (ext, bytes) in [("ron", ron.into_bytes()), ("png", png)] {
            let path = dir.join(format!("{}.{ext}", bake.stem));
            fs::write(&path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
        }
        wrote.push(bake.stem);
    }
    Ok(format!(
        "lettering: wrote {} in assets/{TITLE_DIR}/",
        wrote.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_root;

    #[test]
    fn committed_sheets_are_what_the_tool_makes() {
        let root = repo_root();
        for bake in &BAKES {
            let (ron, png) = files(&root, bake).unwrap();
            let dir = root.join("assets").join(TITLE_DIR);
            let committed = fs::read_to_string(dir.join(format!("{}.ron", bake.stem))).unwrap();
            assert_eq!(committed, ron, "rerun `cargo xtask lettering`");
            let committed = fs::read(dir.join(format!("{}.png", bake.stem))).unwrap();
            assert!(committed == png, "rerun `cargo xtask lettering`");
        }
    }

    #[test]
    fn a_sheet_holds_every_glyph_on_one_baseline_with_its_ink() {
        let font = fs::read(repo_root().join("assets-src/fonts/Cinzel-Bold.ttf")).unwrap();
        let sheet = bake(&font, 40.0).unwrap();
        let def = &sheet.def;
        assert!(LETTERING_GLYPHS.chars().all(|c| def.draws(c)));
        assert_eq!(sheet.size, (def.cell_w * COLUMNS, def.cell_h * 6));
        let ink = def.ink.as_ref().unwrap();
        // Cinzel's capitals sit on one baseline, as tall as each other.
        assert!(ink.capitals.bottom > ink.capitals.top + 20);
        // An `I` is narrower than a `W`.
        let width = |c| {
            let (l, r) = ink.columns_of(c);
            r - l
        };
        assert!(width('I') < width('W'));
        // A space is blank.
        let space = def.glyph_rect(' ').unwrap();
        let alpha = |x: u32, y: u32| sheet.rgba[to_usize((y * sheet.size.0 + x) * 4 + 3)];
        let blank =
            (0..space.h).all(|y| (0..space.w).all(|x| alpha(space.x + x, space.y + y) == 0));
        assert!(blank);
    }

    #[test]
    fn a_font_that_is_not_one_fails() {
        assert!(bake(b"nope", 20.0).is_err());
        let err = files(Path::new("/nowhere"), &BAKES[0]).unwrap_err();
        assert!(err.starts_with("reading "), "{err}");
    }

    #[test]
    fn run_writes_both_sheets() {
        let root = std::env::temp_dir().join(format!("xtask-lettering-{}", std::process::id()));
        let src = root.join("assets-src/fonts");
        fs::create_dir_all(&src).unwrap();
        for bake in &BAKES {
            let font = repo_root().join("assets-src/fonts").join(bake.font);
            fs::copy(font, src.join(bake.font)).unwrap();
        }
        let summary = run(&root).unwrap();
        assert_eq!(
            summary,
            "lettering: wrote chapter_name, chapter_line in assets/title/"
        );
        assert!(root.join("assets/title/chapter_line.png").is_file());
        fs::remove_dir_all(&root).unwrap();
    }
}
