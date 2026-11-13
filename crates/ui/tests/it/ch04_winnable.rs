//! Chapter 4 can be won (ticket 0850): the player's commands in
//! `fixtures/ch04_winning.ron`, played with the army Chapters 1 to 3's
//! recorded playthroughs leave (`ch01_winnable` to `ch03_winnable`) and
//! Preparations left as they are, win battle `ch04` (its boss, `rival`,
//! retreats at 0 HP, which wins) with nobody lost. On the way the army
//! wins the fixed skirmish `aldwater`, which guards the road (1071), with
//! `fixtures/aldwater_winning.ron`. How the fixtures are played and
//! recorded: [`playthrough`].
//!
//! Tests of the game's content (ADR-0118 §1 (a)): each starts with the
//! guard, and skips without the game's content.
//!
//! **If a balance change breaks this test** (or a new Chapter 1, 2 or 3
//! playthrough), the change must update the playthrough on purpose: look
//! at what it did to the chapter, then record a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch04_winnable::record -- --ignored
//! ```
//!
//! (`ch04_winnable::record_aldwater` for `aldwater`, and then
//! Chapter 4's again.)
//!
//! and read the new fixture's last lines (who fell, how many turns). A new
//! Chapter 4 playthrough leaves Chapter 5 another army: record Chapter
//! 5's again after it (`ch05_winnable`).

use trpg_content::Content;
use trpg_content::bundle::skip_without_the_game;
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
/// battle `aldwater` as it starts with it, Preparations left as they are.
fn aldwater(content: &Content) -> (Campaign, BattleState) {
    let (mut campaign, mut state) = chapter_3(content);
    playthrough::play(&mut state, content, &chapter_3_commands());
    won(content, &mut campaign, &state, "ch03");
    let def = &content.battles["aldwater"];
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    (campaign, state)
}

/// The campaign as Chapters 1 to 3's recorded playthroughs and
/// `aldwater`'s leave it, and Chapter 4's battle as it starts with it,
/// Preparations left as they are. The recruit of the optional skirmish
/// `millhaven_road` isn't in it (1071).
pub fn chapter_4(content: &Content) -> (Campaign, BattleState) {
    let (mut campaign, mut state) = aldwater(content);
    playthrough::play(&mut state, content, &aldwater_commands());
    won(content, &mut campaign, &state, "aldwater");
    let def = &content.battles["ch04"];
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    (campaign, state)
}

/// The rules alone: the fixture's commands win `aldwater`, in Classic,
/// losing nobody.
#[test]
fn the_recorded_playthrough_wins_aldwater() {
    if skip_without_the_game() {
        return;
    }
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = aldwater(&content);
    playthrough::play(&mut state, &content, &aldwater_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
}

/// The rules alone: the fixture's commands win, in Classic, losing
/// nobody; the boss is defeated by retreating, not falling, and that wins.
#[test]
fn the_recorded_playthrough_wins_chapter_4() {
    if skip_without_the_game() {
        return;
    }
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = chapter_4(&content);
    assert_eq!(state.units().len(), 7 + 13, "seven and 13 enemies");
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    let boss = CharacterId("rival".into());
    let mut beaten = state.fallen().iter();
    let boss = beaten.find(|u| u.character.as_ref() == Some(&boss));
    assert_eq!(boss.map(|u| u.defeat), Some(Defeat::Retreats));
}

/// New Game, then Chapters 1 to 4 won with the recorded commands through
/// the real game (their scenes skipped, Preparations left as they are),
/// up to the Act 1 world map as Chapter 5 begins, after its road south.
pub fn chapter_4_won(h: &mut Harness) {
    chapter_3_won(h);
    // "Save your progress?" No; a scene, then the map.
    h.keys("Down f");
    assert_eq!(h.top_screen(), "dialogue", "{}", h.snapshot());
    skip_scenes(h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "world_map"],
        "{}",
        h.snapshot()
    );
    // Find → Story battles: `veyne_hall`; travel there. `aldwater`
    // guards the road (1071): the army stops there and fights first.
    h.keys("x Down Down f f");
    h.wait(3.0);
    win_on_the_way(h, "aldwater", &aldwater_commands());
    // On to `veyne_hall`. Arriving plays its scenes, then Preparations;
    // Left wraps to `Fight!`.
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
    assert_eq!(battle(h).units().len(), 7 + 13, "seven and 13 enemies");
    playthrough::play_in_game(h, &winning_commands());
    for _ in 0..150 {
        match h.top_screen() {
            "dialogue" => skip_scenes(h),
            "world_map" => break,
            // The boss retreats and the rest after it; banners and the
            // results (a page for each level gained) take a press.
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
/// Chapter 4 plays through the real game: its first scene, the Act 1
/// world map, the march to `veyne_hall`, `aldwater` on the way, the
/// arrival's scenes, Preparations, the battle won with the recorded
/// commands, the results, the victory scenes, and on to Chapter 5 (0851):
/// its first scene, and the map again.
#[test]
fn chapter_4_plays_from_new_game_into_chapter_5() {
    if skip_without_the_game() {
        return;
    }
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_4_won(&mut h);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let chapter = flow.chapter().map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch05"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.roster.len(), 7, "nobody was lost");
    let world = campaign.world.as_ref();
    let cleared = world.is_some_and(|w| w.cleared.contains("veyne_hall"));
    assert!(cleared, "veyne_hall is won");
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
            "// A winning playthrough of battle aldwater (ticket 1071): the player's",
            "// commands, phase by phase, with the army Chapters 1 to 3's playthroughs",
            "// leave. Played back by ch04_winnable.rs, which says how to record a new one.",
            "// Units 1-7: the New Game roster, then Chapter 2's recruit.",
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
            "// New Game roster, then Chapter 2's recruit.",
        ],
        "tests/it/fixtures/ch04_winning.ron",
    );
}
