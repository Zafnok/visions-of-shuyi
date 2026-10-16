//! The Playdate's one-bit inks and its palette (ADR-0074): the
//! Playdate's palette file (`assets/data/palettes/playdate.ron`) gives
//! each colour name an [`Ink`] instead of an RGB value. It loads as a
//! [`PaletteDef`] like any other, each colour being its ink's reserved
//! [`code`](Ink::code), so the screens and skins draw with it unchanged
//! and the one-bit renderer (`trpg_ui::onebit`) reads the codes back as
//! inks. Inks are Shuyi Engine's (`shuyi_assets::ink`, ticket 1104); this
//! module names the game's file.
use crate::prelude::*;

pub use shuyi_assets::ink::Ink;

use crate::bundle;
use crate::error::ContentError;
use crate::palette::PaletteDef;

/// Path of the Playdate's palette inside the asset bundle.
pub const PLAYDATE_PALETTE_PATH: &str = "data/palettes/playdate.ron";

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
}
