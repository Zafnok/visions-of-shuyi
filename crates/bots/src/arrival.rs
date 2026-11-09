//! The army a player brings to a story battle (ticket 0520): the New Game
//! campaign, each earlier chapter's battle won by its recorded playthrough
//! and its result applied as the game applies it, and, in a battle with
//! Preparations, the pack a player fills (`docs/playtesting.md`, *The
//! army*). Deterministic: the same content gives the same army.

use trpg_content::chapter::{ChapterDef, roster_units};
use trpg_content::{Content, new_campaign};
use trpg_core::{
    BattleDef, BattleSetup, BattleState, Campaign, Command, GameMode, ItemId, LeadProfile, Phase,
    Shop, next_command,
};

use crate::army::{roster_by, story_chapters};

/// The story battles' recorded winning playthroughs, by battle id: the
/// player's commands, phase by phase (tickets 0803, 0848 to 0851).
/// `trpg-ui`'s `ch01_winnable` to `ch05_winnable` tests play them in the
/// real game and say how to record a new one.
const PLAYTHROUGHS: [(&str, &str); 5] = [
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
];

/// The story chapters played before the one whose battle is `battle`, in
/// order. Empty for the first chapter's battle and for a battle no story
/// chapter fights.
pub fn chapters_before<'a>(content: &'a Content, battle: &str) -> Vec<&'a ChapterDef> {
    let mut chapters = story_chapters(content);
    match chapters.iter().position(|c| c.battle == battle) {
        Some(at) => {
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
/// each chapter joins fresh from the character data before its battle.
/// Then `def` is prepared too.
pub fn arrival(
    content: &Content,
    def: &BattleDef,
    mode: GameMode,
    lead: LeadProfile,
) -> Result<Option<BattleSetup>, String> {
    let chapters = story_chapters(content);
    let at = chapters.iter().position(|c| c.battle == def.id);
    let Some(at) = at.filter(|&at| at > 0) else {
        return Ok(None);
    };
    let mut campaign = new_campaign(content, mode, lead);
    for chapter in &chapters[..at] {
        recruit(&mut campaign, content, &chapter.id);
        let won = content.battles.get(&chapter.battle);
        let won = won.ok_or_else(|| format!("no battle \"{}\"", chapter.battle))?;
        let mut state = BattleState::new(setup(content, &campaign, won, chapter)).0;
        play(&mut state, content, &recorded(&won.id)?)
            .map_err(|e| format!("{}'s recorded playthrough: {e}", won.id))?;
        let unused = state.rewind_charges();
        campaign
            .apply_result(won, &state, unused)
            .map_err(|e| format!("{}'s recorded playthrough: {e}", won.id))?;
    }
    recruit(&mut campaign, content, &chapters[at].id);
    Ok(Some(setup(content, &campaign, def, chapters[at])))
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
