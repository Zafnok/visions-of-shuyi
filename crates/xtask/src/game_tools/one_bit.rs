//! The Playdate's one-bit look as `check --target` reads it (ticket 0140).
//!
//! The game draws with inks (ADR-0074), not the engine's pattern maps, so
//! the palette file stays the one place that says how a role looks: the
//! map here is derived from each role's ink, by the fixed pattern nearest
//! it. Only what the palette can't say is written here: the roles that
//! must stay apart. `text` stays empty: an ink has one pattern for both
//! ground and glyph, so no role can read "black on white", and the test
//! `the_text_roles_are_solid_black` keeps text undithered instead.

use shuyi_assets::onebit::{Base, Pattern, PatternMapDef, RolePatterns};
use shuyi_cli::OneBitSkin;
use trpg_content::ink::{Ink, PLAYDATE_PALETTE_PATH};
use trpg_content::palette::PaletteDef;

/// Where the map comes from, as the check's messages show it.
const MAP_FILE: &str = "crates/xtask/src/game_tools/one_bit.rs";

/// What must read apart on the Playdate's map (`docs/design/playdate.md`):
/// the three states of a unit's tile, and the ranges. The heal range isn't
/// here: it is the attack range's lines, as the palette file says.
const DISTINCT: [&[&str]; 2] = [
    &["map_player_tile", "map_enemy_tile", "map_acted_tile"],
    &["move_range", "attack_range", "danger_zone"],
];

/// The fixed pattern nearest `ink`. An edge or a flip is the renderer's
/// and no pattern says it: an edged black is black, and a flip is the one
/// pattern no other ink uses.
const fn pattern(ink: Ink) -> Pattern {
    let base = match ink {
        Ink::White => Base::White,
        Ink::Black | Ink::BlackEdged => Base::Black,
        Ink::Grey => Base::Dither50,
        Ink::Dots => Base::Dither25,
        Ink::Diagonals => Base::DiagonalUp,
        Ink::Dashes => Base::HLines,
        Ink::Invert => Base::Checker2,
    };
    Pattern::new(base)
}

/// The pattern map of `palette`, a palette of inks: a colour that is no
/// ink's code has no patterns, which the check reports.
fn map_of(palette: &PaletteDef) -> PatternMapDef {
    PatternMapDef {
        roles: palette
            .colors
            .iter()
            .filter_map(|(role, &rgb)| {
                let fg = pattern(Ink::from_code(rgb)?);
                Some((role.clone(), RolePatterns { fg, bg: None }))
            })
            .collect(),
        distinct: DISTINCT
            .iter()
            .map(|group| group.iter().map(|&role| role.to_owned()).collect())
            .collect(),
        text: Vec::new(),
    }
}

/// The Playdate's skin, `palette` being its palette.
pub(super) fn skin(palette: PaletteDef) -> OneBitSkin {
    let map = map_of(&palette);
    let file = trpg_content::bundle::display_path(PLAYDATE_PALETTE_PATH);
    OneBitSkin::new(file, palette, MAP_FILE, map)
}
