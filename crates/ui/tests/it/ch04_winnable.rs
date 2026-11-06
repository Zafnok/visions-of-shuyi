//! Chapter 4 can be won (ticket 0850): the player's commands in
//! `fixtures/ch04_winning.ron`, played with the army Chapters 1 to 3's
//! recorded playthroughs leave (`ch01_winnable` to `ch03_winnable`) and
//! Preparations left as they are, beat Dace at Veyne Hall: he retreats at
//! 0 HP, which wins, and nobody is lost. How the fixture is played and
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
//! and read the new fixture's last lines (who fell, how many turns).

use trpg_content::Content;
use trpg_core::{BattleState, Campaign, CharacterId, Defeat, Outcome};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::ch03_winnable::{chapter_3, chapter_3_won, winning_commands as chapter_3_commands};
use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch04_winning.ron");

fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch04_winning.ron", FIXTURE)
}

/// The campaign as Chapters 1 to 3's recorded playthroughs leave it
/// (every rewind charge unused, as the game counts them), and Chapter 4's
/// battle as it starts with it, Preparations left as they are. Gil isn't
/// in it: he joins only on Millhaven Road (1071).
fn chapter_4(content: &Content) -> (Campaign, BattleState) {
    let tables = content.tables();
    let (mut campaign, mut state) = chapter_3(content);
    playthrough::play(&mut state, content, &chapter_3_commands());
    let unused = state.rewind_charges();
    campaign
        .apply_result(&content.battles["ch03"], &state, unused)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let def = &content.battles["ch04"];
    let state = BattleState::new(campaign.battle_setup(def, &tables)).0;
    (campaign, state)
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

/// Acceptance: New Game reaches Chapter 4 through Chapters 1 to 3, and
/// Chapter 4 plays through the real game: the valley road, the Act 1
/// world map, the march to Veyne Hall, the rise and the gate,
/// Preparations, the battle won with the recorded commands, the results,
/// the stable, the Hound post, "To be continued".
#[test]
fn chapter_4_plays_from_new_game_to_to_be_continued() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_3_won(&mut h);
    // "Save your progress?" No; the valley road, then the map.
    h.keys("Down f");
    assert_eq!(h.top_screen(), "dialogue", "{}", h.snapshot());
    skip_scenes(&mut h);
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "world_map"],
        "{}",
        h.snapshot()
    );
    // Find → Story battles: Veyne Hall; travel there. Arriving plays the
    // rise and the gate, then Preparations; Left wraps to `Fight!`.
    h.keys("x Down Down f f");
    h.wait(3.0);
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
    assert_eq!(battle(&h).units().len(), 7 + 13, "seven and the hall");
    playthrough::play_in_game(&mut h, &winning_commands());
    for _ in 0..150 {
        match h.top_screen() {
            "dialogue" => skip_scenes(&mut h),
            "to_be_continued" => break,
            // Dace runs and his Hounds after him; banners and the results
            // (a page for each level gained) take a press.
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "to_be_continued"],
        "{}",
        h.snapshot()
    );
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.roster.len(), 7, "nobody was lost");
    let world = campaign.world.as_ref();
    let cleared = world.is_some_and(|w| w.cleared.contains("veyne_hall"));
    assert!(cleared, "Veyne Hall is won");
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
