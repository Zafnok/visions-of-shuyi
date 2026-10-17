//! The one-bit renderer (ADR-0074) is Shuyi Engine's
//! ([`shuyi_ui::frame::onebit`], ticket 1108). The test here draws with
//! this game's one-bit font (`trpg_content::font::ATLAS_1BIT_PATH`).

pub use shuyi_ui::frame::onebit::*;

#[cfg(test)]
mod tests {
    use trpg_content::Ink;

    use super::*;
    use crate::color::Rgb;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::{Cell, GlyphBuffer};

    /// The colour that is `ink`'s code.
    fn c(ink: Ink) -> Rgb {
        Rgb::from(ink.code())
    }

    /// The game's own font: a glyph's pixels are the one-bit atlas's.
    #[test]
    fn the_games_font_draws_its_glyphs() {
        let atlas = trpg_content::font::load().unwrap();
        let bits = trpg_content::bundle::bytes(trpg_content::font::ATLAS_1BIT_PATH).unwrap();
        let font = Font::new(&atlas, bits).unwrap();
        let mut b = GlyphBuffer::with_spec(
            ConsoleSpec::PLAYDATE,
            Cell::new(' ', c(Ink::Black), c(Ink::White)),
        );
        b.print(49, 14, "A", c(Ink::Black), c(Ink::White));
        let mut frame = Box::new([0u8; FRAME_BYTES]);
        render(&b, &font, &(), &mut frame);
        let rows = font.rows('A').unwrap();
        for (y, &row) in (224..).zip(rows) {
            for x in 0..8 {
                let ink = row >> (7 - x) & 1 == 1;
                assert_eq!(is_white(&frame, 392 + x, y), !ink, "({x}, {y})");
            }
        }
        assert!(rows.iter().any(|&row| row != 0));
    }
}
