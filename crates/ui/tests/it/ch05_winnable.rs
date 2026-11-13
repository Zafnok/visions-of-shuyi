//! Chapter 5 can be won (ticket 0851): the player's commands in
//! `fixtures/ch05_winning.ron`, played with the army Chapters 1 to 4's
//! recorded playthroughs leave (`ch01_winnable` to `ch04_winnable`),
//! `shieldbearer` joined in a scene before the battle, and Preparations
//! left as they are, win battle `ch05` (a hold to turn 10) with nobody
//! lost. On the way the army wins the fixed skirmish `salt_road`, which
//! guards the way (1071), with `fixtures/salt_road_winning.ron`. How the
//! fixtures are played and recorded: [`playthrough`].
//!
//! Tests of the game's content (ADR-0118 §1 (a)): each starts with the
//! guard, and skips without the game's content.
//!
//! **If a balance change breaks this test** (or a new Chapter 1 to 4
//! playthrough), the change must update the playthrough on purpose: look
//! at what it did to the chapter, then record a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch05_winnable::record -- --ignored
//! ```
//!
//! (`ch05_winnable::record_salt_road` for `salt_road`, and then Chapter
//! 5's again.)
//!
//! and read the new fixture's last lines (who fell, how many turns).

use trpg_content::Content;
use trpg_content::bundle::skip_without_the_game;
use trpg_content::chapter::roster_units;
use trpg_core::{BattleState, Campaign, CharacterId, Outcome, SavePoint};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::ch04_winnable::{
    chapter_4, chapter_4_won, win_on_the_way, winning_commands as chapter_4_commands, won,
};
use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch05_winning.ron");
const SALT_ROAD: &str = include_str!("fixtures/salt_road_winning.ron");

fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch05_winning.ron", FIXTURE)
}

fn salt_road_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("salt_road_winning.ron", SALT_ROAD)
}

/// The campaign as Chapters 1 to 4's recorded playthroughs leave it, and
/// battle `salt_road` as it starts with it, Preparations left as they
/// are.
fn salt_road(content: &Content) -> (Campaign, BattleState) {
    let (mut campaign, mut state) = chapter_4(content);
    playthrough::play(&mut state, content, &chapter_4_commands());
    won(content, &mut campaign, &state, "ch04");
    let def = &content.battles["salt_road"];
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    (campaign, state)
}

/// The campaign as Chapters 1 to 4's recorded playthroughs and
/// `salt_road`'s leave it (every rewind charge unused, as the game counts
/// them), `shieldbearer` joined in a scene before the battle, and Chapter
/// 5's battle as it starts with it, Preparations left as they are.
fn chapter_5(content: &Content) -> (Campaign, BattleState) {
    let (mut campaign, mut state) = salt_road(content);
    playthrough::play(&mut state, content, &salt_road_commands());
    won(content, &mut campaign, &state, "salt_road");
    for recruit in roster_units(content, &[CharacterId("shieldbearer".into())]) {
        campaign.joins_in_scene(recruit);
    }
    let def = &content.battles["ch05"];
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    (campaign, state)
}

/// The rules alone: the fixture's commands win `salt_road`, in Classic,
/// losing nobody.
#[test]
fn the_recorded_playthrough_wins_salt_road() {
    if skip_without_the_game() {
        return;
    }
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = salt_road(&content);
    playthrough::play(&mut state, &content, &salt_road_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
}

/// The rules alone: the fixture's commands hold until the last phase of
/// turn 10 ends, in Classic, losing nobody.
#[test]
fn the_recorded_playthrough_wins_chapter_5() {
    if skip_without_the_game() {
        return;
    }
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = chapter_5(&content);
    assert_eq!(state.units().len(), 8 + 9, "eight and 9 enemies");
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    assert!(state.turn() >= 10, "won as turn 10 ends, not before");
}

/// Acceptance: New Game reaches Chapter 5 through Chapters 1 to 4, and
/// Chapter 5 plays through the real game: its first scene, the Act 1
/// world map, `salt_road` won on the way to `envoys_house`, the arrival's
/// scenes (`shieldbearer` joins in one), Preparations, the hold won with
/// the recorded commands, the results, the victory scenes, "Save your
/// progress?" (0869: the map won't come back), a world-map save, "To be
/// continued".
#[test]
fn chapter_5_plays_from_new_game_to_to_be_continued() {
    if skip_without_the_game() {
        return;
    }
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_4_won(&mut h);
    // Find, Fixed skirmishes: `salt_road`, which opened with `veyne_hall`
    // won and guards the way (1071); travel there and win it.
    h.keys("x Down Down Down Down f f");
    h.wait(3.0);
    win_on_the_way(&mut h, "salt_road", &salt_road_commands());
    // Find, Story battles, the next one from `salt_road` where the army
    // is: `envoys_house`; close the list and travel there. Arriving plays
    // its scenes, then Preparations; Left wraps to `Fight!`.
    h.keys("x Down Down s x f");
    h.wait(6.0);
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
    assert_eq!(battle(&h).units().len(), 8 + 9, "eight and 9 enemies");
    playthrough::play_in_game(&mut h, &winning_commands());
    for _ in 0..150 {
        match h.top_screen() {
            "dialogue" => skip_scenes(&mut h),
            "save_prompt" => break,
            // The banners and the results (a page for each level gained)
            // take a press.
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "save_prompt"],
        "{}",
        h.snapshot()
    );
    // Yes, into the first empty slot: a save on the map, the battle won.
    h.keys("f f");
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    let saved = trpg_ui::save::read(&h.game().ctx().storage, &trpg_ui::save::slot_key(1));
    let saved = saved
        .ok()
        .flatten()
        .unwrap_or_else(|| panic!("nothing saved"));
    assert_eq!(saved.point, SavePoint::WorldMap);
    let world = saved.campaign.world.as_ref();
    assert!(world.is_some_and(|w| w.cleared.contains("envoys_house")));
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    let recruit = CharacterId("shieldbearer".into());
    let joined = campaign
        .roster
        .iter()
        .any(|u| u.character.as_ref() == Some(&recruit));
    assert!(joined, "shieldbearer joined");
    assert_eq!(campaign.roster.len(), 8, "nobody was lost");
    let world = campaign.world.as_ref();
    let cleared = world.is_some_and(|w| w.cleared.contains("envoys_house"));
    assert!(cleared, "envoys_house is won");
}

/// Records a new `fixtures/salt_road_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record_salt_road() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, start) = salt_road(&content);
    playthrough::record(
        start,
        &content,
        1073,
        &[
            "// A winning playthrough of battle salt_road (ticket 1071): the player's",
            "// commands, phase by phase, with the army Chapters 1 to 4's playthroughs leave.",
            "// Played back by ch05_winnable.rs, which says how to record a new one. Units",
            "// 1-7: the New Game roster, then Chapter 2's recruit.",
        ],
        "tests/it/fixtures/salt_road_winning.ron",
    );
}

/// Records a new `fixtures/ch05_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, start) = chapter_5(&content);
    playthrough::record(
        start,
        &content,
        851,
        &[
            "// A winning playthrough of Chapter 5 (ticket 0851): the player's commands,",
            "// phase by phase, with the army Chapters 1 to 4's playthroughs leave and",
            "// Chapter 5's recruit. Played back by ch05_winnable.rs, which says how to",
            "// record a new one. Units 1-8: the lead, the recruit, the rest of the New",
            "// Game roster, then Chapter 2's recruit.",
        ],
        "tests/it/fixtures/ch05_winning.ron",
    );
}
