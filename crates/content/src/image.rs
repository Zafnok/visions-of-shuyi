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

/// Path of the test card inside the bundle: a generated 16×16 image
/// (`cargo xtask test-card`) for tests and the "Sprite test" debug tool.
pub const TEST_CARD_PATH: &str = "images/test_card.png";

/// Path of the effect arrows inside the bundle: a generated 14×7 image
/// (`cargo xtask effect-marks`), an up arrow for a bonus then a down arrow
/// for a penalty, each 7×7, which a sprite map skin puts on a unit under a
/// timed effect (ADR-0049).
pub const EFFECT_MARKS_PATH: &str = "images/effect_marks.png";

/// Reads the size of every PNG in the embedded bundle (the font atlas
/// aside, which [`crate::font`] checks), reporting every bad file.
pub fn load() -> Result<ImageTable, Vec<ContentError>> {
    ImageTable::load(bundle::install(), &[ATLAS_PNG_PATH])
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
            Some(ImageInfo {
                width: 16,
                height: 16
            })
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
}
