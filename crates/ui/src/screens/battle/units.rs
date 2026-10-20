//! What the battle screen's panels and the map skins share about a unit's
//! look: its faction's colour and how full an HP bar is.

use trpg_core::{Faction, StatValue};

use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};

/// The colour of `faction`: player blue, enemy red, ally green, neutral
/// yellow.
pub const fn faction_color(faction: Faction) -> UiColor {
    match faction {
        Faction::Player => UiColor::Player,
        Faction::Enemy => UiColor::Enemy,
        Faction::Ally => UiColor::Ally,
        Faction::Neutral => UiColor::Neutral,
    }
}

/// How much of a `full`-long HP bar is filled (`round(full × hp / max)`, in
/// `0..=full`) and its colour: `hp_high` above 2/3, `hp_mid` above 1/3, else
/// `hp_low`. HP is clamped to `0..=max`; a non-positive `max` gives an empty
/// bar. Shared by the map's pixel bar and the side panel's cell bar.
pub fn hp_fill(hp: StatValue, max: StatValue, full: i32) -> (i32, UiColor) {
    if max <= 0 {
        return (0, UiColor::HpLow);
    }
    let (hp, max) = (i64::from(hp.clamp(0, max)), i64::from(max));
    let full = i64::from(full.max(0));
    // Round half up, in integers.
    let width = (2 * full * hp + max) / (2 * max);
    let color = if 3 * hp > 2 * max {
        UiColor::HpHigh
    } else if 3 * hp > max {
        UiColor::HpMid
    } else {
        UiColor::HpLow
    };
    (i32::try_from(width).unwrap_or(0), color)
}

/// A bar across `rect` with its first `filled` cells `full` and the rest
/// `empty`: the engine's [`GlyphBuffer::meter`] given cells already
/// rounded, so each bar keeps its own rule (an HP bar, [`hp_fill`], rounds
/// half up and shows 0.4 of a cell as empty; an EXP bar rounds down, so
/// 99 of 100 is never full).
pub fn meter_cells(buf: &mut GlyphBuffer, rect: Rect, filled: i32, full: Cell, empty: Cell) {
    let n = |cells: i32| u32::try_from(cells).unwrap_or(0);
    buf.meter(rect, n(filled), n(rect.w), full, empty);
}

/// The HP bar of `hp` of `max` across `rect` on `bg`: `█` in its HP
/// colour ([`hp_fill`]), the rest `░` dim.
pub fn paint_hp_bar(
    buf: &mut GlyphBuffer,
    palette: &Palette,
    rect: Rect,
    (hp, max): (StatValue, StatValue),
    bg: Rgb,
) {
    let (filled, color) = hp_fill(hp, max, rect.w);
    let full = Cell::new('█', palette.get(color), bg);
    let empty = Cell::new('░', palette.get(UiColor::TextDim), bg);
    meter_cells(buf, rect, filled, full, empty);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_colours() {
        assert_eq!(faction_color(Faction::Player), UiColor::Player);
        assert_eq!(faction_color(Faction::Enemy), UiColor::Enemy);
        assert_eq!(faction_color(Faction::Ally), UiColor::Ally);
        assert_eq!(faction_color(Faction::Neutral), UiColor::Neutral);
    }

    #[test]
    fn hp_fill_at_the_thresholds() {
        use UiColor::{HpHigh, HpLow, HpMid};
        let hp_bar = |hp, max| hp_fill(hp, max, 16);
        assert_eq!(hp_bar(30, 30), (16, HpHigh));
        assert_eq!(hp_bar(21, 30), (11, HpHigh)); // 11.2
        assert_eq!(hp_bar(20, 30), (11, HpMid)); // exactly 2/3: 10.67
        assert_eq!(hp_bar(11, 30), (6, HpMid)); // 5.87
        assert_eq!(hp_bar(10, 30), (5, HpLow)); // exactly 1/3: 5.33
        assert_eq!(hp_bar(1, 30), (1, HpLow)); // 0.53 rounds up
        assert_eq!(hp_bar(1, 40), (0, HpLow)); // 0.4 rounds down
        assert_eq!(hp_bar(0, 30), (0, HpLow));
        assert_eq!(hp_bar(1, 32), (1, HpLow)); // exactly 0.5 rounds up
        assert_eq!(hp_bar(-5, 30), (0, HpLow));
        assert_eq!(hp_bar(99, 30), (16, HpHigh));
        assert_eq!(hp_bar(5, 0), (0, HpLow));
        assert_eq!(hp_bar(5, -3), (0, HpLow));
    }
}
