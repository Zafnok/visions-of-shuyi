//! The battle's village rule (ticket 0313, `turn-structure.md`, *Save the
//! village*): houses to save, `Evacuate`, raiders who burn them, the loss
//! and the rewards. The rules are in the parent module's docs.
//!
//! A burned house is marked lost here ([`HouseState::Burned`]) and its
//! tile becomes the goal's [`burnt`](VillageGoal::burnt) terrain (Nick:
//! a burned house gives no bonus to a unit standing on it), with an
//! [`Event::TerrainChanged`] as any terrain change has. Its state stays
//! here whatever the tile becomes later.

use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{BattleState, CommandError, Event, Step};
use crate::geom::Pos;
use crate::shop::Loot;
use crate::terrain::TerrainId;
use crate::unit::{Faction, Unit, UnitId};

/// A battle's village goal, from its battle file: which houses there are,
/// how many must be saved, and what each one past that gives on a win.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VillageGoal {
    /// Every house's tile (each `house` terrain tile of the map, found by
    /// `content`), in `(y, x)` order.
    pub houses: Vec<Pos>,
    /// Houses to save at least (1 to the number of houses).
    pub minimum: u32,
    /// The reward of each house standing past the minimum on a win: the
    /// first extra house gives the first, and so on. One per house past
    /// the minimum (`content` checks it).
    pub rewards: Vec<Loot>,
    /// The terrain a burned house's tile becomes (`burnt`, found by
    /// `content`).
    pub burnt: TerrainId,
}

/// Where a house stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HouseState {
    /// Not saved yet, not burned.
    Standing,
    /// Evacuated, or still standing when the battle was won: safe for good.
    Saved,
    /// Burned by a raider: lost for good.
    Burned,
}

/// The village of a running battle: its goal and each house's state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Village {
    minimum: u32,
    burnt: TerrainId,
    rewards: Vec<Loot>,
    houses: BTreeMap<Pos, HouseState>,
}

impl Village {
    /// The village of `goal`, every house standing.
    pub(super) fn new(goal: VillageGoal) -> Self {
        Village {
            minimum: goal.minimum,
            burnt: goal.burnt,
            rewards: goal.rewards,
            houses: goal
                .houses
                .into_iter()
                .map(|p| (p, HouseState::Standing))
                .collect(),
        }
    }

    /// Houses to save at least.
    pub fn minimum(&self) -> u32 {
        self.minimum
    }

    /// Every house and its state, by tile.
    pub fn houses(&self) -> &BTreeMap<Pos, HouseState> {
        &self.houses
    }

    /// The state of the house at `pos`, if there is one.
    pub fn house(&self, pos: Pos) -> Option<HouseState> {
        self.houses.get(&pos).copied()
    }

    /// How many houses there are.
    pub fn total(&self) -> u32 {
        count(self.houses.len())
    }

    /// How many houses are saved for good.
    pub fn saved(&self) -> u32 {
        self.count(HouseState::Saved)
    }

    /// How many houses burned.
    pub fn lost(&self) -> u32 {
        self.count(HouseState::Burned)
    }

    /// Whether so many houses burned that the minimum can't be met.
    pub fn is_failed(&self) -> bool {
        self.total() - self.lost() < self.minimum
    }

    /// The tiles of the houses still standing (not saved, not burned).
    pub fn standing(&self) -> impl Iterator<Item = Pos> + '_ {
        self.houses
            .iter()
            .filter(|&(_, &s)| s == HouseState::Standing)
            .map(|(&p, _)| p)
    }

    fn count(&self, state: HouseState) -> u32 {
        count(self.houses.values().filter(|&&s| s == state).count())
    }
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Whether a raider standing at `at` burns the house at `house`: on its
/// door or one of the four tiles beside it.
pub(crate) fn burns(at: Pos, house: Pos) -> bool {
    Pos::manhattan(at, house) <= 1
}

impl BattleState {
    /// The battle's village, if it has a village goal.
    pub fn village(&self) -> Option<&Village> {
        self.village.as_ref()
    }

    /// Whether unit `id` may choose [`UnitAction::Evacuate`](super::UnitAction::Evacuate)
    /// after moving to `dest` (the action menu offers it only then).
    /// Doesn't check that it can reach `dest` or still act.
    pub fn can_evacuate(&self, id: UnitId, dest: Pos) -> bool {
        self.unit(id)
            .is_some_and(|u| self.plan_evacuate(u, dest).is_ok())
    }

    /// Validates `unit` evacuating the house at `dest`.
    pub(super) fn plan_evacuate(&self, unit: &Unit, dest: Pos) -> Result<Step, CommandError> {
        if unit.faction != Faction::Player {
            return Err(CommandError::PlayerOnly(unit.id));
        }
        let house = self.village.as_ref().and_then(|v| v.house(dest));
        match house {
            None => Err(CommandError::NoHouse(dest)),
            Some(HouseState::Saved) => Err(CommandError::HouseAlreadySaved(dest)),
            Some(HouseState::Burned) => Err(CommandError::HouseLost(dest)),
            Some(HouseState::Standing) => Ok(Step::Evacuate(dest)),
        }
    }

    /// Carries out unit `id`'s validated evacuation of the house at `pos`.
    pub(super) fn evacuate(&mut self, id: UnitId, pos: Pos, events: &mut Vec<Event>) {
        if let Some(v) = self.village.as_mut() {
            v.houses.insert(pos, HouseState::Saved);
            events.push(Event::HouseSaved {
                pos,
                unit: Some(id),
            });
        }
    }

    /// At the end of unit `id`'s action: if it is a raider still on the
    /// map, every standing house on or beside its tile burns, in `(y, x)`
    /// order, and becomes burnt terrain.
    pub(super) fn raid(&mut self, id: UnitId, events: &mut Vec<Event>) {
        let Some(at) = self
            .unit(id)
            .filter(|u| u.raider && u.faction == Faction::Enemy)
            .map(|u| u.pos)
        else {
            return;
        };
        let Some(v) = self.village.as_mut() else {
            return;
        };
        let burned: Vec<Pos> = v.standing().filter(|&p| burns(at, p)).collect();
        let to = v.burnt;
        for pos in burned {
            v.houses.insert(pos, HouseState::Burned);
            events.push(Event::HouseBurned { pos, unit: id });
            if let Some(tile) = self.map.tiles.get_mut(pos)
                && *tile != to
            {
                let from = core::mem::replace(tile, to);
                events.push(Event::TerrainChanged { pos, from, to });
            }
        }
    }

    /// Whether the village goal is lost: too many houses burned.
    pub(super) fn village_failed(&self) -> bool {
        self.village.as_ref().is_some_and(Village::is_failed)
    }

    /// On a win: the houses still standing count as saved
    /// ([`Event::HouseSaved`] with no unit, in `(y, x)` order), then each
    /// house past the minimum gives its reward ([`Event::HouseReward`]),
    /// taken as a chest's loot is.
    pub(super) fn reward_village(&mut self, events: &mut Vec<Event>) {
        let Some(v) = self.village.as_mut() else {
            return;
        };
        let standing: Vec<Pos> = v.standing().collect();
        for pos in standing {
            v.houses.insert(pos, HouseState::Saved);
            events.push(Event::HouseSaved { pos, unit: None });
        }
        let extra = usize::try_from(v.saved().saturating_sub(v.minimum)).unwrap_or(usize::MAX);
        let rewards: Vec<Loot> = v.rewards.iter().take(extra).cloned().collect();
        for loot in rewards {
            events.push(Event::HouseReward { loot: loot.clone() });
            self.gain_loot(loot, events);
        }
    }
}
