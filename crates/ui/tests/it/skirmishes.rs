//! Act 1's fixed skirmishes of Chapters 4 and 5 (ticket 1071) on the real
//! Act 1 map: `aldwater` and `salt_road` guard the way to their chapters'
//! story battles, `millhaven_road` (whose boss, `prizefighter`, joins if
//! beaten) and `smugglers_cove` are optional. The guards' recorded
//! playthroughs are part of Chapters 4 and 5's (`ch04_winnable`,
//! `ch05_winnable`), played with the army those chapters bring; the
//! optional ones' (`fixtures/<battle>_winning.ron`) with the army the
//! battle's `bots_army` describes, as the side quests' are
//! (`side_quests`).
//!
//! Tests of the game's content (ADR-0118 §1 (a)): each starts with the
//! guard, and skips without the game's content.
//!
//! **If a balance change breaks a replay**, record a new one on purpose:
//!
//! ```text
//! cargo test -p trpg-ui --test it skirmishes::record_ -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns).

use trpg_content::bundle::skip_without_the_game;
use trpg_core::{BattleState, CharacterId, ItemId, NodeKind, Outcome, WorldCommand, WorldEvent};

use crate::playthrough::{self, lost, skip_scenes};
use crate::side_quests::{act1_from_a_save, army_for, campaign, content, on_act1};

const MILLHAVEN_ROAD: &str = include_str!("fixtures/millhaven_road_winning.ron");
const SMUGGLERS_COVE: &str = include_str!("fixtures/smugglers_cove_winning.ron");

/// Each place is a fixed skirmish of its own battle and scenes, and only
/// the two on the way guard it.
#[test]
fn the_four_places_are_their_fixed_skirmishes() {
    if skip_without_the_game() {
        return;
    }
    let content = content();
    let map = &content.worldmaps["act1"].map;
    for (place, guards, intro) in [
        ("aldwater", true, "ch04_aldwater_intro"),
        ("millhaven_road", false, "ch04_millhaven_road_intro"),
        ("salt_road", true, "ch05_salt_road_intro"),
        ("smugglers_cove", false, "ch05_cove_intro"),
    ] {
        let node = map.node(place).unwrap_or_else(|| panic!("no {place}"));
        let NodeKind::FixedSkirmish {
            battle,
            guards_path,
        } = &node.kind
        else {
            panic!("{place} is {:?}", node.kind);
        };
        assert_eq!(battle.battle, place);
        assert_eq!(*guards_path, guards, "{place}");
        assert_eq!(battle.intro_scenes, [intro]);
        assert!(content.battles.contains_key(place), "{place}'s battle");
    }
}

/// Acceptance: a guard stops the army on its way to the story battle
/// until it is won. `aldwater` stands between `valley_road` and
/// `veyne_hall`; `salt_road` between `veyne_hall` and `saltmere`, which,
/// with `envoys_house` and `smugglers_cove`, opens only once it is won.
#[test]
fn a_guard_blocks_the_way_until_won() {
    if skip_without_the_game() {
        return;
    }
    let content = content();
    let map = &content.worldmaps["act1"].map;
    // Any army will do: the map reads only what is won.
    let (mut c, _) = army_for(&content, "millhaven_road");
    // `on_act1` puts the campaign in Chapter 2; `veyne_hall` waits for 4.
    on_act1(&mut c, "valley_road", &[]);
    c.chapter = "ch04".into();
    let to_hall = WorldCommand::TravelTo("veyne_hall".into());
    let events = c.world_command(map, &to_hall);
    let events = events.unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(
        events.last(),
        Some(&WorldEvent::BattleAhead("aldwater".into()))
    );
    on_act1(&mut c, "valley_road", &["aldwater"]);
    c.chapter = "ch04".into();
    let events = c.world_command(map, &to_hall);
    let events = events.unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(
        events.last(),
        Some(&WorldEvent::BattleAhead("veyne_hall".into()))
    );

    on_act1(&mut c, "veyne_hall", &["aldwater", "veyne_hall"]);
    c.chapter = "ch05".into();
    for closed in ["saltmere", "envoys_house", "smugglers_cove"] {
        assert!(!map.is_open(closed, &c), "{closed} before salt_road");
    }
    assert!(map.blocks_path("salt_road", &c));
    on_act1(
        &mut c,
        "salt_road",
        &["aldwater", "veyne_hall", "salt_road"],
    );
    c.chapter = "ch05".into();
    for open in ["saltmere", "envoys_house", "smugglers_cove"] {
        assert!(map.is_open(open, &c), "{open} after salt_road");
    }
    // The optional ones never stop travel.
    on_act1(&mut c, "millhaven", &[]);
    assert!(!map.blocks_path("millhaven_road", &c));
    assert!(!map.blocks_path("smugglers_cove", &c));
}

/// The rules alone: the fixture's commands beat `prizefighter` before the
/// battle is won, in Classic, losing nobody: it is recruited as it falls,
/// and joins the army when the win is applied.
#[test]
fn the_recorded_playthrough_wins_millhaven_road_with_its_recruit() {
    if skip_without_the_game() {
        return;
    }
    let content = content();
    let (mut campaign, def) = army_for(&content, "millhaven_road");
    let mut state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    let commands = playthrough::commands("millhaven_road", MILLHAVEN_ROAD);
    playthrough::play(&mut state, &content, &commands);
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    let recruit = CharacterId("prizefighter".into());
    let recruited = state.recruited().iter();
    let recruited: Vec<_> = recruited.filter_map(|u| u.character.clone()).collect();
    assert_eq!(recruited, std::slice::from_ref(&recruit));
    campaign
        .apply_result(def, &state, 0)
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert!(campaign.member(&recruit).is_some(), "prizefighter joined");
}

/// The rules alone: the fixture's commands win `smugglers_cove`, in
/// Classic, losing nobody; the gold and an elixir are the loot.
#[test]
fn the_recorded_playthrough_wins_smugglers_cove() {
    if skip_without_the_game() {
        return;
    }
    let content = content();
    let (mut campaign, def) = army_for(&content, "smugglers_cove");
    let mut state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    let commands = playthrough::commands("smugglers_cove", SMUGGLERS_COVE);
    playthrough::play(&mut state, &content, &commands);
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    let (gold, elixirs) = (campaign.gold, campaign.stock.count(&ItemId::new("elixir")));
    campaign
        .apply_result(def, &state, 0)
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(campaign.gold, gold + 800);
    assert_eq!(campaign.stock.count(&ItemId::new("elixir")), elixirs + 1);
}

/// Acceptance: from `millhaven`, `millhaven_road` is a place the army can
/// walk to; its scene plays, then Preparations, the battle (won with the
/// recorded commands, `prizefighter` beaten), the results, the scene
/// after, and the map again with the place cleared and the recruit in the
/// army.
#[test]
fn millhaven_road_plays_from_the_map_to_the_map_with_its_recruit() {
    if skip_without_the_game() {
        return;
    }
    let mut h = act1_from_a_save("millhaven", &[], "millhaven_road");
    let before = campaign(&h);
    // Find → Fixed skirmishes: the nearest, `millhaven_road`; travel there.
    h.keys("x Down Down Down Down f f");
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
    let commands = playthrough::commands("millhaven_road", MILLHAVEN_ROAD);
    playthrough::play_in_game(&mut h, &commands);
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
    let world = after.world.as_ref();
    assert!(world.is_some_and(|w| w.cleared.contains("millhaven_road")));
    assert_eq!(after.roster.len(), before.roster.len() + 1, "one joined");
    assert!(after.member(&CharacterId("prizefighter".into())).is_some());
}

/// Records a new `fixtures/millhaven_road_winning.ron`
/// ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record_millhaven_road() {
    let content = content();
    let (campaign, def) = army_for(&content, "millhaven_road");
    let start = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    let recruit_beaten = |state: &BattleState| !state.recruited().is_empty();
    playthrough::record_where(
        start,
        &content,
        1072,
        &[
            "// A winning playthrough of battle millhaven_road (ticket 1071), its recruit",
            "// beaten: the player's commands, phase by phase, with the army the battle's",
            "// bots_army describes. Played back by skirmishes.rs, which says how to record",
            "// a new one. Units 1-7: the battle's slots, in order.",
        ],
        "tests/it/fixtures/millhaven_road_winning.ron",
        &recruit_beaten,
    );
}

/// Records a new `fixtures/smugglers_cove_winning.ron`.
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record_smugglers_cove() {
    let content = content();
    let (campaign, def) = army_for(&content, "smugglers_cove");
    let start = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    playthrough::record(
        start,
        &content,
        1074,
        &[
            "// A winning playthrough of battle smugglers_cove (ticket 1071): the player's",
            "// commands, phase by phase, with the army the battle's bots_army describes.",
            "// Played back by skirmishes.rs, which says how to record a new one. Units",
            "// 1-9: the battle's slots, in order.",
        ],
        "tests/it/fixtures/smugglers_cove_winning.ron",
    );
}
