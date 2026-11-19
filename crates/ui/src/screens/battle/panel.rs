//! The side panel's hover info (ADR-0018, `docs/design/look-and-feel.md`):
//! the terrain under the cursor, then the unit on it, if any: its block,
//! then its stats, weapon and combat numbers (ticket 0484). Stats only, no
//! portrait (Nick, ticket 0011). This says what it shows ([`view`]); the
//! battle screen's skin paints it.

use crate::prelude::*;
use trpg_core::{BattleState, Faction, Pos, StatKind, StatValue, Unit};

use super::art_list;
use super::info::{class_level, stat_name};
use super::shop::gold_text;
use super::view::{PanelCellView, PanelUnitView, PanelView, TerrainView};
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
/// there, if any); `selected` while a unit is selected; with the unit's
/// stats, weapon and combat numbers if `numbers` (the Playdate's corner
/// window shows none: they cost its frame time for nothing).
pub fn view(
    (state, words): (&BattleState, Words<'_>),
    pos: Pos,
    unit: Option<&Unit>,
    (selected, numbers): (bool, bool),
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
        unit: unit.map(|u| unit_view((state, words), u, numbers)),
        gold: words.text_with("battle.shop.gold", &[("gold", &gold_text(state.gold()))]),
        preview: None,
    }
}

/// The unit block: name, class and level, HP, faction; then its stats,
/// weapon and combat numbers (0484, `look-and-feel.md` *Side panel*).
fn unit_view(
    (state, words): (&BattleState, Words<'_>),
    unit: &Unit,
    numbers: bool,
) -> PanelUnitView {
    let class = words.class_of(&unit.class, state.classes());
    let mut view = PanelUnitView {
        name: words.unit(unit).to_owned(),
        faction: unit.faction,
        class_level: class_level(words, class, unit.level),
        hp_name: stat_name(words, StatKind::Hp).to_owned(),
        hp: unit.hp,
        max: unit.stats.hp,
        hp_then: None,
        faction_name: faction_name(words, unit.faction).to_owned(),
        stats: Vec::new(),
        weapon: None,
        no_weapon: String::new(),
        combat: Vec::new(),
    };
    if numbers {
        add_numbers((state, words), unit, &mut view);
    }
    view
}

/// Adds `unit`'s stats, weapon and combat numbers to `view`.
fn add_numbers((state, words): (&BattleState, Words<'_>), unit: &Unit, view: &mut PanelUnitView) {
    let card = state.stat_card(unit);
    let none = words.text("battle.panel.none");
    let stat_cells = PANEL_STATS
        .iter()
        .map(|&kind| {
            let (value, changed) = card.map_or((unit.stats.get(kind), false), |c| {
                (
                    c.stats.get(kind),
                    c.stats.get(kind) != c.without_effects.get(kind),
                )
            });
            PanelCellView {
                name: stat_name(words, kind).to_owned(),
                value: value.to_string(),
                changed,
            }
        })
        .collect();
    let numbers = card.map(|c| c.numbers);
    let shown = |n: Option<StatValue>| n.map_or_else(|| none.to_owned(), |n| n.to_string());
    let combat = [
        ("battle.panel.atk", numbers.and_then(|n| n.atk)),
        ("battle.panel.hit", numbers.and_then(|n| n.hit)),
        ("battle.panel.crit", numbers.and_then(|n| n.crit)),
        ("battle.panel.avo", numbers.map(|n| n.avoid)),
        ("battle.panel.as", numbers.map(|n| n.attack_speed)),
    ]
    .into_iter()
    .map(|(key, n)| PanelCellView {
        name: words.text(key).to_owned(),
        value: shown(n),
        changed: false,
    })
    .collect();
    // The weapon: none when the card says it can't attack (disarmed).
    let weapon = unit
        .loadout
        .equipped
        .as_ref()
        .filter(|_| numbers.is_some_and(|n| n.atk.is_some()))
        .and_then(|with| art_list::unit_header((state, words), unit, with));
    view.stats = stat_cells;
    view.weapon = weapon;
    none.clone_into(&mut view.no_weapon);
    view.combat = combat;
}

/// The stats the panel lists, in order.
const PANEL_STATS: [StatKind; 7] = [
    StatKind::Str,
    StatKind::Mag,
    StatKind::Dex,
    StatKind::Spd,
    StatKind::Def,
    StatKind::Res,
    StatKind::Mov,
];

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
