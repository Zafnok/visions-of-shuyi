//! The Playdate's one-bit inks and its palette (ADR-0074): the
//! Playdate's palette file (`assets/data/palettes/playdate.ron`) gives
//! each colour name an [`Ink`] instead of an RGB value. It loads as a
//! [`PaletteDef`] like any other, each colour being its ink's reserved
//! [`code`](Ink::code), so the screens and skins draw with it unchanged.
//!
//! The Playdate's build draws the colours by the palette's pattern map
//! (`playdate@onebit.ron`, engine ADR-0026, ticket 1253), which gives each
//! role its ink's [`patterns`](Ink::patterns). The inks were Shuyi
//! Engine's (ticket 1104) until engine ADR-0107 removed them; they are
//! this game's palette format, so they live here now.
use crate::prelude::*;

use alloc::collections::BTreeMap;

pub use shuyi_assets::onebit::PatternMapDef;
use shuyi_assets::onebit::{Base, Pattern, RolePatterns};

use crate::bundle;
use crate::error::ContentError;
use crate::palette::PaletteDef;

/// Path of the Playdate's palette inside the asset bundle.
pub const PLAYDATE_PALETTE_PATH: &str = "data/palettes/playdate.ron";

/// Path of the Playdate's pattern map inside the asset bundle, beside its
/// palette (`shuyi_assets::onebit::map_path`).
pub const PLAYDATE_MAP_PATH: &str = "data/palettes/playdate@onebit.ron";

/// How a colour of the Playdate's palette is drawn in one bit.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Ink {
    /// White.
    White,
    /// Black.
    Black,
    /// Black, and a glyph gets a white edge.
    BlackEdged,
    /// A 50% checker.
    Grey,
    /// Sparse dots.
    Dots,
    /// Lines down to the right.
    Diagonals,
    /// A checker of 2 × 2 squares: a one-pixel line of it is dashed.
    Dashes,
    /// A rectangle flips the pixels under it.
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

    /// The ink's name in the palette file.
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

    /// The colour that stands for the ink in the loaded palette, one of
    /// its own.
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

    /// The ink whose code is `rgb`.
    pub fn from_code(rgb: [u8; 3]) -> Option<Self> {
        Self::ALL.into_iter().find(|ink| ink.code() == rgb)
    }

    /// The patterns that draw as the ink does (engine ADR-0089's table):
    /// what the pattern map gives the ink's roles.
    pub const fn patterns(self) -> RolePatterns {
        const fn of(base: Base) -> RolePatterns {
            RolePatterns::of(Pattern::new(base))
        }
        match self {
            Self::White => of(Base::White),
            Self::Black => of(Base::Black),
            Self::BlackEdged => of(Base::Black).with_edge(true),
            Self::Grey => of(Base::Dither50),
            Self::Dots => of(Base::Dots),
            Self::Diagonals => of(Base::DiagonalDown),
            Self::Dashes => of(Base::Checker2),
            Self::Invert => of(Base::Black).with_invert(true),
        }
    }

    /// Whether the ink makes screen pixel `(x, y)` white. `BlackEdged` and
    /// `Invert` are black here: the edge is drawn around glyphs, and only
    /// rectangles flip.
    pub fn is_white(self, x: i32, y: i32) -> bool {
        let rows = self.patterns().fg.rows();
        let row = rows[y.rem_euclid(8).unsigned_abs() as usize];
        row >> (7 - x.rem_euclid(8)) & 1 == 1
    }
}

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
    from_ink_source(&display, source, base)
}

/// The palette in `source`, a map from colour names to ink names, each
/// colour its ink's code; every name of `base` must be there.
fn from_ink_source(
    file: &str,
    source: &str,
    base: &PaletteDef,
) -> Result<PaletteDef, Vec<ContentError>> {
    let raw: BTreeMap<String, String> =
        shuyi_assets::parse_ron(file, source).map_err(|e| vec![e])?;
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
            if let Some(line) = shuyi_assets::palette::line_of_key(source, name) {
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
        Ok(PaletteDef { colors })
    } else {
        Err(errors)
    }
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
            let want = ink.map(Ink::patterns);
            assert_eq!(map.get(role).copied(), want, "{role}: {ink:?}");
        }
        assert_eq!(map.roles.len(), palette.colors.len());
        assert_eq!(shuyi_assets::onebit::check(&palette, &map), Vec::new());
        assert_eq!(
            shuyi_assets::onebit::map_path(PLAYDATE_PALETTE_PATH),
            PLAYDATE_MAP_PATH
        );
    }

    /// A misspelled ink and a missing colour are each an error naming it.
    #[test]
    fn a_wrong_ink_and_a_missing_colour_are_errors() {
        let base = PaletteDef {
            colors: [("a", [9, 9, 9]), ("b", [9, 9, 9])]
                .into_iter()
                .map(|(n, c)| (n.to_owned(), c))
                .collect(),
        };
        let errors = from_ink_source("p.ron", "{\n  \"a\": \"gray\",\n}", &base)
            .err()
            .unwrap_or_default();
        let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(errors.len(), 2, "{text:?}");
        assert!(text[0].contains("no ink \"gray\""), "{text:?}");
        assert!(text[1].contains("missing colour \"b\""), "{text:?}");
        let ok = from_ink_source("p.ron", r#"{"a": "dots", "b": "invert"}"#, &base);
        assert_eq!(ok.unwrap_or_default().get("b"), Some(Ink::Invert.code()));
    }

    /// Each ink has its own name and code, and parses back from both; the
    /// pixels are its pattern's.
    #[test]
    fn every_ink_has_its_own_name_and_code() {
        for ink in Ink::ALL {
            assert_eq!(Ink::parse(ink.name()), Some(ink));
            assert_eq!(Ink::from_code(ink.code()), Some(ink));
        }
        assert!(Ink::White.is_white(3, -5));
        assert!(!Ink::Black.is_white(3, 5));
        assert!(!Ink::Invert.is_white(3, 5));
        let grey = (0..8).filter(|&x| Ink::Grey.is_white(x, 0)).count();
        assert_eq!(grey, 4);
    }
}
