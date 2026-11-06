//! Chapter 3 can be won (ticket 0849): the player's commands in
//! `fixtures/ch03_winning.ron`, played with the army Chapters 1 and 2's
//! recorded playthroughs leave (`ch01_winnable`, `ch02_winnable`, Joss
//! joined) and Preparations left as they are, seize Greywater's
//! scriptorium with nobody lost, the prior beaten on his seat. How the
//! fixture is played and recorded: [`playthrough`].
//!
//! **If a balance change breaks this test** (or a new Chapter 1 or 2
//! playthrough), the change must update the playthrough on purpose: look
//! at what it did to the chapter, then record a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch03_winnable::record -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns). A new
//! Chapter 3 playthrough leaves Chapter 4 another army: record Chapter
//! 4's again after it (`ch04_winnable`).

use trpg_content::Content;
use trpg_content::chapter::roster_units;
use trpg_core::{BattleState, Campaign, CharacterId, Outcome};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::ch02_winnable::{chapter_2, chapter_2_won, winning_commands as chapter_2_commands};
use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch03_winning.ron");

/// The player's commands, in order.
pub fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch03_winning.ron", FIXTURE)
}

/// The campaign as Chapters 1 and 2's recorded playthroughs leave it
/// (every rewind charge unused, as the game counts them; Joss joined in
/// the scene after the ford), and Chapter 3's battle as it starts with
/// it, Preparations left as they are.
pub fn chapter_3(content: &Content) -> (Campaign, BattleState) {
    let tables = content.tables();
    let (mut campaign, mut state) = chapter_2(content);
    playthrough::play(&mut state, content, &chapter_2_commands());
    let unused = state.rewind_charges();
    campaign
        .apply_result(&content.battles["ch02"], &state, unused)
        .unwrap_or_else(|e| panic!("{e:?}"));
    for joss in roster_units(content, &[CharacterId("defector".into())]) {
        campaign.joins_in_scene(joss);
    }
    let def = &content.battles["ch03"];
    let state = BattleState::new(campaign.battle_setup(def, &tables)).0;
    (campaign, state)
}

/// The rules alone: the fixture's commands seize the scriptorium, in
/// Classic, losing nobody; the prior fell first, since he sits on it.
#[test]
fn the_recorded_playthrough_seizes_the_scriptorium() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = chapter_3(&content);
    assert_eq!(state.units().len(), 7 + 14, "seven and the abbey");
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    let prior = CharacterId("greywater_prior".into());
    let fell = state.fallen().iter();
    assert!(
        fell.into_iter()
            .any(|u| u.character.as_ref() == Some(&prior))
    );
}

/// New Game, then Chapters 1 to 3 won with the recorded commands through
/// the real game (their scenes skipped, Preparations left as they are),
/// up to Chapter 3's "Save your progress?".
pub fn chapter_3_won(h: &mut Harness) {
    chapter_2_won(h);
    // "Save your progress?" No; the boathouse and the causeway.
    h.keys("Down f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch03"));
    skip_scenes(h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "preparations"],
        "{}",
        h.snapshot()
    );
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(h).units().len(), 7 + 14, "seven and the abbey");
    playthrough::play_in_game(h, &winning_commands());
    playthrough::on_to_the_save_prompt(h);
}

/// Acceptance: New Game reaches Chapter 3 through Chapters 1 and 2, and
/// Chapter 3 plays through the real game: the boathouse and the
/// causeway, Preparations, the battle won with the recorded commands, the
/// results, the scriptorium, the causeway at night, the kitchen, Veyne
/// Hall, the save prompt, and on to Chapter 4 (0850).
#[test]
fn chapter_3_plays_from_new_game_into_chapter_4() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_3_won(&mut h);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.roster.len(), 7, "nobody was lost");
    h.keys("Down f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch04"));
}

/// Records a new `fixtures/ch03_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, start) = chapter_3(&content);
    playthrough::record(
        start,
        &content,
        849,
        &[
            "// A winning playthrough of Chapter 3 (ticket 0849): the player's commands,",
            "// phase by phase, with the army Chapters 1 and 2's playthroughs leave. Played",
            "// back by ch03_winnable.rs, which says how to record a new one. Units 1-7: the",
            "// lead, Hollis, Tamsin, Aske, Maud, Rue, Joss.",
        ],
        "tests/it/fixtures/ch03_winning.ron",
    );
}
