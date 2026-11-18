//! The chapter title card's lettering (ticket 0812, ADR-0128): Cinzel,
//! the one font besides the cell font (Nick: a title card may break from
//! one font for everything), baked by `cargo xtask lettering` into two
//! glyph sheets in `assets/title/`, each a white-on-clear PNG and a
//! `FontAtlasDef` beside it whose `ink` gives each glyph's columns. The
//! card draws a line of it as sprites, one a glyph (ADR-0038). Format:
//! `assets/title/README.md`.
use crate::prelude::*;

pub use shuyi_assets::font::FontAtlasDef;

use crate::bundle;
use crate::error::ContentError;
use crate::image::{ImageId, ImageTable};

/// The glyphs a sheet must hold: printable ASCII. A title with any other
/// character is drawn in the cell font instead.
pub const LETTERING_GLYPHS: &str = concat!(
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`",
    "abcdefghijklmnopqrstuvwxyz{|}~",
);

/// One baked size of a font: its sheet and where each glyph is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    /// The PNG, white glyphs on clear.
    pub image: ImageId,
    /// Each glyph's cell, and its inked columns (`ink`).
    pub atlas: FontAtlasDef,
}

/// The chapter card's two sizes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lettering {
    /// The chapter's name, big (Cinzel Bold).
    pub heading: Sheet,
    /// The `Chapter 1` line above it, smaller (Cinzel Medium).
    pub line: Sheet,
}

/// The sheet `stem` (`<dir>/<stem>.ron` and `.png` in the bundle), checked
/// against its image in `images`: every glyph of [`LETTERING_GLYPHS`],
/// every cell inside the image, and the ink of every glyph given.
pub fn sheet(dir: &str, stem: &str, images: &ImageTable) -> Result<Sheet, Vec<ContentError>> {
    let (ron, png) = (format!("{dir}/{stem}.ron"), format!("{dir}/{stem}.png"));
    let file = bundle::display_path(&ron);
    let source = bundle::file(&ron)
        .ok_or_else(|| vec![ContentError::new(&file, "file not found in asset bundle")])?;
    let atlas = from_source(&file, source, &png, bundle::bytes(&png))?;
    let image = images.id(&png).ok_or_else(|| {
        let shown = bundle::display_path(&png);
        vec![ContentError::new(
            &file,
            format!("{shown} is not a PNG image in the assets"),
        )]
    })?;
    Ok(Sheet { image, atlas })
}

/// Parses sheet `source` (errors attributed to `file`) against its PNG
/// at bundle path `png` (`bytes`, `None` if missing): the atlas's own
/// checks, then that it says where each glyph's ink is.
pub fn from_source(
    file: &str,
    source: &str,
    png: &str,
    bytes: Option<&[u8]>,
) -> Result<FontAtlasDef, Vec<ContentError>> {
    let shown = bundle::display_path(png);
    let atlas = FontAtlasDef::from_source(file, source, &shown, bytes, LETTERING_GLYPHS)?;
    if atlas.ink.is_none() {
        return Err(vec![ContentError::new(
            file,
            "no `ink`: rerun `cargo xtask lettering`",
        )]);
    }
    Ok(atlas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::png_header;

    const FILE: &str = "assets/title/x.ron";

    /// A sheet of every glyph in 4×6 cells, 16 a row (96 glyphs: 6 rows).
    fn source(ink: bool) -> String {
        let glyphs: Vec<String> = LETTERING_GLYPHS
            .chars()
            .enumerate()
            .map(|(i, c)| format!("'{}': {i}", c.escape_default()))
            .collect();
        let ink = if ink {
            ", ink: Some((capitals: (left: 0, top: 1, right: 3, bottom: 5)))"
        } else {
            ""
        };
        format!(
            "(cell_w: 4, cell_h: 6, columns: 16, glyphs: {{{}}}{ink})",
            glyphs.join(", ")
        )
    }

    #[test]
    fn a_sheet_with_ink_and_every_glyph_loads() {
        let atlas = from_source(
            FILE,
            &source(true),
            "title/x.png",
            Some(&png_header(64, 36)),
        );
        let atlas = atlas.unwrap();
        assert_eq!((atlas.cell_w, atlas.cell_h), (4, 6));
        assert!(LETTERING_GLYPHS.chars().all(|c| atlas.draws(c)));
    }

    #[test]
    fn a_sheet_needs_its_ink_its_glyphs_and_its_image() {
        let png = png_header(64, 36);
        let errors = from_source(FILE, &source(false), "title/x.png", Some(&png)).unwrap_err();
        assert_eq!(
            errors[0].to_string(),
            format!("{FILE}: no `ink`: rerun `cargo xtask lettering`")
        );
        let few = "(cell_w: 4, cell_h: 6, columns: 16, glyphs: {'a': 0})";
        let errors = from_source(FILE, few, "title/x.png", Some(&png)).unwrap_err();
        assert!(
            errors[0].message.starts_with("missing required glyphs"),
            "{errors:?}"
        );
        let errors = from_source(FILE, &source(true), "title/x.png", None).unwrap_err();
        assert_eq!(
            errors.last().map(ToString::to_string).as_deref(),
            Some("assets/title/x.png: file not found in asset bundle")
        );
    }

    #[test]
    fn the_embedded_sheets_load() {
        let images = crate::image::load().unwrap();
        for stem in ["chapter_name", "chapter_line"] {
            let sheet = sheet("title", stem, &images).unwrap();
            assert_eq!(sheet.image.path(), format!("title/{stem}.png"));
        }
        let errors = sheet("title", "none", &images).unwrap_err();
        assert_eq!(
            errors[0].to_string(),
            "assets/title/none.ron: file not found in asset bundle"
        );
    }
}
