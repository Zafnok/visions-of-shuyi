//! The Playdate's one-bit inks and its palette (ADR-0074): the
//! Playdate's palette file (`assets/data/palettes/playdate.ron`) gives
//! each colour name an [`Ink`] instead of an RGB value. It loads as a
//! [`PaletteDef`] like any other, each colour being its ink's reserved
//! [`code`](Ink::code), so the screens and skins draw with it unchanged
//! and the one-bit renderer (`trpg_ui::onebit`) reads the codes back as
//! inks.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use crate::bundle;
use crate::error::ContentError;
use crate::palette::PaletteDef;
use crate::ron_loader::parse_ron;

/// Path of the Playdate's palette inside the asset bundle.
pub const PLAYDATE_PALETTE_PATH: &str = "data/palettes/playdate.ron";

/// How a one-bit pixel is coloured (ADR-0074, `docs/design/playdate.md`,
/// *Telling things apart without colour*). The patterns are fixed to the
/// screen's pixels, so neighbouring tiles line up.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Ink {
    /// White: the ground.
    White,
    /// Black.
    Black,
    /// Black, and on a glyph a white edge one pixel wide around it, so it
    /// reads over a pattern. Anything else in it is plain black.
    BlackEdged,
    /// A 50% checker (enemies' tiles).
    Grey,
    /// Light dots, one pixel in eight (the move range).
    Dots,
    /// Diagonal lines, one pixel in four (the attack range).
    Diagonals,
    /// Dashes two pixels long (the danger zone's edge): a 1-pixel line of
    /// it is a dashed line.
    Dashes,
    /// Flips the pixels under it (the cursor's corner marks). Only a
    /// rectangle item flips; anything else in it is plain black.
    Invert,
}

impl Ink {
    /// Every ink.
    pub const ALL: [Self; 8] = [
        Self::White,
        Self::Black,
        Self::BlackEdged,
        Self::Grey,
        Self::Dots,
        Self::Diagonals,
        Self::Dashes,
        Self::Invert,
    ];

    /// Its name in the Playdate's palette file.
    pub const fn name(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Black => "black",
            Self::BlackEdged => "black_edged",
            Self::Grey => "grey",
            Self::Dots => "dots",
            Self::Diagonals => "diagonals",
            Self::Dashes => "dashes",
            Self::Invert => "invert",
        }
    }

    /// The ink named `name`.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|ink| ink.name() == name)
    }

    /// The colour reserved for it: a cell or item in this colour is drawn
    /// in this ink. Each is about as dark as the ink, so a frame drawn in
    /// colour (`cargo xtask frame-png` without `--playdate`) still reads.
    pub const fn code(self) -> [u8; 3] {
        match self {
            Self::White => [0xff, 0xff, 0xff],
            Self::Black => [0x00, 0x00, 0x00],
            Self::BlackEdged => [0x01, 0x01, 0x01],
            Self::Grey => [0x80, 0x80, 0x80],
            Self::Dots => [0xe0, 0xe0, 0xe0],
            Self::Diagonals => [0xc0, 0xc0, 0xc0],
            Self::Dashes => [0x40, 0x40, 0x40],
            Self::Invert => [0xff, 0x00, 0xff],
        }
    }

    /// The ink whose [`code`](Self::code) is `rgb`, if any.
    pub fn from_code(rgb: [u8; 3]) -> Option<Self> {
        Self::ALL.into_iter().find(|ink| ink.code() == rgb)
    }
}

impl PaletteDef {
    /// Loads the Playdate's palette. It must name an ink for every colour
    /// of `base` (the default theme's palette), so nothing drawn with it
    /// lacks a colour; it may have names of its own for the Playdate's
    /// skins.
    pub fn load_playdate(base: &PaletteDef) -> Result<Self, Vec<ContentError>> {
        let display = bundle::display_path(PLAYDATE_PALETTE_PATH);
        let source = bundle::file(PLAYDATE_PALETTE_PATH).ok_or_else(|| {
            vec![ContentError::new(
                &display,
                "file not found in asset bundle",
            )]
        })?;
        Self::from_ink_source(&display, source, base)
    }

    /// Parses palette `source` of ink names (errors attributed to `file`)
    /// into each ink's code. Reports every unknown ink and every colour of
    /// `base` it lacks.
    pub fn from_ink_source(
        file: &str,
        source: &str,
        base: &PaletteDef,
    ) -> Result<Self, Vec<ContentError>> {
        let raw: BTreeMap<String, String> = parse_ron(file, source).map_err(|e| vec![e])?;
        let mut errors = Vec::new();
        let mut colors = BTreeMap::new();
        for (name, value) in &raw {
            if let Some(ink) = Ink::parse(value) {
                colors.insert(name.clone(), ink.code());
            } else {
                let inks: Vec<&str> = Ink::ALL.iter().map(|ink| ink.name()).collect();
                let mut e = ContentError::new(
                    file,
                    format!(
                        "colour \"{name}\": no ink \"{value}\"; the inks are {}",
                        inks.join(", ")
                    ),
                );
                if let Some(line) = crate::palette::line_of_key(source, name) {
                    e = e.at(line, None);
                }
                errors.push(e);
            }
        }
        for name in base.colors.keys().filter(|n| !raw.contains_key(*n)) {
            errors.push(ContentError::new(
                file,
                format!("missing colour \"{name}\""),
            ));
        }
        if errors.is_empty() {
            Ok(Self { colors })
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(names: &[&str]) -> PaletteDef {
        PaletteDef {
            colors: names.iter().map(|&n| (n.to_owned(), [9, 9, 9])).collect(),
        }
    }

    #[test]
    fn every_ink_has_its_own_name_and_code() {
        for ink in Ink::ALL {
            assert_eq!(Ink::parse(ink.name()), Some(ink));
            assert_eq!(Ink::from_code(ink.code()), Some(ink));
        }
        assert_eq!(Ink::parse("gray"), None);
        assert_eq!(Ink::from_code([0x80, 0x80, 0x81]), None);
        assert_eq!(Ink::Grey.code(), [0x80; 3]);
        assert_eq!(Ink::Invert.name(), "invert");
    }

    #[test]
    fn a_palette_of_inks_loads_as_their_codes() {
        let src =
            "{\n    \"text\": \"black\",\n    \"enemy\": \"grey\",\n    \"own\": \"dots\",\n}";
        let p = PaletteDef::from_ink_source("p.ron", src, &base(&["text", "enemy"]));
        let p = p.unwrap_or_default();
        assert_eq!(p.get("text"), Some(Ink::Black.code()));
        assert_eq!(p.get("enemy"), Some(Ink::Grey.code()));
        // A name of its own, for the Playdate's skins.
        assert_eq!(p.get("own"), Some(Ink::Dots.code()));
        assert_eq!(p.colors.len(), 3);
    }

    #[test]
    fn an_unknown_ink_and_a_missing_colour_are_errors() {
        let src = "{\n    \"text\": \"black\",\n    \"enemy\": \"#808080\",\n}";
        let errs = PaletteDef::from_ink_source("p.ron", src, &base(&["text", "enemy", "cursor"]))
            .err()
            .unwrap_or_default();
        let shown: Vec<String> = errs.iter().map(ToString::to_string).collect();
        assert_eq!(shown.len(), 2, "{shown:?}");
        assert_eq!(errs[0].line, Some(3));
        assert!(
            shown[0].contains("colour \"enemy\": no ink \"#808080\"; the inks are white, black,"),
            "{}",
            shown[0]
        );
        assert_eq!(shown[1], "p.ron: missing colour \"cursor\"");
        assert!(PaletteDef::from_ink_source("p.ron", "nope", &base(&[])).is_err());
    }

    /// The embedded Playdate palette names an ink for every colour of the
    /// default theme.
    #[test]
    fn the_playdate_palette_loads() {
        let base = PaletteDef::load().unwrap_or_default();
        let p = PaletteDef::load_playdate(&base);
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
