//! The loading card (ticket 1036): what the screen shows from the first
//! frame while the game's data loads, so it is never blank or the SDK's
//! own. The game's name, centred, in the game's font, black on the white
//! ground of the Playdate's one-bit look (`docs/design/playdate.md`).
//! *Claude's starting rule (Nick can veto at the 1050 play-through).*
//!
//! The font comes first and on its own: the glyph map
//! (`fonts/atlas.ron`) and the one-bit glyphs (`fonts/atlas_1bit.bin`),
//! not the rest of the data. The game draws with the same font once it
//! runs.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use trpg_content::font::ATLAS_1BIT_PATH;
use trpg_content::{FontAtlasDef, Ink};
use trpg_ui::glyph_buffer::Cell;
use trpg_ui::onebit;
use trpg_ui::{ConsoleSpec, GlyphBuffer, Rgb};

/// The game's name on the card: the `.pdx`'s name too (`cargo xtask
/// playdate`), and the logo's words.
pub const GAME_NAME: &str = "Visions of Shuyi";

/// The game's font, read from the bundle before anything else. The glyph
/// map is kept for as long as the game runs: the font borrows it.
pub fn font() -> Result<onebit::Font<'static>, String> {
    let atlas = trpg_content::font::load().map_err(|errors| {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    })?;
    let atlas: &'static FontAtlasDef = Box::leak(Box::new(atlas));
    // check-text: not player text (a build problem for the log)
    let bits = trpg_content::bundle::bytes(ATLAS_1BIT_PATH)
        .ok_or_else(|| alloc::format!("{ATLAS_1BIT_PATH} is not in the bundle"))?;
    Ok(onebit::Font::new(atlas, bits)?)
}

/// The card on a console of `spec`: the white ground, and the game's name
/// in black on the middle row, centred.
pub fn buffer(spec: ConsoleSpec) -> GlyphBuffer {
    let (black, white) = (Rgb::from(Ink::Black.code()), Rgb::from(Ink::White.code()));
    let mut buffer = GlyphBuffer::with_spec(spec, Cell::new(' ', black, white));
    let len = i32::try_from(GAME_NAME.chars().count()).unwrap_or(0);
    let x = (i32::from(spec.cols) - len) / 2;
    let y = i32::from(spec.rows) / 2;
    buffer.print(x, y, GAME_NAME, black, white);
    buffer
}

#[cfg(test)]
mod tests {
    use trpg_ui::ConsoleSizes;
    use trpg_ui::onebit::{FRAME_BYTES, ROW_BYTES, render};

    use super::*;

    /// The name sits on row 7 of the Playdate's 50 × 15 console, its 16
    /// letters from column 17: as many cells left of it as right.
    #[test]
    fn the_card_is_the_games_name_centred_on_white() {
        let card = buffer(ConsoleSpec::PLAYDATE);
        let (black, white) = (Rgb::from(Ink::Black.code()), Rgb::from(Ink::White.code()));
        let rows: Vec<String> = (0..15)
            .map(|y| {
                (0..50)
                    .filter_map(|x| card.get(x, y))
                    .map(|c| c.glyph)
                    .collect()
            })
            .collect();
        assert_eq!(rows[7], alloc::format!("{:17}{GAME_NAME}{:17}", "", ""));
        for (y, row) in rows.iter().enumerate() {
            assert!(y == 7 || row.trim().is_empty(), "row {y}: {row:?}");
        }
        for y in 0..15 {
            for x in 0..50 {
                let cell = card.get(x, y).unwrap_or_else(|| panic!("({x}, {y})"));
                assert_eq!((cell.fg, cell.bg), (black, white), "({x}, {y})");
            }
        }
    }

    /// Drawn with the game's font: white everywhere but the name's glyphs,
    /// which are black, on the name's 16 pixel rows only.
    #[test]
    fn the_card_draws_black_letters_on_white_with_the_games_font() {
        let font = font().unwrap_or_else(|e| panic!("{e}"));
        let mut frame = Box::new([0; FRAME_BYTES]);
        render(&buffer(ConsoleSpec::PLAYDATE), &font, &(), &mut frame);
        let (rows, _) = frame.as_chunks::<ROW_BYTES>();
        let inked = |row: &[u8; ROW_BYTES]| row[..50].iter().any(|&b| b != u8::MAX);
        let inked_rows: Vec<usize> = (0..240).filter(|&y| inked(&rows[y])).collect();
        assert_ne!(inked_rows, [] as [usize; 0]);
        assert!(inked_rows.iter().all(|y| (7 * 16..8 * 16).contains(y)));
        // No ink left of the name's first cell nor right of its last.
        for row in rows {
            assert!(row[..17].iter().all(|&b| b == u8::MAX));
            assert!(row[17 + 16..50].iter().all(|&b| b == u8::MAX));
        }
    }
}
