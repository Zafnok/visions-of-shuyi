//! The Playdate's palette and its pattern map (ADR-0074, engine ADR-0026):
//! the palette (`assets/data/palettes/playdate.ron`) gives each colour
//! name an RGB colour, as any palette does, and the pattern map beside it
//! (`playdate@onebit.ron`) says how each role is drawn in one bit. Roles
//! drawn alike share a colour.
use crate::prelude::*;

pub use shuyi_assets::onebit::PatternMapDef;

use crate::bundle;
use crate::error::ContentError;
use crate::palette::PaletteDef;

/// Path of the Playdate's palette inside the asset bundle.
pub const PLAYDATE_PALETTE_PATH: &str = "data/palettes/playdate.ron";

/// Path of the Playdate's pattern map inside the asset bundle, beside its
/// palette (`shuyi_assets::onebit::map_path`).
pub const PLAYDATE_MAP_PATH: &str = "data/palettes/playdate@onebit.ron";

/// The file at `path` in the asset bundle, with its name as shown.
fn file(path: &str) -> Result<(String, &'static str), Vec<ContentError>> {
    let display = bundle::display_path(path);
    let source = bundle::file(path).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    Ok((display, source))
}

/// Loads the Playdate's pattern map: how each colour of its palette is
/// drawn in one bit.
pub fn load_playdate_map() -> Result<PatternMapDef, Vec<ContentError>> {
    let (display, source) = file(PLAYDATE_MAP_PATH)?;
    PatternMapDef::from_source(&display, source)
}

/// Loads the Playdate's palette. It must have every colour of `base` (the
/// default theme's palette), so nothing drawn with it lacks a colour; it
/// may have names of its own for the Playdate's skins.
pub fn load_playdate(base: &PaletteDef) -> Result<PaletteDef, Vec<ContentError>> {
    let (display, source) = file(PLAYDATE_PALETTE_PATH)?;
    let required: Vec<&str> = base.colors.keys().map(String::as_str).collect();
    PaletteDef::from_source(&display, source, &required)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded Playdate palette has every colour of the default
    /// theme; the ground is white and the glyphs black
    /// (`docs/design/playdate.md`).
    #[test]
    fn the_playdate_palette_loads() {
        let base = crate::palette::load().unwrap_or_default();
        let p = load_playdate(&base);
        assert!(p.is_ok(), "{p:?}");
        let p = p.unwrap_or_default();
        assert!(base.colors.keys().all(|n| p.colors.contains_key(n)));
        let map = load_playdate_map().unwrap_or_default();
        let fg = |role: &str| map.get(role).map(|r| r.fg);
        assert_eq!(fg("black"), Some(shuyi_assets::onebit::Pattern::WHITE));
        assert_eq!(fg("text"), Some(shuyi_assets::onebit::Pattern::BLACK));
        assert!(map.get("cursor").is_some_and(|r| r.invert));
    }

    /// Every role of the Playdate's palette has a row in the pattern map,
    /// and the map names no other role, so the two files can't drift.
    #[test]
    fn the_pattern_map_covers_every_role() {
        let base = crate::palette::load().unwrap_or_default();
        let palette = load_playdate(&base).unwrap_or_default();
        let map = load_playdate_map();
        assert!(map.is_ok(), "{map:?}");
        let map = map.unwrap_or_default();
        for role in palette.colors.keys() {
            assert!(map.get(role).is_some(), "{role}");
        }
        assert_eq!(map.roles.len(), palette.colors.len());
        assert_eq!(shuyi_assets::onebit::check(&palette, &map), Vec::new());
        assert_eq!(
            shuyi_assets::onebit::map_path(PLAYDATE_PALETTE_PATH),
            PLAYDATE_MAP_PATH
        );
    }

    /// A colour of the default theme missing from the Playdate's palette
    /// is an error naming it.
    #[test]
    fn a_missing_colour_is_an_error() {
        let mut base = crate::palette::load().unwrap_or_default();
        base.colors
            .insert("not_on_the_playdate".to_owned(), [9, 9, 9]);
        let errors = load_playdate(&base).err().unwrap_or_default();
        let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(errors.len(), 1, "{text:?}");
        assert!(text[0].contains("not_on_the_playdate"), "{text:?}");
    }
}
