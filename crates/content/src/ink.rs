//! The Playdate's one-bit inks and its palette (ADR-0074): the
//! Playdate's palette file (`assets/data/palettes/playdate.ron`) gives
//! each colour name an [`Ink`] instead of an RGB value. It loads as a
//! [`PaletteDef`] like any other, each colour being its ink's reserved
//! [`code`](Ink::code), so the screens and skins draw with it unchanged
//! and the one-bit renderer (`trpg_ui::onebit`) reads the codes back as
//! inks. Inks are Shuyi Engine's (`shuyi_assets::ink`, ticket 1104); this
//! module names the game's file.
//!
//! The Playdate's build draws the colours by the palette's pattern map
//! (`playdate@onebit.ron`, engine ADR-0026, ticket 1253), which gives each
//! role its ink's patterns, not by reading the codes back as inks.
use crate::prelude::*;

pub use shuyi_assets::ink::Ink;
pub use shuyi_assets::onebit::PatternMapDef;

use crate::bundle;
use crate::error::ContentError;
use crate::palette::PaletteDef;

/// Path of the Playdate's palette inside the asset bundle.
pub const PLAYDATE_PALETTE_PATH: &str = "data/palettes/playdate.ron";

/// Path of the Playdate's pattern map inside the asset bundle, beside its
/// palette (`shuyi_assets::onebit::map_path`).
pub const PLAYDATE_MAP_PATH: &str = "data/palettes/playdate@onebit.ron";

/// Loads the Playdate's pattern map: how each colour of its palette is
/// drawn in one bit.
pub fn load_playdate_map() -> Result<PatternMapDef, Vec<ContentError>> {
    let display = bundle::display_path(PLAYDATE_MAP_PATH);
    let source = bundle::file(PLAYDATE_MAP_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    PatternMapDef::from_source(&display, source)
}

/// Loads the Playdate's palette. It must name an ink for every colour of
/// `base` (the default theme's palette), so nothing drawn with it lacks a
/// colour; it may have names of its own for the Playdate's skins.
pub fn load_playdate(base: &PaletteDef) -> Result<PaletteDef, Vec<ContentError>> {
    let display = bundle::display_path(PLAYDATE_PALETTE_PATH);
    let source = bundle::file(PLAYDATE_PALETTE_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    PaletteDef::from_ink_source(&display, source, base)
}

#[cfg(test)]
mod tests {
    use shuyi_assets::onebit::{Base, Pattern, RolePatterns};

    use super::*;

    /// The embedded Playdate palette names an ink for every colour of the
    /// default theme.
    #[test]
    fn the_playdate_palette_loads() {
        let base = crate::palette::load().unwrap_or_default();
        let p = load_playdate(&base);
        assert!(p.is_ok(), "{p:?}");
        let p = p.unwrap_or_default();
        assert!(base.colors.keys().all(|n| p.colors.contains_key(n)));
        assert!(p.colors.values().all(|&c| Ink::from_code(c).is_some()));
        // The ground is white and the glyphs black (`docs/design/playdate.md`).
        assert_eq!(p.get("black"), Some(Ink::White.code()));
        assert_eq!(p.get("text"), Some(Ink::Black.code()));
        assert_eq!(p.get("cursor"), Some(Ink::Invert.code()));
    }

    /// The patterns that draw as `ink` does (engine ADR-0089's table).
    fn patterns_of(ink: Ink) -> RolePatterns {
        let of = |base| RolePatterns::of(Pattern::new(base));
        match ink {
            Ink::White => of(Base::White),
            Ink::Black => of(Base::Black),
            Ink::BlackEdged => of(Base::Black).with_edge(true),
            Ink::Grey => of(Base::Dither50),
            Ink::Dots => of(Base::Dots),
            Ink::Diagonals => of(Base::DiagonalDown),
            Ink::Dashes => of(Base::Checker2),
            Ink::Invert => of(Base::Black).with_invert(true),
        }
    }

    /// Every role of the Playdate's palette has its ink's patterns in the
    /// pattern map, and the map names no other role, so the two files
    /// can't drift.
    #[test]
    fn the_pattern_map_gives_each_role_its_inks_patterns() {
        let base = crate::palette::load().unwrap_or_default();
        let palette = load_playdate(&base).unwrap_or_default();
        let map = load_playdate_map();
        assert!(map.is_ok(), "{map:?}");
        let map = map.unwrap_or_default();
        for (role, &code) in &palette.colors {
            let ink = Ink::from_code(code);
            assert!(ink.is_some(), "{role}");
            let want = ink.map(patterns_of);
            assert_eq!(map.get(role).copied(), want, "{role}: {ink:?}");
        }
        assert_eq!(map.roles.len(), palette.colors.len());
        assert_eq!(shuyi_assets::onebit::check(&palette, &map), Vec::new());
        assert_eq!(
            shuyi_assets::onebit::map_path(PLAYDATE_PALETTE_PATH),
            PLAYDATE_MAP_PATH
        );
    }
}
