//! Act 1's side quests (ticket 1072): *The Company Ledger* (Tamsin's) and
//! *Letters Home* (Aske's) on the real Act 1 map. Each opens under its
//! condition and not otherwise; each can be won by its recorded
//! playthrough (`fixtures/side_*_winning.ron`, played with the army the
//! battle's `bots_army` describes, raised by average growths as the bots
//! raise theirs) and gives its reward; and *The Company Ledger* plays
//! through the real game from the
//! map: its scene, Preparations, the battle, the results, the strongroom,
//! and the map again with the place cleared and the gold in the purse.
//!
//! **If a balance change breaks a replay**, record a new one on purpose:
//!
//! ```text
//! cargo test -p trpg-ui --test it side_quests::record_ -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns).

use trpg_content::{Content, battle_campaign};
use trpg_core::{
    BattleDef, BattleState, Campaign, CharacterId, GameMode, ItemId, LeadGender, LeadProfile,
    Level, Outcome, StatKind, StatValue, Stats, Unit, WorldState, growth,
};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::WorldMapScreen;
use trpg_ui::{Ctx, MemoryStorage, ShuyiCtx, Storage};

use crate::playthrough::{self, battle, lost, skip_scenes};

const LEDGER: &str = include_str!("fixtures/side_company_ledger_winning.ron");
const LETTERS: &str = include_str!("fixtures/side_letters_home_winning.ron");

pub fn lead() -> LeadProfile {
    LeadProfile::new("Mara", LeadGender::Female)
}

pub fn content() -> Content {
    trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

/// The army a player brings to quest `battle`: the characters of its
/// slots at the level its `bots_army` says, each raised by its class's
/// average growths as the bots raise theirs (`docs/playtesting.md`, *The
/// army*; the bots crate is dev tooling the game's crates never depend
/// on, ADR-0033, so the raising is repeated here), with the character
/// data's gear and the battle's own stock and gold.
pub fn army_for<'a>(content: &'a Content, battle: &str) -> (Campaign, &'a BattleDef) {
    let def = &content.battles[battle];
    let mut campaign = battle_campaign(content, def, GameMode::Classic, lead());
    let army = def
        .bots_army
        .as_ref()
        .unwrap_or_else(|| panic!("{battle} has no bots_army"));
    for unit in &mut campaign.roster {
        raise(unit, content, army.level);
    }
    (campaign, def)
}

/// `unit` at `level` (never lower than it is): its class's average growths
/// for each level gained, within the hard ceilings; the personal spells it
/// has reached learned; rested.
fn raise(unit: &mut Unit, content: &Content, level: Level) {
    let Some(class) = content.classes.get(&unit.class) else {
        return;
    };
    let levels = StatValue::try_from(level.saturating_sub(unit.level)).unwrap_or(StatValue::MAX);
    let ceilings = &content.classes.hard_ceilings;
    let gain = |kind: StatKind, value: StatValue| {
        let points = StatValue::from(growth(unit, class, kind)).saturating_mul(levels) / 100;
        value.saturating_add(points).min(ceilings.get(kind))
    };
    let s = unit.stats;
    unit.stats = Stats {
        hp: gain(StatKind::Hp, s.hp),
        str: gain(StatKind::Str, s.str),
        mag: gain(StatKind::Mag, s.mag),
        dex: gain(StatKind::Dex, s.dex),
        spd: gain(StatKind::Spd, s.spd),
        def: gain(StatKind::Def, s.def),
        res: gain(StatKind::Res, s.res),
        ..s
    };
    unit.level = unit.level.max(level);
    unit.exp = 0;
    let learned: Vec<_> = unit
        .personal_spells
        .iter()
        .filter(|(l, _)| *l <= level)
        .map(|(_, s)| s.clone())
        .collect();
    unit.learned.extend(learned);
    unit.hp = unit.stats.hp;
}

/// Puts `campaign` on the Act 1 map at `at`, with `cleared` won.
pub fn on_act1(campaign: &mut Campaign, at: &str, cleared: &[&str]) {
    campaign.chapter = "ch02".into();
    campaign.world = Some(WorldState {
        map: "act1".into(),
        at: at.into(),
        cleared: cleared.iter().map(|&s| s.to_owned()).collect(),
        ..WorldState::default()
    });
}

fn without(campaign: &mut Campaign, character: &str) {
    let gone = CharacterId(character.into());
    campaign
        .roster
        .retain(|u| u.character.as_ref() != Some(&gone));
}

/// Both quests open once their battle is won while their character is in
/// the army, and not before, and not without them.
#[test]
fn each_quest_opens_under_its_condition_and_not_otherwise() {
    let content = content();
    let map = &content.worldmaps["act1"].map;
    for (place, battle, after, who) in [
        (
            "company_ledger",
            "side_company_ledger",
            "aldwater",
            "sergeant",
        ),
        (
            "letters_home",
            "side_letters_home",
            "envoys_house",
            "poacher",
        ),
    ] {
        let (mut c, _) = army_for(&content, battle);
        on_act1(&mut c, "millhaven", &[]);
        assert!(!map.is_open(place, &c), "{place} before {after}");
        on_act1(&mut c, "millhaven", &[after]);
        assert!(map.is_open(place, &c), "{place} after {after}");
        without(&mut c, who);
        assert!(!map.is_open(place, &c), "{place} without {who}");
    }
}

/// The rules alone: the fixture's commands win *The Company Ledger* in
/// Classic, losing nobody; the paymaster is beaten (he retreats, which
/// wins); the win pays the company's number.
#[test]
fn the_recorded_playthrough_takes_the_barn() {
    let content = content();
    let (mut campaign, def) = army_for(&content, "side_company_ledger");
    let mut state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    playthrough::play(
        &mut state,
        &content,
        &playthrough::commands("ledger", LEDGER),
    );
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    // The paymaster is defeated (he retreats, which counts), so the win
    // is his retreat, not a rout.
    let tull = CharacterId("paymaster".into());
    let defeated = state
        .fallen()
        .iter()
        .any(|u| u.character.as_ref() == Some(&tull));
    assert!(defeated, "the paymaster was beaten");
    let before = campaign.gold;
    campaign
        .apply_result(def, &state, 0)
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(campaign.gold, before + 4212);
}

/// The rules alone: the fixture's commands win *Letters Home* in Classic,
/// losing nobody; the win puts Tor's Bow in the stock, and only Aske can
/// wield it.
#[test]
fn the_recorded_playthrough_frees_the_boat() {
    let content = content();
    let (mut campaign, def) = army_for(&content, "side_letters_home");
    let mut state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    playthrough::play(
        &mut state,
        &content,
        &playthrough::commands("letters", LETTERS),
    );
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    campaign
        .apply_result(def, &state, 0)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let bow = ItemId::new("tors_bow");
    assert!(campaign.stock.weapons.iter().any(|w| w.def == bow));
    let def = content
        .items
        .weapon(&bow)
        .unwrap_or_else(|| panic!("no bow"));
    let wielder = def.special.as_ref().and_then(|s| s.wielder.clone());
    assert_eq!(wielder, Some(CharacterId("poacher".into())));
}

/// A launch from a save on the Act 1 map at `at` with `cleared` won and
/// the army of quest `battle`, loaded from the title.
pub fn act1_from_a_save(at: &str, cleared: &[&str], battle: &str) -> Harness {
    let mut storage: Box<dyn Storage> = Box::new(MemoryStorage::new());
    if let Err(e) = storage.write("layout", RIGHT_HANDED.name()) {
        panic!("saving the layout: {e}");
    }
    let ctx = Ctx::embedded().unwrap_or_else(|e| panic!("{e}"));
    let (mut c, _) = army_for(&ctx.game.content, battle);
    on_act1(&mut c, at, cleared);
    let file = trpg_core::SaveFile::world_map(c);
    trpg_ui::save::write(storage.as_mut(), &trpg_ui::save::slot_key(1), &file)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let ctx = ctx.with_storage(storage);
    let mut h = Harness::from_game(trpg_ui::game::start(ctx));
    // The title's Load Game, the first slot.
    h.keys("Down f f");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    h
}

pub fn campaign(h: &Harness) -> Campaign {
    let map = h.game().screen::<WorldMapScreen>();
    map.unwrap_or_else(|| panic!("no world map"))
        .campaign()
        .clone()
}

/// Acceptance: from Millhaven with the toll bridge won, *The Company
/// Ledger* is a place the army can walk to; its scene plays, then
/// Preparations, the battle (won with the recorded commands), the
/// results, the strongroom, and the map again with the place cleared and
/// the company's number in the purse.
#[test]
fn the_company_ledger_plays_from_the_map_to_the_map() {
    let mut h = act1_from_a_save("millhaven", &["aldwater"], "side_company_ledger");
    let before = campaign(&h);
    // Find → Side quests: the one entry; travel there. Arriving starts the
    // quest: the barn gate, then Preparations; Left wraps to `Fight!`.
    h.keys("x Down Down Down f f");
    h.wait(2.0);
    assert_eq!(h.top_screen(), "dialogue", "{}", h.snapshot());
    skip_scenes(&mut h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "preparations"],
        "{}",
        h.snapshot()
    );
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(
        battle(&h).units().len(),
        7 + 15,
        "seven, and the barn's men"
    );
    // The notes and the banner.
    h.keys("f f");
    playthrough::play_in_game(&mut h, &playthrough::commands("ledger", LEDGER));
    for _ in 0..40 {
        match h.top_screen() {
            "dialogue" => skip_scenes(&mut h),
            "world_map" => break,
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "world_map"],
        "{}",
        h.snapshot()
    );
    let after = campaign(&h);
    let world = after
        .world
        .clone()
        .unwrap_or_else(|| panic!("not on a map"));
    assert!(world.cleared.contains("company_ledger"));
    assert_eq!(after.gold, before.gold + 4212);
    assert_eq!(after.roster.len(), before.roster.len(), "nobody was lost");
}

/// Records a new `fixtures/side_company_ledger_winning.ron`
/// ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record_the_company_ledger() {
    let content = content();
    let (campaign, def) = army_for(&content, "side_company_ledger");
    let start = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    playthrough::record(
        start,
        &content,
        1072,
        &[
            "// A winning playthrough of The Company Ledger (ticket 1072): the player's",
            "// commands, phase by phase, with the army the battle's bots_army describes.",
            "// Played back by side_quests.rs, which says how to record a new one. Units",
            "// 1-7: the lead, Hollis, Tamsin, Aske, Maud, Rue, Joss.",
        ],
        "tests/it/fixtures/side_company_ledger_winning.ron",
    );
}

/// Records a new `fixtures/side_letters_home_winning.ron`.
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record_letters_home() {
    let content = content();
    let (campaign, def) = army_for(&content, "side_letters_home");
    let start = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    playthrough::record(
        start,
        &content,
        1073,
        &[
            "// A winning playthrough of Letters Home (ticket 1072): the player's commands,",
            "// phase by phase, with the army the battle's bots_army describes. Played back",
            "// by side_quests.rs, which says how to record a new one. Units 1-7: the lead,",
            "// Hollis, Tamsin, Aske, Maud, Rue, Joss.",
        ],
        "tests/it/fixtures/side_letters_home_winning.ron",
    );
}
