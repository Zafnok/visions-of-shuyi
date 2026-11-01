//! One random pool per world map, in `skirmishes/<world-map-id>.ron`.
use crate::prelude::*;
use crate::ron_loader::parse_ron;
use crate::worldmap::{WorldMapDef, WorldRefs};
use crate::{Content, ContentError, battle, bundle};
use alloc::collections::BTreeMap;
use trpg_core::{Loot, ShopKind, SkirmishRules};

pub(crate) fn load(
    maps: &mut BTreeMap<String, WorldMapDef>,
    refs: WorldRefs<'_>,
    errors: &mut Vec<ContentError>,
) {
    let loaded = bundle::load_each("skirmishes", "ron", |file| {
        let rules: SkirmishRules = parse_ron(file.path, file.text).map_err(|e| vec![e])?;
        let mut problems = Vec::new();
        check(&rules, refs, &mut |e| {
            problems.push(ContentError::new(file.path, e));
        });
        if problems.is_empty() {
            Ok(rules)
        } else {
            Err(problems)
        }
    });
    match loaded {
        Err(e) => errors.extend(e),
        Ok(pools) => {
            for (id, rules) in pools {
                match maps.get_mut(&id) {
                    Some(def) => def.map.skirmishes = Some(rules),
                    None => errors.push(ContentError::new(
                        format!("skirmishes/{id}.ron"),
                        "pool names no world map",
                    )),
                }
            }
        }
    }
}

fn check(rules: &SkirmishRules, refs: WorldRefs<'_>, err: &mut dyn FnMut(String)) {
    if rules.spawn.0 > rules.spawn.1
        || rules.spawn.1 > rules.maximum
        || rules.size.0 == 0
        || rules.size.0 > rules.size.1
    {
        err("invalid spawn, capacity or enemy-count range".into());
    }
    if rules.pool.is_empty() || rules.rewards.is_empty() {
        err("pool and rewards must not be empty".into());
    }
    for id in &rules.pool {
        if !refs
            .battles
            .get(id)
            .is_some_and(|b| battle::random_pool(b, rules.size.1))
        {
            err(format!(
                "{id}: random pool needs an Easy rout battle, generic templates with default loadouts, enough enemies, Pool(\"skirmish\"), and no authored loot or scripts"
            ));
        }
    }
    for reward in &rules.rewards {
        if let Some(Loot::Item(item)) = reward
            && !refs
                .items
                .get(item)
                .is_some_and(|i| ShopKind::Basic.stocks(i))
        {
            err(format!(
                "{}: skirmish reward must be an ordinary basic-shop item, never a special weapon or seal",
                item.0
            ));
        }
    }
}

/// Resolve a battle with a saved group's count, levels, seed and reward.
/// The ordinary battle remains unchanged in the shared content tables.
pub fn battle(
    content: &Content,
    campaign: &trpg_core::Campaign,
    id: &str,
) -> Option<trpg_core::BattleDef> {
    let group = campaign.world.as_ref().and_then(|w| {
        content
            .worldmaps
            .get(&w.map)?
            .map
            .random_at(&w.at, campaign)
    });
    match group {
        Some(group) if group.battle == id => {
            battle::random_skirmish(group, &content.battle_refs()).ok()
        }
        _ => content.battles.get(id).cloned(),
    }
}

#[cfg(test)]
mod tests;
