//! The army a player brings to a story battle (tickets 0520, 0531): the
//! New Game campaign, each earlier chapter's battle won by a recorded
//! playthrough and its result applied as the game applies it, and, in a
//! battle with Preparations, the pack a player fills (`docs/playtesting.md`,
//! *The army*). On a world map, the fixed skirmishes that guard the way to a
//! chapter's story battle are won first, as every player wins them (ticket
//! 1071). One army per line of the bots' recorded wins ([`Wins`]), so no
//! single recording sets it. Deterministic: the same content and wins give
//! the same armies.

use trpg_content::chapter::{ChapterDef, roster_units};
use trpg_content::{Content, new_campaign};
use trpg_core::{
    BattleDef, BattleSetup, BattleState, Campaign, GameMode, ItemId, LeadGender, LeadProfile,
    NodeKind, Shop, WorldMap,
};

use crate::army::{roster_by, story_chapters};
use crate::record::play;
use crate::wins::Wins;

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
/// before it (`lead` the lead's profile), one army per line of the bots'
/// recorded wins ([`Wins::embedded`]), line 1's first: `None` when nothing
/// comes before it ([`chapters_before`]). An error when a battle before it
/// has no recorded wins, or fewer lines than the first.
pub fn arrival(
    content: &Content,
    def: &BattleDef,
    mode: GameMode,
    lead: &LeadProfile,
) -> Result<Option<Vec<BattleSetup>>, String> {
    arrival_with(content, def, mode, lead, Wins::embedded()?)
}

/// [`arrival`], with the lines of `wins`.
pub fn arrival_with(
    content: &Content,
    def: &BattleDef,
    mode: GameMode,
    lead: &LeadProfile,
    wins: &Wins,
) -> Result<Option<Vec<BattleSetup>>, String> {
    let Some(steps) = steps(content, &def.id) else {
        return Ok(None);
    };
    let Some((first, _)) = steps.won.first() else {
        return Ok(None);
    };
    let lines = wins.lines(first);
    if lines == 0 {
        wins.line(first, 0)?;
    }
    let armies = (0..lines).map(|line| {
        let (campaign, chapter) = campaign_before(content, &steps, mode, lead.clone(), wins, line)?;
        Ok(setup(content, &campaign, def, chapter))
    });
    armies.collect::<Result<Vec<_>, String>>().map(Some)
}

/// Battle `def` as line `line` (from 0) arrives at it in `mode`, the
/// battle its recorded win is played from ([`record`](crate::record)): the
/// campaign after that line's wins of the battles before it, prepared.
/// Chapter 1's is the New Game campaign. `None` for a battle no story
/// chapter fights.
pub fn start_of(
    content: &Content,
    def: &BattleDef,
    mode: GameMode,
    lead: LeadProfile,
    wins: &Wins,
    line: usize,
) -> Result<Option<BattleSetup>, String> {
    let Some(steps) = steps(content, &def.id) else {
        return Ok(None);
    };
    let (campaign, chapter) = campaign_before(content, &steps, mode, lead, wins, line)?;
    Ok(Some(setup(content, &campaign, def, chapter)))
}

/// The battles whose recorded wins a later battle's army plays, in story
/// order: each chapter's guards (1071), then its story battle; the last
/// chapter's battle has none after it. Their wins are recorded in this
/// order, each line from the army its wins before it leave.
pub fn won_on_the_way(content: &Content) -> Vec<String> {
    let chapters = story_chapters(content);
    let guards = guards_by_chapter(content, &chapters);
    let mut battles = Vec::new();
    for (chapter, guards) in chapters.iter().zip(&guards) {
        battles.extend(guards.iter().map(|(_, b)| b.clone()));
        battles.push(chapter.battle.clone());
    }
    battles.pop();
    battles
}

/// What comes before a battle on its way through the story.
struct Steps<'a> {
    /// The chapters up to the battle's own, its own last.
    chapters: Vec<&'a ChapterDef>,
    /// The battles won before it (story battles and guards, in order),
    /// each with the index in `chapters` of the chapter it is fought in.
    won: Vec<(String, usize)>,
}

/// The [`Steps`] before battle `battle`: `None` for a battle no story
/// chapter fights.
fn steps<'a>(content: &'a Content, battle: &str) -> Option<Steps<'a>> {
    let mut chapters = story_chapters(content);
    let (at, guard) = place_in_story(content, &chapters, battle)?;
    let guards = guards_by_chapter(content, &chapters);
    chapters.truncate(at + 1);
    let mut won = Vec::new();
    for (i, chapter) in chapters.iter().enumerate() {
        let before = if i == at { guard } else { None };
        let on_the_way = &guards[i][..before.unwrap_or(guards[i].len())];
        won.extend(on_the_way.iter().map(|(_, b)| (b.clone(), i)));
        if i < at {
            won.push((chapter.battle.clone(), i));
        }
    }
    Some(Steps { chapters, won })
}

/// The campaign as line `line` brings it to the last of `steps`' chapters:
/// New Game, each chapter's recruits joining before its battles, each
/// battle won by that line's recorded win and its result applied with every
/// rewind charge unused. With that chapter.
fn campaign_before<'a>(
    content: &Content,
    steps: &Steps<'a>,
    mode: GameMode,
    lead: LeadProfile,
    wins: &Wins,
    line: usize,
) -> Result<(Campaign, &'a ChapterDef), String> {
    let mut campaign = new_campaign(content, mode, lead);
    // The army's place on the map changes no battle: which guards it
    // fights is the plan's, so only the wins are played here.
    let mut won = steps.won.iter().peekable();
    for (i, chapter) in steps.chapters.iter().enumerate() {
        recruit(&mut campaign, content, &chapter.id);
        while let Some((battle, _)) = won.next_if(|(_, at)| *at == i) {
            let commands = wins.line(battle, line)?;
            win(&mut campaign, content, battle, chapter, commands)
                .map_err(|e| format!("line {} of {e}", line + 1))?;
        }
    }
    let Some(last) = steps.chapters.last() else {
        return Err("no chapter".to_owned());
    };
    Ok((campaign, last))
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
    // Each turn wins a place, so a way longer than the map has places
    // never ends: stop there.
    for _ in 0..map.nodes.len() {
        let at = campaign.world.as_ref().map_or_default(|w| w.at.clone());
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
    won
}

/// Battle `battle`, fought in `chapter`, won by `commands` (a recorded
/// win) and its result applied with every rewind charge unused.
fn win(
    campaign: &mut Campaign,
    content: &Content,
    battle: &str,
    chapter: &ChapterDef,
    commands: &[trpg_core::Command],
) -> Result<(), String> {
    let won = content.battles.get(battle);
    let won = won.ok_or_else(|| format!("no battle \"{battle}\""))?;
    let mut state = BattleState::new(setup(content, campaign, won, chapter)).0;
    play(&mut state, content, commands).map_err(|e| format!("{}'s recorded win: {e}", won.id))?;
    let unused = state.rewind_charges();
    campaign
        .apply_result(won, &state, unused)
        .map_err(|e| format!("{}'s recorded win: {e}", won.id))?;
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

#[cfg(test)]
mod tests;
