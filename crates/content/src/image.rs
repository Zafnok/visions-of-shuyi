//! The image table (ADR-0038): every PNG in the asset bundle except the font
//! atlas, by bundle path, with its size. Screens name a picture by an
//! [`ImageId`] from this table; `app` decodes and draws it. `content` reads
//! only each file's header, never its pixels. The table is Shuyi Engine's
//! (`shuyi_assets::image`, ticket 1104); this module names the game's
//! files and reads the game's bundle.
use crate::prelude::*;

pub use shuyi_assets::image::{ImageId, ImageInfo, ImageTable, MAX_SIDE, png_size};

use crate::bundle;
use crate::error::ContentError;
use crate::font::ATLAS_PNG_PATH;
use crate::formats;

/// Path of the test card inside the bundle: a generated 16×16 image (its
/// tool was deleted, `assets/images/README.md`) for tests and the "Sprite
/// test" debug tool.
pub const TEST_CARD_PATH: &str = "images/test_card.png";

/// Path of the effect arrows inside the bundle: a generated 14×7 image
/// (its tool was deleted, `assets/images/README.md`), an up arrow for a bonus then a down arrow
/// for a penalty, each 7×7, which a sprite map skin puts on a unit under a
/// timed effect (ADR-0049).
pub const EFFECT_MARKS_PATH: &str = "images/effect_marks.png";

/// Path of the edge arrows inside the bundle: a generated 42×13 image
/// (its tool was deleted, `assets/images/README.md`), a right, a left, a down and an up arrow,
/// which a sprite map skin puts at the edges of the map's view where the
/// map goes on past it (ticket 0449).
pub const EDGE_MARKS_PATH: &str = "images/edge_marks.png";

/// Path of the arrow cluster inside the bundle: a 24×16 picture of the
/// four arrow keys as a keyboard has them, four 8×8 key caps in an
/// upside-down T, each with its arrow, which a hint draws for the cursor's
/// keys when they are the arrows (ticket 0254, `docs/design/look-and-feel.md`,
/// *Key names in hints*). Our own picture, in the default theme's key colours.
pub const KEY_ARROWS_PATH: &str = "images/key_arrows.png";

/// Reads the size of every PNG in the embedded bundle (the font atlas
/// aside, which [`crate::font`] checks), reporting every bad file.
pub fn load() -> Result<ImageTable, Vec<ContentError>> {
    ImageTable::load_with(bundle::install(), formats::install(), &[ATLAS_PNG_PATH])
}

/// The table of `files` (bundle path and contents), or every problem
/// found, each file named as the game's bundle shows it.
pub fn from_files<'a>(
    files: impl IntoIterator<Item = (&'static str, &'a [u8])>,
) -> Result<ImageTable, Vec<ContentError>> {
    ImageTable::from_files(bundle::install(), files)
}

/// The start of a `w × h` PNG file: enough for [`png_size`].
#[cfg(test)]
pub(crate) fn png_header(w: u32, h: u32) -> Vec<u8> {
    let mut v = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    v.extend(w.to_be_bytes());
    v.extend(h.to_be_bytes());
    v.extend([8, 6, 0, 0, 0]);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_lists_the_test_card_and_not_the_font_atlas() {
        let table = load();
        assert!(table.is_ok(), "{table:?}");
        let table = table.unwrap_or_default();
        let card = table.id(TEST_CARD_PATH);
        assert_eq!(
            card.and_then(|id| table.info(id)),
            Some(ImageInfo::new(16, 16))
        );
        assert_eq!(table.id(ATLAS_PNG_PATH), None);
        assert!(bundle::bytes(ATLAS_PNG_PATH).is_some());
    }

    #[test]
    fn from_files_names_files_as_the_bundle_shows_them() {
        let errors = from_files([("images/x.png", &b"nope"[..])]).unwrap_err();
        assert_eq!(
            errors[0].to_string(),
            "assets/images/x.png: not a valid PNG file"
        );
        let png = png_header(2, 3);
        assert_eq!(png_size(&png), Some((2, 3)));
        assert!(from_files([("images/x.png", &png[..])]).is_ok());
    }

    /// The pictures baked from the palette whose tools were deleted
    /// (ticket 0191), and the palette names each tool drew with.
    const BAKED: [(&str, &[&str]); 2] = [
        (
            EFFECT_MARKS_PATH,
            &["effect_bonus", "effect_penalty", "black"],
        ),
        (EDGE_MARKS_PATH, &["cursor", "black"]),
    ];

    /// Every pixel of `path` that isn't see-through, as RGB; a pixel that
    /// is only partly see-through is an error.
    fn opaque_pixels(path: &str) -> Vec<[u8; 3]> {
        let bytes = bundle::bytes(path).unwrap_or_else(|| panic!("no {path}"));
        let (_, _, rgba) = shuyi_assets::bitmap::convert::png_pixels(&bytes)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        let (rgba, rest) = rgba.as_chunks::<4>();
        assert!(rest.is_empty(), "{path}: a part of a pixel");
        rgba.iter()
            .filter(|&&[.., a]| a != 0)
            .map(|&[r, g, b, a]| {
                assert_eq!(a, 255, "{path}: a partly see-through pixel");
                [r, g, b]
            })
            .collect()
    }

    #[test]
    fn baked_pictures_are_drawn_in_their_palette_colors() {
        let palette = crate::palette::load().expect("the palette loads");
        for (path, names) in BAKED {
            let colors: Vec<[u8; 3]> = names
                .iter()
                .map(|n| palette.get(n).unwrap_or_else(|| panic!("no colour {n}")))
                .collect();
            let pixels = opaque_pixels(path);
            for pixel in &pixels {
                assert!(
                    colors.contains(pixel),
                    "{path} has {pixel:?}, not one of the palette's {names:?}: \
                     the palette changed, so restore its tool (assets/images/README.md)"
                );
            }
            for (name, color) in names.iter().zip(&colors) {
                assert!(pixels.contains(color), "{path} doesn't use {name}");
            }
        }
    }
}
