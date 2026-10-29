//! Chapter 1 can be won (ticket 0803): the player's commands in
//! `fixtures/ch01_winning.ron`, sent to the real game from New Game, rout
//! the Red Company with nobody lost, and the chapter runs on to Chapter 2
//! (0848). How the fixture is played and recorded: [`playthrough`].
//!
//! **If a balance change breaks this test**, the change must update the
//! playthrough on purpose: look at what it did to the chapter, then record
//! a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch01_winnable::record -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns). A new
//! Chapter 1 playthrough leaves Chapter 2 another army: record Chapter 2's
//! again after it (`ch02_winnable`).

use trpg_content::{Content, new_campaign};
use trpg_core::{BattleState, Faction, GameMode, LeadGender, LeadProfile, Outcome};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch01_winning.ron");

/// The player's commands, in order.
pub fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch01_winning.ron", FIXTURE)
}

/// The lead every recorded playthrough is played with.
pub fn lead() -> LeadProfile {
    LeadProfile::new("Ellery", LeadGender::Male)
}

/// Chapter 1's battle as a new game starts it.
fn chapter_1(content: &Content) -> BattleState {
    let campaign = new_campaign(content, GameMode::Classic, lead());
    let def = &content.battles["ch01"];
    BattleState::new(campaign.battle_setup(def, &content.tables())).0
}

/// The rules alone: the fixture's commands win, in Classic, losing nobody.
#[test]
fn the_recorded_playthrough_routs_the_red_company() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let mut state = chapter_1(&content);
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    assert_eq!(state.fallen().len(), 14, "13 soldiers and Harl");
}

/// New Game, Classic, Not familiar, the default lead, then Chapter 1 won
/// with the recorded commands (its scenes skipped), up to "Save your
/// progress?".
pub fn chapter_1_won(h: &mut Harness) {
    h.keys("f f f Up f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch01"));
    // The letter, then the Red Company.
    skip_scenes(h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let start = battle(h);
    let players = start
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player);
    assert_eq!(players.count(), 6);
    assert_eq!(start.units().len(), 20);
    playthrough::play_in_game(h, &winning_commands());
    // Harl's last words, the VICTORY banner, the results, the victory
    // scenes.
    playthrough::on_to_the_save_prompt(h);
}

/// Acceptance: Chapter 1 from New Game through the real game: the intro
/// scenes, the battle won with the recorded commands, the results, the
/// victory scenes, the save prompt, and on to Chapter 2.
#[test]
fn chapter_1_plays_from_new_game_into_chapter_2() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_1_won(&mut h);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.gold, 1000, "the clear gold");
    assert_eq!(campaign.roster.len(), 6, "nobody was lost");
    // "Save your progress?" No: Chapter 2's first scene.
    h.keys("Down f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch02"));
    assert_eq!(h.screens(), ["title", "game_flow", "dialogue"]);
}

/// Records a new `fixtures/ch01_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    playthrough::record(
        chapter_1(&content),
        &content,
        803,
        &[
            "// A winning playthrough of Chapter 1 (ticket 0803): the player's commands,",
            "// phase by phase. Played back by ch01_winnable.rs, which says how to record",
            "// a new one. Units 1-6: the lead, Hollis, Tamsin, Aske, Maud, Rue.",
        ],
        "tests/it/fixtures/ch01_winning.ron",
    );
}
