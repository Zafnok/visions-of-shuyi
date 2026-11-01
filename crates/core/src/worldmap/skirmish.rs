//! Saved random groups and the authored rules that create them (1008).
use super::{NodeKind, WorldEvent, WorldMap};
use crate::prelude::*;
use crate::{Campaign, Loot, SimRng};
use serde::{Deserialize, Serialize};

/// Tunable rules of one act's random skirmishes, loaded from data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkirmishRules {
    /// Inclusive spawn-count range after a story victory.
    pub spawn: (u32, u32),
    /// Maximum groups present at once.
    pub maximum: u32,
    /// Levels below the latest story victory's enemy level.
    pub level_offset: u32,
    /// Inclusive enemy-count range.
    pub size: (u32, u32),
    /// Ordinary battle files, each with at least `size.1` generic enemies.
    pub pool: Vec<String>,
    /// Equally likely rewards; `None` means combat EXP only.
    pub rewards: Vec<Option<Loot>>,
}

/// A group on a cleared place. Saved rolls are reused on retry and load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomSkirmish {
    /// Authored battle from the act's pool.
    pub battle: String,
    /// Enemy level, never above the cap at spawning.
    pub level: u32,
    /// How many of the file's enemy entries to use, in order.
    pub size: u32,
    /// The battle's deterministic seed.
    pub seed: u64,
    /// Clear reward, in addition to ordinary battle EXP and rewind bonus.
    pub reward: Option<Loot>,
}

fn roll(rng: &mut SimRng, range: (u32, u32)) -> u32 {
    range
        .0
        .saturating_add(rng.roll_below(range.1.saturating_sub(range.0).saturating_add(1)))
}

impl Campaign {
    /// Spawn only after a newly won story battle, before leaving its act.
    pub(super) fn spawn_skirmishes(&mut self, map: &WorldMap, level: u32) -> Vec<WorldEvent> {
        let Some(rules) = &map.skirmishes else {
            return Vec::new();
        };
        let cap = level.saturating_sub(rules.level_offset).min(map.levels.1);
        let mut events = Vec::new();
        let Some(world) = self.world.as_mut() else {
            return events;
        };
        let valid = cap >= map.levels.0.max(1);
        if world.skirmishes.values().any(|g| !valid || g.level > cap) {
            world.skirmishes.retain(|_, g| {
                g.level = g.level.min(cap);
                valid
            });
            events.push(WorldEvent::SkirmishesCapped(cap));
        }
        if !valid || rules.pool.is_empty() {
            return events;
        }
        let mut places: Vec<String> = map
            .nodes
            .iter()
            .filter(|n| {
                world.cleared.contains(&n.id)
                    && n.id != world.at
                    && !world.skirmishes.contains_key(&n.id)
                    && matches!(
                        n.kind,
                        NodeKind::StoryBattle(_)
                            | NodeKind::FixedSkirmish { .. }
                            | NodeKind::SideQuest(_)
                    )
            })
            .map(|n| n.id.clone())
            .collect();
        let room = rules
            .maximum
            .saturating_sub(u32::try_from(world.skirmishes.len()).unwrap_or(u32::MAX));
        let rng = self.rng.get_or_insert_with(|| SimRng::new(0));
        let count = roll(rng, rules.spawn)
            .min(room)
            .min(u32::try_from(places.len()).unwrap_or(u32::MAX));
        for _ in 0..count {
            let index = rng.roll_below(u32::try_from(places.len()).unwrap_or(u32::MAX)) as usize;
            let node = places.remove(index);
            let battle = rules.pool
                [rng.roll_below(u32::try_from(rules.pool.len()).unwrap_or(u32::MAX)) as usize]
                .clone();
            let reward = rules
                .rewards
                .get(
                    rng.roll_below(u32::try_from(rules.rewards.len()).unwrap_or(u32::MAX)) as usize,
                )
                .cloned()
                .flatten();
            let group = RandomSkirmish {
                battle,
                level: cap,
                size: roll(rng, rules.size),
                seed: u64::from(rng.next_u32()),
                reward,
            };
            events.push(WorldEvent::SkirmishSpawned { node, group });
        }
        if let Some(world) = &mut self.world {
            for event in &events {
                if let WorldEvent::SkirmishSpawned { node, group } = event {
                    world.skirmishes.insert(node.clone(), group.clone());
                }
            }
        }
        events
    }
}
