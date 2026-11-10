//! Chapter 4 can be won (ticket 0850): the player's commands in
//! `fixtures/ch04_winning.ron`, played with the army Chapters 1 to 3's
//! recorded playthroughs leave (`ch01_winnable` to `ch03_winnable`) and
//! Preparations left as they are, beat Dace at Veyne Hall: he retreats at
//! 0 HP, which wins, and nobody is lost. On the way the army wins
//! Aldwater Toll Bridge, which guards the road to the hall (1071), with
//! `fixtures/aldwater_winning.ron`. How the fixtures are played and
//! recorded: [`playthrough`].
//!
//! **If a balance change breaks this test** (or a new Chapter 1, 2 or 3
//! playthrough), the change must update the playthrough on purpose: look
//! at what it did to the chapter, then record a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch04_winnable::record -- --ignored
//! ```
//!
//! (`ch04_winnable::record_aldwater` for the toll bridge, and then
//! Chapter 4's again.)
//!
//! and read the new fixture's last lines (who fell, how many turns). A new
//! Chapter 4 playthrough leaves Chapter 5 another army: record Chapter
//! 5's again after it (`ch05_winnable`).

use trpg_content::Content;
use trpg_core::{BattleState, Campaign, CharacterId, Defeat, Outcome};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::ch03_winnable::{chapter_3, chapter_3_won, winning_commands as chapter_3_commands};
use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch04_winning.ron");
const ALDWATER: &str = include_str!("fixtures/aldwater_winning.ron");

/// The player's commands, in order.
pub fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch04_winning.ron", FIXTURE)
}

fn aldwater_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("aldwater_winning.ron", ALDWATER)
}

/// `campaign` after `state`, battle `battle`, is won: every rewind charge
/// unused, as the game counts them.
pub fn won(content: &Content, campaign: &mut Campaign, state: &BattleState, battle: &str) {
    let unused = state.rewind_charges();
    campaign
        .apply_result(&content.battles[battle], state, unused)
        .unwrap_or_else(|e| panic!("{e:?}"));
}

/// The campaign as Chapters 1 to 3's recorded playthroughs leave it, and
/// Aldwater Toll Bridge's battle as it starts with it, Preparations left
/// as they are.
fn aldwater(content: &Content) -> (Campaign, BattleState) {
    let (mut campaign, mut state) = chapter_3(content);
    playthrough::play(&mut state, content, &chapter_3_commands());
    won(content, &mut campaign, &state, "ch03");
    let def = &content.battles["aldwater"];
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    (campaign, state)
}

/// The campaign as Chapters 1 to 3's recorded playthroughs and the toll
/// bridge's leave it, and Chapter 4's battle as it starts with it,
/// Preparations left as they are. Gil isn't in it: he joins only on
/// Millhaven Road, which is optional (1071).
pub fn chapter_4(content: &Content) -> (Campaign, BattleState) {
    let (mut campaign, mut state) = aldwater(content);
    playthrough::play(&mut state, content, &aldwater_commands());
    won(content, &mut campaign, &state, "aldwater");
    let def = &content.battles["ch04"];
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    (campaign, state)
}

/// The rules alone: the fixture's commands rout the toll bridge's Hounds,
/// in Classic, losing nobody.
#[test]
fn the_recorded_playthrough_takes_the_toll_bridge() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = aldwater(&content);
    playthrough::play(&mut state, &content, &aldwater_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
}

/// The rules alone: the fixture's commands beat Dace, in Classic, losing
/// nobody; he is defeated by retreating, not falling, and that wins.
#[test]
fn the_recorded_playthrough_drives_dace_from_the_hall() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = chapter_4(&content);
    assert_eq!(state.units().len(), 7 + 13, "seven and the hall");
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    let dace = CharacterId("rival".into());
    let mut beaten = state.fallen().iter();
    let dace = beaten.find(|u| u.character.as_ref() == Some(&dace));
    assert_eq!(dace.map(|u| u.defeat), Some(Defeat::Retreats));
}

/// New Game, then Chapters 1 to 4 won with the recorded commands through
/// the real game (their scenes skipped, Preparations left as they are),
/// up to the Act 1 world map as Chapter 5 begins, after its road south.
pub fn chapter_4_won(h: &mut Harness) {
    chapter_3_won(h);
    // "Save your progress?" No; the valley road, then the map.
    h.keys("Down f");
    assert_eq!(h.top_screen(), "dialogue", "{}", h.snapshot());
    skip_scenes(h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "world_map"],
        "{}",
        h.snapshot()
    );
    // Find → Story battles: Veyne Hall; travel there. The toll bridge
    // guards the road (1071): the army stops there and fights first.
    h.keys("x Down Down f f");
    h.wait(3.0);
    win_on_the_way(h, "aldwater", &aldwater_commands());
    // On to the hall. Arriving plays the rise and the gate, then
    // Preparations; Left wraps to `Fight!`.
    h.keys("x Down Down f f");
    h.wait(3.0);
    assert_eq!(h.top_screen(), "dialogue", "{}", h.snapshot());
    skip_scenes(h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "preparations"],
        "{}",
        h.snapshot()
    );
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(h).units().len(), 7 + 13, "seven and the hall");
    playthrough::play_in_game(h, &winning_commands());
    for _ in 0..150 {
        match h.top_screen() {
            "dialogue" => skip_scenes(h),
            "world_map" => break,
            // Dace runs and his Hounds after him; banners and the results
            // (a page for each level gained) take a press.
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
}

/// Wins the fixed skirmish the army has stopped at on the map, place
/// `place`, with `commands`: its scene, Preparations (left as they are),
/// the battle, the results and the scene after, and the map again with the
/// place cleared.
pub fn win_on_the_way(h: &mut Harness, place: &str, commands: &[trpg_core::Command]) {
    assert_eq!(h.top_screen(), "dialogue", "{}", h.snapshot());
    skip_scenes(h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "preparations"],
        "{}",
        h.snapshot()
    );
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    playthrough::play_in_game(h, commands);
    for _ in 0..60 {
        match h.top_screen() {
            "dialogue" => skip_scenes(h),
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
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    let world = campaign.world.as_ref();
    assert!(
        world.is_some_and(|w| w.cleared.contains(place)),
        "{place} won"
    );
}

/// Acceptance: New Game reaches Chapter 4 through Chapters 1 to 3, and
/// Chapter 4 plays through the real game: the valley road, the Act 1
/// world map, the march to Veyne Hall, the toll bridge on the way, the
/// rise and the gate,
/// Preparations, the battle won with the recorded commands, the results,
/// the stable, the Hound post, and on to Chapter 5 (0851): its road
/// south, and the map again.
#[test]
fn chapter_4_plays_from_new_game_into_chapter_5() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_4_won(&mut h);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let chapter = flow.chapter().map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch05"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.roster.len(), 7, "nobody was lost");
    let world = campaign.world.as_ref();
    let cleared = world.is_some_and(|w| w.cleared.contains("veyne_hall"));
    assert!(cleared, "Veyne Hall is won");
}

/// Records a new `fixtures/aldwater_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record_aldwater() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, start) = aldwater(&content);
    playthrough::record(
        start,
        &content,
        1071,
        &[
            "// A winning playthrough of Aldwater Toll Bridge (ticket 1071): the player's",
            "// commands, phase by phase, with the army Chapters 1 to 3's playthroughs",
            "// leave. Played back by ch04_winnable.rs, which says how to record a new one.",
            "// Units 1-7: the lead, Hollis, Tamsin, Aske, Maud, Rue, Joss.",
        ],
        "tests/it/fixtures/aldwater_winning.ron",
    );
}

/// Records a new `fixtures/ch04_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, start) = chapter_4(&content);
    playthrough::record(
        start,
        &content,
        850,
        &[
            "// A winning playthrough of Chapter 4 (ticket 0850): the player's commands,",
            "// phase by phase, with the army Chapters 1 to 3's playthroughs leave. Played",
            "// back by ch04_winnable.rs, which says how to record a new one. Units 1-7: the",
            "// lead, Hollis, Tamsin, Aske, Maud, Rue, Joss.",
        ],
        "tests/it/fixtures/ch04_winning.ron",
    );
}
