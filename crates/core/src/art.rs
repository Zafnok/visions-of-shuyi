//! Combat Arts: weapon techniques learned by weapon rank, paid with the
//! attacking weapon's durability (`docs/design/combat-arts.md`).
//!
//! The arts live in `assets/data/arts.ron`, loaded by `trpg-content` into an
//! [`ArtTable`]. How a battle uses them is in [`crate::battle`].
//!
//! # Rules
//!
//! - **Rank arts** have a kind and a rank ([`ArtDef::rank`]). A unit
//!   [knows](Unit::known_arts) one once its recorded rank in that kind is at
//!   least the art's rank. Ranks never drop, so a known art stays known,
//!   even after a reclass to a class without the kind.
//! - **Weapon arts** (`rank: None`) are only reached through a weapon's own
//!   list ([`WeaponDef::arts`](crate::item::WeaponDef::arts)): anyone who
//!   can wield the weapon can use them, whatever its rank (*Claude's
//!   starting rule*).
//! - **Arts for a weapon** ([`Unit::arts_for`]): attacking with the weapon in
//!   a loadout slot, which the unit must be able to wield (so its **current
//!   class** has the kind): the rank arts of its kind up to the unit's rank
//!   in it, then the weapon's own arts. [`Unit::usable_arts`] keeps those it
//!   can pay for: the weapon isn't broken, even with less left than the
//!   cost (it then breaks after the combat; [`check_cost`]).
//! - **Effects** ([`ArtEffect`]) are data: bonuses and changes to the
//!   attacker's numbers for the whole combat ([`ArtEffect::apply`], so the
//!   forecast shows them), debuffs on the target's first hit, a stance on
//!   the user (Hold the Ford's stops enemies next to it), Line Pierce's
//!   extra strike, Pack Hunt's might per ally next to the target, and
//!   Moonwave's power and mitigation stats. The battle applies them.

use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::class::{ClassTable, UnitTag};
use crate::combat::{CombatMods, MitigationStat, PowerStat, WeaponStats, WeaponTrait};
use crate::item::ItemTable;
use crate::skill::{CostSource, SkillCost, TimedMods, check_cost};
use crate::stats::{StatKind, StatValue};
use crate::unit::{Faction, Role, Unit};
use crate::weapon::{WeaponKind, WeaponRank};

/// String id of a Combat Art, e.g. `"guard_break"`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArtId(pub String);

impl ArtId {
    /// An id from a string.
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }
}

/// A stat lowered on a unit until the end of its next phase (Pinning Shot,
/// Pressure Point). It never takes the stat below 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Debuff {
    /// The stat.
    pub stat: StatKind,
    /// How much it is lowered (at least 1).
    pub amount: StatValue,
}

impl Debuff {
    /// The debuff as timed-effect bonuses (a negative stat bonus).
    pub fn mods(self) -> TimedMods {
        TimedMods {
            stats: vec![(self.stat, -self.amount)],
            combat: CombatMods::default(),
            stops_enemies: false,
        }
    }
}

/// Reads a list of debuffs, or the single `Some((stat: .., amount: ..))` /
/// `None` older `arts.ron` entries wrote.
fn debuffs<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Debuff>, D::Error> {
    use serde::de::{self, MapAccess, SeqAccess, Visitor};
    struct Debuffs;
    impl<'de> Visitor<'de> for Debuffs {
        type Value = Vec<Debuff>;
        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            f.write_str("a debuff or a list of debuffs")
        }
        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            d.deserialize_any(self)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(debuff) = seq.next_element()? {
                out.push(debuff);
            }
            Ok(out)
        }
        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            Debuff::deserialize(de::value::MapAccessDeserializer::new(map)).map(|d| vec![d])
        }
    }
    d.deserialize_any(Debuffs)
}

/// What an art does. Every field is optional in data; the default does
/// nothing. Written in `arts.ron` as e.g. `(hit: 10, no_counter: true)`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ArtEffect {
    /// Added to the attacker's hit.
    pub hit: StatValue,
    /// Added to the attacker's crit (Skull Splitter).
    pub crit: StatValue,
    /// Replaces the sword follow-up ratio `(numerator, denominator)`
    /// (Flowing Cut: `(3, 2)`). Sword arts only.
    pub sword_followup: Option<(StatValue, StatValue)>,
    /// The defender can't counter.
    pub no_counter: bool,
    /// `(tag, multiplier)` pairs added to the weapon's effectiveness; the
    /// largest matching multiplier still wins (Unhorse, Armor Cleave).
    pub effective: Vec<(UnitTag, u8)>,
    /// Replaces the axe minimum damage. Axe arts only.
    pub axe_min_damage: Option<StatValue>,
    /// The axe minimum damage is at least a this-th of the target's max HP
    /// (Felling Blow: 3). Axe arts only.
    pub axe_min_damage_max_hp_div: Option<StatValue>,
    /// Replaces the weapon's minimum range (Close Shot: 1).
    pub min_range: Option<u32>,
    /// Replaces the weapon's maximum range (Long Thrust, Hurl: 2).
    pub max_range: Option<u32>,
    /// Each hit deals this much more than the one before it (Cascade).
    /// Sword arts only.
    pub cascade: StatValue,
    /// Debuffs put on the target by the attacker's first strike that hits.
    /// One `Some((stat: .., amount: ..))` loads as a list of one.
    #[serde(deserialize_with = "debuffs")]
    pub on_first_hit: Vec<Debuff>,
    /// The first hit also lowers the target's avoid by this much (Off
    /// Balance); avoid isn't a stat, so it isn't a [`Debuff`].
    pub on_first_hit_avoid: StatValue,
    /// A stance on the user, from this combat (it counts in it) until the
    /// start of the user's next phase (Sidestep).
    pub stance: Option<TimedMods>,
    /// After the combat, one strike at the unit directly behind the target
    /// (Line Pierce; see [`crate::battle`]).
    pub line_pierce: bool,
    /// `(per, max)`: might `+per` for each other unit of the user's side
    /// next to the target, up to `+max` (Pack Hunt: `(2, 6)`). The battle
    /// counts them ([`crate::battle`]).
    pub pack_might: Option<(StatValue, StatValue)>,
    /// Replaces the power stat (Moonwave: the higher of Str and Mag).
    pub power: Option<PowerStat>,
    /// Replaces the stat that mitigates the user's strikes (Moonwave: Res).
    pub mitigation: Option<MitigationStat>,
}

/// An effect of an art that isn't a number of the forecast, for the UI to
/// spell out (`combat-arts.md`, *The arts menu and the forecast*).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtNote {
    /// The defender can't counter.
    NoCounter,
    /// A strike at the unit behind the target follows.
    Pierces,
    /// The target's first hit puts this debuff on it (`pins: Mov −3`,
    /// `slows: Spd −3`).
    Debuff(Debuff),
    /// The target's first hit lowers its avoid by this much (`off balance:
    /// avoid −20`).
    AvoidDebuff(StatValue),
    /// The user takes this stance.
    Stance(TimedMods),
}

impl ArtEffect {
    /// Changes an attacker's `weapon` and combat `mods` for a combat with
    /// this art. The formulas stay in [`crate::combat`]; only their inputs
    /// change. [`no_counter`](Self::no_counter) isn't an input: the battle
    /// drops the defender's side of the forecast.
    pub fn apply(&self, weapon: &mut WeaponStats, mods: &mut CombatMods, target_max_hp: StatValue) {
        mods.hit += self.hit;
        mods.crit += self.crit;
        mods.cascade += self.cascade;
        if self.sword_followup.is_some() {
            mods.sword_followup = self.sword_followup;
        }
        weapon.effective.extend(self.effective.iter().copied());
        if let Some(min) = self.axe_min_damage {
            weapon.trait_ = WeaponTrait::AxeMinDamage(min);
        }
        if let Some(div) = self.axe_min_damage_max_hp_div
            && let WeaponTrait::AxeMinDamage(min) = weapon.trait_
        {
            weapon.trait_ = WeaponTrait::AxeMinDamage(min.max(target_max_hp / div.max(1)));
        }
        if let Some(min) = self.min_range {
            weapon.min_range = min;
        }
        if let Some(max) = self.max_range {
            weapon.max_range = max;
        }
        mods.power = self.power.or(mods.power);
        mods.mitigation = self.mitigation.or(mods.mitigation);
    }

    /// Pack Hunt's might for `allies` other units of the user's side next
    /// to the target; 0 without it.
    pub fn pack_might(&self, allies: usize) -> StatValue {
        self.pack_might.map_or(0, |(per, max)| {
            let allies = StatValue::try_from(allies).unwrap_or(StatValue::MAX);
            per.saturating_mul(allies).min(max)
        })
    }

    /// What the first hit puts on the target, to last until the end of its
    /// next phase; `None` if the art has no debuff.
    pub fn first_hit_mods(&self) -> Option<TimedMods> {
        if self.on_first_hit.is_empty() && self.on_first_hit_avoid == 0 {
            return None;
        }
        Some(TimedMods {
            stats: self
                .on_first_hit
                .iter()
                .map(|d| (d.stat, -d.amount))
                .collect(),
            combat: CombatMods {
                avoid_debuff: self.on_first_hit_avoid,
                ..CombatMods::default()
            },
            stops_enemies: false,
        })
    }

    /// The effects the forecast's numbers don't show, in a fixed order.
    pub fn notes(&self) -> Vec<ArtNote> {
        let mut out = Vec::new();
        if self.no_counter {
            out.push(ArtNote::NoCounter);
        }
        if self.line_pierce {
            out.push(ArtNote::Pierces);
        }
        out.extend(self.on_first_hit.iter().map(|&d| ArtNote::Debuff(d)));
        if self.on_first_hit_avoid > 0 {
            out.push(ArtNote::AvoidDebuff(self.on_first_hit_avoid));
        }
        if let Some(stance) = &self.stance {
            out.push(ArtNote::Stance(stance.clone()));
        }
        out
    }
}

/// One Combat Art.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtDef {
    /// String id.
    pub id: ArtId,
    /// Display name.
    pub name: String,
    /// The weapon kind it is used with.
    pub kind: WeaponKind,
    /// The rank that teaches it; `None` for a weapon art (only through a
    /// weapon's `arts` list).
    pub rank: Option<WeaponRank>,
    /// Durability it costs, paid once per combat (at least 1).
    pub cost: u32,
    /// What it does.
    pub effect: ArtEffect,
}

impl ArtDef {
    /// Its cost as a skill cost, for [`check_cost`] and
    /// [`pay_cost`](crate::skill::pay_cost).
    pub fn skill_cost(&self) -> SkillCost {
        SkillCost::Durability(self.cost)
    }
}

/// Every Combat Art.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtTable {
    /// Arts by id.
    pub arts: BTreeMap<ArtId, ArtDef>,
}

impl ArtTable {
    /// The art `id`.
    pub fn get(&self, id: &ArtId) -> Option<&ArtDef> {
        self.arts.get(id)
    }

    /// The rank arts of `kind` up to `rank`, lowest rank first (id order
    /// within a rank).
    fn rank_arts(&self, kind: WeaponKind, rank: WeaponRank) -> Vec<&ArtDef> {
        let mut out: Vec<&ArtDef> = self
            .arts
            .values()
            .filter(|a| a.kind == kind && a.rank.is_some_and(|r| r <= rank))
            .collect();
        out.sort_by_key(|a| a.rank);
        out
    }
}

/// A unit's Combat Art rules (see the module docs).
impl Unit {
    /// Whether the unit may use arts and active skills (`combat-arts.md`,
    /// *Enemies: bosses only; combat green units too*): every player unit,
    /// bosses among enemies, and green units that aren't non-combat ones.
    pub fn may_use_arts(&self) -> bool {
        match self.faction {
            Faction::Player => true,
            Faction::Enemy => self.role == Role::Boss,
            Faction::Ally | Faction::Neutral => self.role != Role::Noncombatant,
        }
    }

    /// The rank arts the unit knows, by kind, then lowest rank first. Only
    /// ranks it has recorded count (a unit that never used bows knows no
    /// bow arts), whatever its current class.
    pub fn known_arts<'a>(&self, arts: &'a ArtTable) -> Vec<&'a ArtDef> {
        self.weapon_ranks
            .iter()
            .flat_map(|(&kind, &rank)| arts.rank_arts(kind, rank))
            .collect()
    }

    /// The arts the unit could use attacking with the weapon in `slot`,
    /// affordable or not: none if it can't wield it (empty slot, unknown
    /// class or item, a kind its class lacks, too low a rank).
    pub fn arts_for<'a>(
        &self,
        slot: usize,
        classes: &ClassTable,
        items: &ItemTable,
        arts: &'a ArtTable,
    ) -> Vec<&'a ArtDef> {
        let Some(class) = classes.get(&self.class) else {
            return Vec::new();
        };
        let Some((_, weapon)) = self.usable_weapon(slot, class, items) else {
            return Vec::new();
        };
        let mut out = arts.rank_arts(weapon.kind, self.rank(weapon.kind));
        out.extend(weapon.arts.iter().filter_map(|id| arts.get(id)));
        out
    }

    /// [`Unit::arts_for`] the weapon in `slot`, keeping those it can pay for
    /// now (the weapon isn't broken; less left than the cost is fine).
    pub fn usable_arts<'a>(
        &self,
        slot: usize,
        classes: &ClassTable,
        items: &ItemTable,
        arts: &'a ArtTable,
    ) -> Vec<&'a ArtDef> {
        self.arts_for(slot, classes, items, arts)
            .into_iter()
            .filter(|a| check_cost(self, a.skill_cost(), &CostSource::Weapon(slot)).is_ok())
            .collect()
    }
}

#[cfg(test)]
mod tests;
