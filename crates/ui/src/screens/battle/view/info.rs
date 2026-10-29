//! What the unit info screen shows (ticket 0405, ADR-0054): everything
//! about one unit, in the player's language. Built by
//! [`info::view`](super::super::info::view). The lists are whole: the skin
//! shows as many as it has room for.

use crate::prelude::*;
use trpg_core::{Faction, StatValue};

/// The unit info screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoView {
    /// What stands in the portrait's place (there is no portrait yet).
    pub portrait: String,
    /// The unit's name.
    pub name: String,
    /// Its faction.
    pub faction: Faction,
    /// Its class and level (`Lord  Lv 1`).
    pub class_level: String,
    /// What EXP is called.
    pub exp_name: String,
    /// Its EXP towards its next level; `None` at the level cap.
    pub exp: Option<u32>,
    /// The name of the HP stat.
    pub hp_name: String,
    /// Its HP now.
    pub hp: StatValue,
    /// Its max HP.
    pub max: StatValue,
    /// The heading over the stats.
    pub stats_title: String,
    /// Its stats but HP (shown on the HP line), as its timed effects make
    /// them.
    pub stats: Vec<InfoStatView>,
    /// The heading over the timed effects.
    pub effects_title: String,
    /// Its timed effects.
    pub effects: Vec<InfoEffectView>,
    /// The heading over the skills.
    pub skills_title: String,
    /// The skills it can use.
    pub skills: Vec<InfoSkillView>,
    /// What Mov is called.
    pub mov_name: String,
    /// Its movement points.
    pub mov: StatValue,
    /// Its class's movement type, if known.
    pub movement: Option<String>,
    /// Its class's tags that are on, by name.
    pub tags: Vec<String>,
    /// The heading over the weapon ranks.
    pub ranks_title: String,
    /// Each weapon kind it can wield and its rank.
    pub ranks: Vec<(String, String)>,
    /// The heading over the affinities.
    pub affinities_title: String,
    /// What its class makes of each element (`Fire`, `Weak`); empty if
    /// nothing.
    pub affinities: Vec<(String, String)>,
    /// The heading over the supports.
    pub supports_title: String,
    /// Each partner it has a support rank with, and the rank (`Mira`,
    /// `B`), in the support table's order; empty if none (1003).
    pub supports: Vec<(String, String)>,
    /// The heading over the weapons.
    pub weapons_title: String,
    /// The mark on what is equipped (`E`).
    pub equipped_mark: String,
    /// Its weapon slots, an empty one `None`.
    pub weapons: Vec<Option<InfoWeaponView>>,
    /// Its armour.
    pub armour: GearView,
    /// Its accessory.
    pub accessory: GearView,
    /// The heading over the spells.
    pub spells_title: String,
    /// The spells it has learned.
    pub spells: Vec<InfoSpellView>,
    /// The page shown, on a skin that shows the screen a page at a time
    /// (the Playdate's, ticket 1039; [`info::turn_page`]). Always 0 on
    /// the PC.
    ///
    /// [`info::turn_page`]: super::super::info::turn_page
    pub page: u8,
}

/// A stat on the info screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoStatView {
    /// Its short name (`Str`).
    pub name: String,
    /// Its value, with the timed effects.
    pub value: StatValue,
    /// Whether a timed effect changes it.
    pub changed: bool,
}

/// A timed effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoEffectView {
    /// What gave it.
    pub name: String,
    /// What it changes.
    pub mods: String,
    /// Until when it lasts.
    pub until: String,
}

/// A skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoSkillView {
    /// Its mark: active (`A`) or passive (`P`).
    pub mark: String,
    /// Its name.
    pub name: String,
    /// An active's cost; `None` for a passive.
    pub cost: Option<String>,
    /// What it does.
    pub effect: String,
}

/// A weapon in a slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoWeaponView {
    /// Its name.
    pub name: String,
    /// Whether it is equipped.
    pub equipped: bool,
    /// What a broken one is called, if it is broken.
    pub broken: Option<String>,
    /// Its durability left.
    pub left: u32,
    /// Its durability when new.
    pub max: u32,
    /// Its might, hit, crit, range and weight.
    pub numbers: String,
}

/// A piece of gear and its heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GearView {
    /// The heading.
    pub title: String,
    /// Its name and bonus; `None` if there is none.
    pub item: Option<(String, String)>,
}

/// A learned spell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoSpellView {
    /// Its name.
    pub name: String,
    /// Whether it is equipped.
    pub equipped: bool,
    /// Its uses left this battle.
    pub left: u8,
    /// Its uses per battle.
    pub uses: u8,
}

#[cfg(test)]
impl InfoView {
    /// Every line of text the info screen shows, for tests.
    pub fn texts(&self) -> Vec<String> {
        let mut texts = vec![
            self.portrait.clone(),
            self.name.clone(),
            self.class_level.clone(),
            format!("{} {}/{}", self.hp_name, self.hp, self.max),
            self.stats_title.clone(),
            self.skills_title.clone(),
            self.ranks_title.clone(),
            self.weapons_title.clone(),
            self.armour.title.clone(),
            self.accessory.title.clone(),
            self.spells_title.clone(),
        ];
        texts.push(format!("{} {}", self.mov_name, self.mov));
        texts.extend(self.movement.clone());
        texts.extend(self.tags.iter().cloned());
        let stats = self.stats.iter().map(|s| format!("{} {}", s.name, s.value));
        texts.extend(stats);
        for e in &self.effects {
            texts.extend([e.name.clone(), e.mods.clone(), e.until.clone()]);
        }
        if !self.effects.is_empty() {
            texts.push(self.effects_title.clone());
        }
        for s in &self.skills {
            texts.extend([s.name.clone(), s.effect.clone()]);
            texts.extend(s.cost.clone());
        }
        for (kind, rank) in self.ranks.iter().chain(&self.affinities) {
            texts.extend([kind.clone(), rank.clone()]);
        }
        if !self.affinities.is_empty() {
            texts.push(self.affinities_title.clone());
        }
        for w in self.weapons.iter().flatten() {
            texts.extend([w.name.clone(), w.numbers.clone()]);
            texts.extend(w.broken.clone());
        }
        for (name, bonus) in [&self.armour, &self.accessory]
            .into_iter()
            .filter_map(|g| g.item.as_ref())
        {
            texts.extend([name.clone(), bonus.clone()]);
        }
        let spells = self.spells.iter();
        texts.extend(spells.map(|s| format!("{} {}/{}", s.name, s.left, s.uses)));
        texts
    }
}
