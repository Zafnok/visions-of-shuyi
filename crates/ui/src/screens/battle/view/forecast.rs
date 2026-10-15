//! What the attack forecast shows (ticket 0404, ADR-0054): both sides'
//! names, arms, HP, hit and crit, every strike in the order it happens and
//! each side's total, as numbers and flags. Built by
//! [`forecast::view`](super::super::forecast::view).

use trpg_core::{Faction, Side, StatValue};

/// The attack forecast, filling the side panel while a target is aimed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForecastView {
    /// Its title.
    pub title: String,
    /// The chosen art's or combat active's line, if one is chosen.
    pub skill: Option<SkillLineView>,
    /// The attacker, then the target.
    pub sides: [ForecastSideView; 2],
    /// The name of the HP stat.
    pub hp_name: String,
    /// What the hit chance is called.
    pub hit_name: String,
    /// What the crit chance is called.
    pub crit_name: String,
    /// Every strike, in combat order.
    pub strikes: Vec<StrikeView>,
    /// What the target's column says when it can't strike back.
    pub no_counter: Option<String>,
    /// What the totals' row is called.
    pub total_name: String,
    /// The art's effects that aren't numbers (`pierces`, `pins: Mov −3`),
    /// one per line; `no counter` is left out (the target's column says
    /// it).
    pub notes: Vec<String>,
}

/// The chosen art's or combat active's line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillLineView {
    /// Its name.
    pub name: String,
    /// The weapon's durability before and after, if it costs any.
    pub durability: Option<(u32, u32)>,
}

/// One side of the forecast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForecastSideView {
    /// The unit's name.
    pub name: String,
    /// Its faction.
    pub faction: Faction,
    /// What it fights with (empty with nothing).
    pub weapon: String,
    /// What is left of it, if anything is known.
    pub left: Option<LeftView>,
    /// Its HP now, after the attack if every strike hit, and its max.
    pub hp: ForecastHp,
    /// Its hit chance; `None` if it can't strike.
    pub hit: Option<u8>,
    /// Its crit chance; `None` if it can't strike.
    pub crit: Option<u8>,
    /// The target's affinity to its attack (`(resist)`, `heals 9`).
    pub affinity: Option<String>,
    /// Its strikes' total; `None` if it can't strike back.
    pub total: Option<TotalView>,
}

/// What is left of what a side fights with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeftView {
    /// The weapon is broken (or breaks): what that is called.
    Broken(String),
    /// Durability or uses left, of how many.
    Count {
        /// Left.
        left: u32,
        /// Of how many.
        max: u32,
    },
}

/// A side's HP in the forecast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForecastHp {
    /// Its HP now.
    pub now: StatValue,
    /// Its HP if every strike at it hit.
    pub after: StatValue,
    /// Its max HP.
    pub max: StatValue,
}

/// One strike of the forecast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrikeView {
    /// Who strikes.
    pub by: Side,
    /// Its damage as listed: `8 dmg`, or `+9 hp` for an Absorb heal.
    pub text: String,
    /// Whether it is effective (or the target is weak to it).
    pub effective: bool,
    /// Whether it kills.
    pub kills: bool,
    /// Whether it comes after a kill (and so won't happen).
    pub after_a_fall: bool,
}

/// A side's strikes, added up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalView {
    /// Their damage.
    pub damage: StatValue,
    /// How many there are.
    pub strikes: usize,
}

#[cfg(test)]
impl ForecastView {
    /// Every line of text the forecast shows, for tests.
    pub fn texts(&self) -> Vec<String> {
        let mut texts = vec![self.title.clone(), self.total_name.clone()];
        texts.extend(self.skill.as_ref().map(|s| s.name.clone()));
        for side in &self.sides {
            texts.extend([side.name.clone(), side.weapon.clone()]);
            texts.extend(side.affinity.clone());
            if let Some(LeftView::Broken(b)) = &side.left {
                texts.push(b.clone());
            }
        }
        texts.extend(self.strikes.iter().map(|s| s.text.clone()));
        texts.extend(self.no_counter.clone());
        texts.extend(self.notes.iter().cloned());
        texts
    }
}
