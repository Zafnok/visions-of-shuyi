//! The one-bit renderer (ADR-0074) is Shuyi Engine's
//! ([`shuyi_ui::frame::onebit`], ticket 1108). The test here draws with
//! this game's one-bit font (`trpg_content::font::ATLAS_1BIT_PATH`).
//! [`playdate_map`] is how the Playdate's build draws its colours.

pub use shuyi_ui::frame::onebit::*;

use crate::prelude::*;

use crate::color::Rgb;

/// The Playdate's pattern map (ticket 1253): its palette's roles
/// (`trpg_content::playdate_palette::PLAYDATE_PALETTE_PATH`) drawn by their patterns
/// in `trpg_content::playdate_palette::PLAYDATE_MAP_PATH`. The front end builds it once
/// and draws every frame with it ([`render`]). It reads the palette
/// alone, as the loading card needs it before the rest of the data: the
/// game checks the palette has every colour when it loads it.
pub fn playdate_map() -> Result<PatternMap, String> {
    let shown = |errors: Vec<trpg_content::ContentError>| {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    };
    let palette =
        trpg_content::playdate_palette::load_playdate(&trpg_content::PaletteDef::default())
            .map_err(shown)?;
    let map = trpg_content::playdate_palette::load_playdate_map().map_err(shown)?;
    let roles = palette
        .colors
        .iter()
        .map(|(role, &rgb)| (role.as_str(), Rgb::from(rgb)));
    Ok(PatternMap::new(roles, &map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::{Cell, GlyphBuffer};
    use crate::map_view::playdate::tests::c;

    /// The Playdate's map draws each role's colour by the role's
    /// patterns: its ground, a glyph's edge and a shape's flip.
    #[test]
    fn the_playdate_map_draws_each_role_by_its_patterns() {
        let map = playdate_map().unwrap();
        let def = trpg_content::playdate_palette::load_playdate_map().unwrap();
        for (role, p) in &def.roles {
            let rgb = c(role);
            assert!(map.role_of(rgb).is_some(), "{role}");
            // Only a glyph is edged, and only a shape flips.
            assert_eq!(map.ground(rgb).rows, p.background().rows(), "{role}");
            let glyph = map.glyph(rgb);
            assert_eq!((glyph.rows, glyph.edged), (p.fg.rows(), p.edge), "{role}");
            let shape = map.shape(rgb);
            assert_eq!((shape.rows, shape.flips), (p.fg.rows(), p.invert), "{role}");
        }
        assert_eq!(map.role_of(c("cursor")), Some("cursor"));
    }

    /// The game's own font: a glyph's pixels are the one-bit atlas's.
    #[test]
    fn the_games_font_draws_its_glyphs() {
        let atlas = trpg_content::font::load().unwrap();
        let bits = trpg_content::bundle::bytes(trpg_content::font::ATLAS_1BIT_PATH).unwrap();
        let font = Font::new(&atlas, bits).unwrap();
        let mut b = GlyphBuffer::with_spec(
            ConsoleSpec::PLAYDATE,
            Cell::new(' ', c("text"), c("panel_bg")),
        );
        b.print(49, 14, "A", c("text"), c("panel_bg"));
        let mut frame = Box::new([0u8; FRAME_BYTES]);
        render(&b, &font, &(), &playdate_map().unwrap(), &mut frame);
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
