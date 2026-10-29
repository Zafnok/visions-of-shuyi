//! The side panel's hover info (ADR-0018, `docs/design/look-and-feel.md`):
//! the terrain under the cursor, then the unit on it, if any. Stats only, no
//! portrait (Nick, ticket 0011). This says what it shows ([`view`]); the
//! battle screen's skin paints it.

use crate::prelude::*;
use trpg_core::{BattleState, Faction, Pos, StatKind, Unit};

use super::info::{class_level, stat_name};
use super::shop::gold_text;
use super::view::{PanelUnitView, PanelView, TerrainView};
use crate::words::Words;

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

/// What the panel shows of the terrain of `pos` and `unit` (the unit shown
/// there, if any); `selected` while a unit is selected.
pub fn view(
    (state, words): (&BattleState, Words<'_>),
    pos: Pos,
    unit: Option<&Unit>,
    selected: bool,
) -> PanelView {
    let terrain = state
        .map()
        .tiles
        .get(pos)
        .and_then(|&id| state.terrain().get(id))
        .map(|t| TerrainView {
            name: words.terrain(t).to_owned(),
            bonuses: words.text_with(
                "battle.panel.terrain",
                &[("defense", &signed(t.defense)), ("avoid", &signed(t.avoid))],
            ),
            heals: (t.heal_percent > 0)
                .then(|| words.text_with("battle.panel.heals", &[("percent", &t.heal_percent)])),
        });
    PanelView {
        selected,
        terrain,
        unit: unit.map(|u| unit_view((state, words), u)),
        gold: words.text_with("battle.shop.gold", &[("gold", &gold_text(state.gold()))]),
    }
}

/// The unit block: name, class and level, HP, faction.
fn unit_view((state, words): (&BattleState, Words<'_>), unit: &Unit) -> PanelUnitView {
    let class = words.class_of(&unit.class, state.classes());
    PanelUnitView {
        name: words.unit(unit).to_owned(),
        faction: unit.faction,
        class_level: class_level(words, class, unit.level),
        hp_name: stat_name(words, StatKind::Hp).to_owned(),
        hp: unit.hp,
        max: unit.stats.hp,
        faction_name: faction_name(words, unit.faction).to_owned(),
    }
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
}
