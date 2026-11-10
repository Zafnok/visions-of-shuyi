//! The army a player brings to a story battle (ticket 0520): the New Game
//! campaign, each earlier chapter's battle won by its recorded playthrough
//! and its result applied as the game applies it, and, in a battle with
//! Preparations, the pack a player fills (`docs/playtesting.md`, *The
//! army*). On a world map, the fixed skirmishes that guard the way to a
//! chapter's story battle are won first, as every player wins them (ticket
//! 1071). Deterministic: the same content gives the same army.

use trpg_content::chapter::{ChapterDef, roster_units};
use trpg_content::{Content, new_campaign};
use trpg_core::{
    BattleDef, BattleSetup, BattleState, Campaign, Command, GameMode, ItemId, LeadGender,
    LeadProfile, NodeKind, Phase, Shop, WorldMap, next_command,
};

use crate::army::{roster_by, story_chapters};

/// The story battles' recorded winning playthroughs, by battle id: the
/// player's commands, phase by phase (tickets 0803, 0848 to 0851), and
/// those of the fixed skirmishes that guard their way (1071).
/// `trpg-ui`'s `ch01_winnable` to `ch05_winnable` tests play them in the
/// real game and say how to record a new one.
const PLAYTHROUGHS: [(&str, &str); 7] = [
    (
        "ch01",
        include_str!("../../ui/tests/it/fixtures/ch01_winning.ron"),
    ),
    (
        "ch02",
        include_str!("../../ui/tests/it/fixtures/ch02_winning.ron"),
    ),
    (
        "ch03",
        include_str!("../../ui/tests/it/fixtures/ch03_winning.ron"),
    ),
    (
        "ch04",
        include_str!("../../ui/tests/it/fixtures/ch04_winning.ron"),
    ),
    (
        "ch05",
        include_str!("../../ui/tests/it/fixtures/ch05_winning.ron"),
    ),
    (
        "aldwater",
        include_str!("../../ui/tests/it/fixtures/aldwater_winning.ron"),
    ),
    (
        "salt_road",
        include_str!("../../ui/tests/it/fixtures/salt_road_winning.ron"),
    ),
];

/// Where battle `battle` falls in the story: the index of the chapter it
/// is fought in among `chapters`, and for a guard (1071), its index among
/// the guards that chapter's army wins on the way ([`guards_by_chapter`]).
/// `None` for a battle no story chapter fights.
fn place_in_story(
    content: &Content,
    chapters: &[&ChapterDef],
    battle: &str,
) -> Option<(usize, Option<usize>)> {
    if let Some(at) = chapters.iter().position(|c| c.battle == battle) {
        return Some((at, None));
    }
    let guards = guards_by_chapter(content, chapters);
    guards.iter().enumerate().find_map(|(at, guards)| {
        let i = guards.iter().position(|(_, b)| b == battle)?;
        Some((at, Some(i)))
    })
}

/// The story chapters played before the one battle `battle` is fought in,
/// in order: its own for a story battle, the one whose way it guards for a
/// guard (1071). Empty for the first chapter's battle and for a battle no
/// story chapter fights.
pub fn chapters_before<'a>(content: &'a Content, battle: &str) -> Vec<&'a ChapterDef> {
    let mut chapters = story_chapters(content);
    match place_in_story(content, &chapters, battle) {
        Some((at, _)) => {
            chapters.truncate(at);
            chapters
        }
        None => Vec::new(),
    }
}

/// Battle `def` in `mode` as a player arrives at it after the chapters
/// before it (`lead` the lead's profile): `None` when nothing comes
/// before it ([`chapters_before`]). The campaign starts as New Game; each
/// earlier chapter's battle is prepared ([`prepare`]), won by its recorded
/// playthrough with the enemy playing as in the game, and its result
/// applied with every rewind charge unused; whoever a scene recruits by
/// each chapter joins fresh from the character data before its battle; on
/// a world map, the guards on the way are won first, in the same way
/// (`guards_by_chapter`). Then `def` is prepared too: a chapter's story
/// battle, or a guard on its way, after the guards before it.
pub fn arrival(
    content: &Content,
    def: &BattleDef,
    mode: GameMode,
    lead: LeadProfile,
) -> Result<Option<BattleSetup>, String> {
    let chapters = story_chapters(content);
    let Some((at, guard)) = place_in_story(content, &chapters, &def.id) else {
        return Ok(None);
    };
    if at == 0 && guard.is_none() {
        return Ok(None);
    }
    let guards = guards_by_chapter(content, &chapters);
    let mut campaign = new_campaign(content, mode, lead);
    for (i, chapter) in chapters.iter().enumerate().take(at + 1) {
        recruit(&mut campaign, content, &chapter.id);
        let map = chapter
            .world_map
            .as_ref()
            .and_then(|m| content.worldmaps.get(m));
        if let Some(map) = map {
            chapter.id.clone_into(&mut campaign.chapter);
            campaign.enter_world(&map.map);
        }
        let before = if i == at { guard } else { None };
        let on_the_way = &guards[i][..before.unwrap_or(guards[i].len())];
        for (place, battle) in on_the_way {
            win(&mut campaign, content, battle, chapter)?;
            if let Some(map) = map {
                campaign.clear_node(&map.map, place, None, None);
            }
        }
        if i == at {
            break;
        }
        win(&mut campaign, content, &chapter.battle, chapter)?;
        let story = NodeKind::StoryBattle(chapter.id.clone());
        if let Some(map) = map
            && let Some(node) = map.map.nodes.iter().find(|n| n.kind == story)
        {
            campaign.clear_node(&map.map, &node.id, None, None);
        }
    }
    Ok(Some(setup(content, &campaign, def, chapters[at])))
}

/// For each of `chapters`, the fixed skirmishes that guard the way to its
/// story battle, as (place, battle), in the order its army must win them
/// (1071): while the army can't reach the story battle without passing an
/// unwon battle, the first open guard of the map's places. The way
/// depends only on the places won, so this wins nothing; [`arrival`]
/// plays it. Empty for a chapter off the map.
fn guards_by_chapter(content: &Content, chapters: &[&ChapterDef]) -> Vec<Vec<(String, String)>> {
    // Who leads changes nothing on the map.
    let lead = LeadProfile::new("Lead", LeadGender::Male);
    let mut campaign = new_campaign(content, GameMode::Classic, lead);
    chapters
        .iter()
        .map(|chapter| {
            let Some(map) = chapter
                .world_map
                .as_ref()
                .and_then(|m| content.worldmaps.get(m))
            else {
                return Vec::new();
            };
            guards_on_the_way(&mut campaign, &map.map, &chapter.id)
        })
        .collect()
}

/// The guards [`guards_by_chapter`] finds for chapter `chapter` on `map`,
/// with `campaign` where the chapters before left it; then its story
/// battle is won too.
fn guards_on_the_way(
    campaign: &mut Campaign,
    map: &WorldMap,
    chapter: &str,
) -> Vec<(String, String)> {
    chapter.clone_into(&mut campaign.chapter);
    campaign.enter_world(map);
    let story = NodeKind::StoryBattle(chapter.to_owned());
    let Some(node) = map.nodes.iter().find(|n| n.kind == story) else {
        return Vec::new();
    };
    let mut won = Vec::new();
    loop {
        let at = campaign
            .world
            .as_ref()
            .map(|w| w.at.clone())
            .unwrap_or_default();
        let route = map.route(&at, &node.id, campaign);
        let guard = map.nodes.iter().find_map(|n| match &n.kind {
            NodeKind::FixedSkirmish {
                battle,
                guards_path: true,
            } if map.is_open(&n.id, campaign) && map.blocks_path(&n.id, campaign) => {
                Some((n.id.clone(), battle.battle.clone()))
            }
            _ => None,
        });
        let reached = route.and_then(|r| r.last().cloned()).as_ref() == Some(&node.id);
        let next = if reached { None } else { guard };
        // The guard, else the story battle: won, and the army there.
        let place = next.as_ref().map_or(&node.id, |(p, _)| p).clone();
        campaign.clear_node(map, &place, None, None);
        if let Some(world) = &mut campaign.world {
            world.at = place;
        }
        match next {
            Some(guard) => won.push(guard),
            None => return won,
        }
    }
}

/// Battle `battle`, fought in `chapter`, won by its recorded playthrough
/// and its result applied with every rewind charge unused.
fn win(
    campaign: &mut Campaign,
    content: &Content,
    battle: &str,
    chapter: &ChapterDef,
) -> Result<(), String> {
    let won = content.battles.get(battle);
    let won = won.ok_or_else(|| format!("no battle \"{battle}\""))?;
    let mut state = BattleState::new(setup(content, campaign, won, chapter)).0;
    play(&mut state, content, &recorded(&won.id)?)
        .map_err(|e| format!("{}'s recorded playthrough: {e}", won.id))?;
    let unused = state.rewind_charges();
    campaign
        .apply_result(won, &state, unused)
        .map_err(|e| format!("{}'s recorded playthrough: {e}", won.id))?;
    Ok(())
}

/// `campaign`'s battle `def` of `chapter`, prepared.
fn setup(
    content: &Content,
    campaign: &Campaign,
    def: &BattleDef,
    chapter: &ChapterDef,
) -> BattleSetup {
    let mut setup = campaign.battle_setup(def, &content.tables());
    if def.preparations {
        let shop = chapter.number.map(|n| content.basic_shop.shop(n));
        prepare(&mut setup, shop.as_ref());
    }
    setup
}

/// Whoever the scenes recruit by `chapter`, not yet in `campaign`, joins it.
fn recruit(campaign: &mut Campaign, content: &Content, chapter: &str) {
    for who in roster_by(content, chapter) {
        if campaign.member(&who).is_none() {
            for unit in roster_units(content, &[who]) {
                campaign.joins_in_scene(unit);
            }
        }
    }
}

/// Fills the pack of `setup`, a battle with Preparations, as a player
/// would (*the agent's starting rule*, `docs/playtesting.md`): first the
/// stock's consumables, the dearest first, then, while the pack has room,
/// the dearest consumable `shop` sells that the gold still pays for.
/// Weapons, armour and loadouts stay as they are.
pub fn prepare(setup: &mut BattleSetup, shop: Option<&Shop>) {
    let items = std::sync::Arc::clone(&setup.items);
    let price = |id: &ItemId| items.consumable(id).map(|c| c.price);
    let dearest_first = |ids: &mut Vec<ItemId>| ids.sort_by_key(|id| std::cmp::Reverse(price(id)));
    let mut owned: Vec<ItemId> = setup
        .stock
        .items
        .keys()
        .filter(|id| price(id).is_some())
        .cloned()
        .collect();
    dearest_first(&mut owned);
    for id in &owned {
        while setup.pack_from_stock(id).is_ok() {}
    }
    let Some(shop) = shop else {
        return;
    };
    let mut sold: Vec<ItemId> = shop
        .stock
        .iter()
        .filter(|id| price(id).is_some())
        .cloned()
        .collect();
    dearest_first(&mut sold);
    while setup.pack.items.len() < setup.pack.cap {
        let affordable = sold
            .iter()
            .find(|id| price(id).is_some_and(|p| p <= setup.gold));
        let Some(id) = affordable else {
            break;
        };
        if setup.buy(shop, id).is_err() || setup.pack_from_stock(id).is_err() {
            break;
        }
    }
}

/// The recorded playthrough of `battle`.
fn recorded(battle: &str) -> Result<Vec<Command>, String> {
    let (_, text) = PLAYTHROUGHS
        .iter()
        .find(|(id, _)| *id == battle)
        .ok_or_else(|| {
            format!("no recorded playthrough of \"{battle}\" (trpg-ui's tests record one)")
        })?;
    ron::from_str(text).map_err(|e| format!("{battle}'s recorded playthrough: {e}"))
}

/// Plays `commands`, the player's, from `state` to the battle's end, the
/// other phases as the game plays them: the enemy AI's command, else the
/// phase's end.
fn play(state: &mut BattleState, content: &Content, commands: &[Command]) -> Result<(), String> {
    for (i, command) in commands.iter().enumerate() {
        others(state, content)?;
        if state.outcome().is_some() {
            return Err(format!("over before command {i}"));
        }
        state
            .apply(command)
            .map_err(|e| format!("command {i}, {command:?}, turn {}: {e}", state.turn()))?;
    }
    others(state, content)
}

/// Plays the phases that aren't the player's.
fn others(state: &mut BattleState, content: &Content) -> Result<(), String> {
    while state.outcome().is_none() && state.phase() != Phase::Player {
        let command = next_command(state, &content.ai).unwrap_or(Command::EndPhase);
        state
            .apply(&command)
            .map_err(|e| format!("the AI's {command:?} was refused: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
