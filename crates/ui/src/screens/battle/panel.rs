//! The side panel's hover info (ADR-0018, `docs/design/look-and-feel.md`):
//! the terrain under the cursor, then the unit on it, if any. Stats only, no
//! portrait (Nick, ticket 0011).

use trpg_core::{BattleState, Faction, Pos, StatKind, Unit};

use super::info::{class_level, stat_name};
use super::layout::SIDE_PANEL;
use super::units::{faction_color, hp_fill};
use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::words::Words;

/// Length of the panel's HP bar, in cells.
pub const HP_BAR_CELLS: i32 = 10;

/// First text column: inside the border, one blank column in.
pub const TEXT_X: i32 = SIDE_PANEL.x + 2;

/// Widest line of text, so nothing reaches the right border.
pub const TEXT_W: usize = 26;

/// Row of the terrain name; its bonuses follow.
pub const TERRAIN_ROW: i32 = SIDE_PANEL.y + 1;

/// Row of the unit's name; the rest of the unit block follows.
pub const UNIT_ROW: i32 = SIDE_PANEL.y + 5;

/// Column of the HP bar, after `HP 999/999 `.
pub const HP_BAR_X: i32 = TEXT_X + 11;

/// The name players see for `faction`.
pub fn faction_name(words: Words<'_>, faction: Faction) -> &str {
    words.text(match faction {
        Faction::Player => "battle.faction.player",
        Faction::Enemy => "battle.faction.enemy",
        Faction::Ally => "battle.faction.ally",
        Faction::Neutral => "battle.faction.neutral",
    })
}

/// `+n` / `-n`, as bonuses are shown.
fn signed(n: i8) -> String {
    format!("{n:+}")
}

/// Prints `text` at the panel's text column on row `y`, cut to [`TEXT_W`].
fn line(buf: &mut GlyphBuffer, y: i32, text: &str, fg: Rgb) {
    let bg = buf.get(TEXT_X, y).map_or(fg, |c| c.bg);
    let cut: String = text.chars().take(TEXT_W).collect();
    buf.print(TEXT_X, y, &cut, fg, bg);
}

/// Draws the terrain of `pos` and `unit` (the unit shown there, if any) into
/// the (already cleared) panel.
pub fn draw_hover(
    buf: &mut GlyphBuffer,
    palette: &Palette,
    (state, words): (&BattleState, Words<'_>),
    pos: Pos,
    unit: Option<&Unit>,
) {
    let c = |u| palette.get(u);
    if let Some(t) = state
        .map()
        .tiles
        .get(pos)
        .and_then(|&id| state.terrain().get(id))
    {
        line(
            buf,
            TERRAIN_ROW,
            words.terrain(t),
            c(UiColor::TextHighlight),
        );
        let bonuses = words.text_with(
            "battle.panel.terrain",
            &[("defense", &signed(t.defense)), ("avoid", &signed(t.avoid))],
        );
        line(buf, TERRAIN_ROW + 1, &bonuses, c(UiColor::Text));
        if t.heal_percent > 0 {
            let heal = words.text_with("battle.panel.heals", &[("percent", &t.heal_percent)]);
            line(buf, TERRAIN_ROW + 2, &heal, c(UiColor::Text));
        }
    }
    if let Some(unit) = unit {
        draw_unit(buf, palette, (state, words), unit);
    }
}

/// The unit block: name (faction colour), class and level, HP with a bar,
/// faction.
fn draw_unit(
    buf: &mut GlyphBuffer,
    palette: &Palette,
    (state, words): (&BattleState, Words<'_>),
    unit: &Unit,
) {
    let c = |u| palette.get(u);
    let y = UNIT_ROW;
    line(buf, y, words.unit(unit), c(faction_color(unit.faction)));
    let class = words.class_of(&unit.class, state.classes());
    let class_line = class_level(words, class, unit.level);
    line(buf, y + 1, &class_line, c(UiColor::Text));
    let hp_name = stat_name(words, StatKind::Hp);
    let hp = format!("{hp_name} {}/{}", unit.hp, unit.stats.hp);
    line(buf, y + 2, &hp, c(UiColor::Text));
    let (filled, color) = hp_fill(unit.hp, unit.stats.hp, HP_BAR_CELLS);
    for i in 0..HP_BAR_CELLS {
        let (glyph, fg) = if i < filled {
            ('█', c(color))
        } else {
            ('░', c(UiColor::TextDim))
        };
        let bg = c(UiColor::PanelBg);
        buf.set(HP_BAR_X + i, y + 2, Cell::new(glyph, fg, bg));
    }
    let faction = faction_name(words, unit.faction);
    line(buf, y + 3, faction, c(UiColor::TextDim));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_names_and_signs() {
        assert_eq!(
            [
                Faction::Player,
                Faction::Enemy,
                Faction::Ally,
                Faction::Neutral
            ]
            .map(|faction| faction_name(Words::ENGLISH, faction)),
            ["Player", "Enemy", "Ally", "Neutral"]
        );
        assert_eq!(
            (signed(0), signed(2), signed(-1)),
            ("+0".into(), "+2".into(), "-1".into())
        );
    }

    #[test]
    fn text_fits_inside_the_border() {
        assert_eq!(
            TEXT_X + i32::try_from(TEXT_W).unwrap(),
            SIDE_PANEL.x + SIDE_PANEL.w - 2
        );
        // The bar ends well inside the border (column 99).
        assert_eq!(HP_BAR_X + HP_BAR_CELLS, 93);
    }
}
