//! The attack forecast (ticket 0404, `docs/design/look-and-feel.md` →
//! *Attack forecast*): while a target is chosen it replaces the side panel.
//! Both sides' names, weapons, HP (and what each would have left if every
//! strike hit), hit and crit; then every strike in the order it happens,
//! the strike that kills, and each side's total. A spell's element against
//! the target's affinity (0410) is marked: as effective on the strikes for
//! Weak (as any effective attack), `(resist)` or `heals N` for the
//! striker. All numbers come from [`AttackPreview`]; nothing is computed
//! here. [`view`] says what it shows; the battle skin paints it
//! (ADR-0054).

use crate::prelude::*;
use trpg_core::{
    Affinity, ArtNote, AttackPreview, BattleState, Equipped, PlannedStrike, Side, SideForecast,
    StatKind, StatValue, Unit,
};

use super::art_list::{art_name, note_text};
use super::attack::{Targeting, spell_uses, weapon_durability};
use super::info::stat_name;
use super::skills::skill_name;
use super::view::{
    ForecastHp, ForecastSideView, ForecastView, LeftView, SkillLineView, StrikeView, TotalView,
};
use crate::words::Words;

/// What is left of what a side fights with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Left {
    /// A weapon's durability left and max (broken at 0).
    Durability(u32, u32),
    /// A spell's uses left and per battle.
    Uses(u32, u32),
}

/// What `unit` fights with when it uses `with` (a weapon's slot or a
/// spell; nothing for `None`): its name and what is left of it.
fn arms(
    (state, words): (&BattleState, Words<'_>),
    unit: &Unit,
    with: Option<&Equipped>,
) -> (String, Option<Left>) {
    let found = match with {
        Some(Equipped::Weapon(slot)) => weapon_durability(state, words, unit.id, *slot)
            .map(|(name, left, max)| (name, Left::Durability(left, max))),
        Some(Equipped::Spell(id)) => spell_uses(state, words, unit.id, id)
            .map(|(name, left, max)| (name, Left::Uses(left, max))),
        None => None,
    };
    found.map_or((String::new(), None), |(name, left)| (name, Some(left)))
}

/// The marker for the target's affinity to a side's attack: `(resist)`, or
/// `heals 9` for Absorb (`n.damage` is then the HP a hit heals). Weak has
/// none of its own: its strikes are marked effective.
pub fn affinity_text(words: Words<'_>, n: &SideForecast) -> Option<String> {
    match n.affinity? {
        Affinity::Weak => None,
        Affinity::Resist => Some(words.text("battle.forecast.resist").to_owned()),
        Affinity::Absorb => Some(words.text_with("battle.forecast.heals", &[("n", &n.damage)])),
    }
}

/// What the forecast of `t` shows, or `None` if its attacker or target
/// is gone.
pub fn view((state, words): (&BattleState, Words<'_>), t: &Targeting) -> Option<ForecastView> {
    let attacker = state.unit(t.sel.unit)?;
    let target = state.unit(t.target())?;
    let p = &t.preview;
    let (weapon, left) = arms((state, words), attacker, Some(&t.with));
    let (counter, counter_left) = arms((state, words), target, target.loadout.equipped.as_ref());
    let attacker_side = Column {
        unit: attacker,
        weapon,
        left,
        numbers: Some(p.forecast.attacker),
        hp_after: p.plan.attacker_hp,
        by: Side::Attacker,
    };
    let target_side = Column {
        unit: target,
        weapon: counter,
        left: counter_left,
        numbers: p.forecast.defender,
        hp_after: p.plan.defender_hp,
        by: Side::Defender,
    };
    let no_counter = p
        .forecast
        .defender
        .is_none()
        .then(|| words.text("battle.forecast.no_counter").to_owned());
    Some(ForecastView {
        title: words.text("battle.forecast.title").to_owned(),
        skill: skill_line((state, words), p),
        sides: [
            side_view(words, p, attacker_side),
            side_view(words, p, target_side),
        ],
        hp_name: stat_name(words, StatKind::Hp).to_owned(),
        hit_name: words.text("battle.forecast.hit").to_owned(),
        crit_name: words.text("battle.forecast.crit").to_owned(),
        strikes: strikes(words, p),
        no_counter,
        total_name: words.text("battle.forecast.total").to_owned(),
        notes: notes(words, p),
        tiles: [t.sel.dest(), target.pos],
    })
}

/// The art's effects that aren't numbers (`pierces`, `pins: Mov −3`…).
/// `no counter` already shows in the target's column, so it isn't
/// repeated.
fn notes(words: Words<'_>, p: &AttackPreview) -> Vec<String> {
    p.notes
        .iter()
        .filter(|n| **n != ArtNote::NoCounter)
        .map(|n| note_text(words, n))
        .collect()
}

/// The chosen art's or combat active's line: its name and the weapon's
/// durability before and after.
fn skill_line(
    (state, words): (&BattleState, Words<'_>),
    p: &AttackPreview,
) -> Option<SkillLineView> {
    let name = match (&p.art, &p.active) {
        (Some(art), _) => art_name(state, words, art),
        (None, Some(skill)) => skill_name(state, words, skill),
        (None, None) => return None,
    };
    Some(SkillLineView {
        name,
        durability: p.durability,
    })
}

/// One side of the forecast, as [`side_view`] reads it.
struct Column<'a> {
    /// The unit.
    unit: &'a Unit,
    /// What it fights with.
    weapon: String,
    /// What is left of it.
    left: Option<Left>,
    /// Its numbers (`None`: it can't strike back).
    numbers: Option<SideForecast>,
    /// Its HP if every strike hit.
    hp_after: StatValue,
    /// Which side it is.
    by: Side,
}

/// One side's block: name, weapon or spell, its durability or uses (or
/// that it is broken), HP, hit, crit, affinity marker and total.
fn side_view(words: Words<'_>, p: &AttackPreview, side: Column<'_>) -> ForecastSideView {
    let (unit, numbers) = (side.unit, side.numbers);
    let broken =
        numbers.is_some_and(|n| n.broken) || matches!(side.left, Some(Left::Durability(0, _)));
    let left = if broken {
        Some(LeftView::Broken(
            words.text("battle.forecast.broken").to_owned(),
        ))
    } else {
        side.left.map(
            |(Left::Durability(left, max) | Left::Uses(left, max))| LeftView::Count { left, max },
        )
    };
    let total = numbers.map(|_| {
        let mine = p.plan.strikes.iter().filter(|s| s.by == side.by);
        TotalView {
            damage: mine.clone().map(|s| s.damage).sum(),
            strikes: mine.count(),
        }
    });
    ForecastSideView {
        name: words.unit(unit).to_owned(),
        faction: unit.faction,
        weapon: side.weapon,
        left,
        hp: ForecastHp {
            now: unit.hp,
            after: side.hp_after,
            max: unit.stats.hp,
        },
        hit: numbers.map(|n| n.hit),
        crit: numbers.map(|n| n.crit),
        affinity: numbers.as_ref().and_then(|n| affinity_text(words, n)),
        total,
    }
}

/// The strike list, in combat order: each strike's damage, whether it is
/// effective, kills, or comes after a kill.
fn strikes(words: Words<'_>, p: &AttackPreview) -> Vec<StrikeView> {
    p.plan
        .strikes
        .iter()
        .map(|s| {
            let numbers = match s.by {
                Side::Attacker => Some(p.forecast.attacker),
                Side::Defender => p.forecast.defender,
            };
            StrikeView {
                by: s.by,
                text: strike_text(words, s),
                effective: numbers.is_some_and(|f| f.effective),
                kills: s.kills(),
                after_a_fall: s.after_a_fall,
            }
        })
        .collect()
}

/// A strike's damage as listed: `8 dmg`, or `+9 hp` for an Absorb heal.
pub fn strike_text(words: Words<'_>, s: &PlannedStrike) -> String {
    if s.healed {
        words.text_with("battle.forecast.strike_heal", &[("n", &s.damage)])
    } else {
        let n = format!("{:>2}", s.damage);
        words.text_with("battle.forecast.strike", &[("n", &n)])
    }
}

#[cfg(test)]
pub(crate) mod tests;
