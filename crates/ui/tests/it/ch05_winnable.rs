//! Chapter 5 can be won (ticket 0851): the player's commands in
//! `fixtures/ch05_winning.ron`, played with the army Chapters 1 to 4's
//! recorded playthroughs leave (`ch01_winnable` to `ch04_winnable`),
//! Hedda joined in the attack scene, and Preparations left as they are,
//! hold the envoy's house until first light with nobody lost. How the
//! fixture is played and recorded: [`playthrough`].
//!
//! **If a balance change breaks this test** (or a new Chapter 1 to 4
//! playthrough), the change must update the playthrough on purpose: look
//! at what it did to the chapter, then record a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch05_winnable::record -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns).

use trpg_content::Content;
use trpg_content::chapter::roster_units;
use trpg_core::{BattleState, Campaign, CharacterId, Outcome, SavePoint};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::ch04_winnable::{chapter_4, chapter_4_won, winning_commands as chapter_4_commands};
use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch05_winning.ron");

fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch05_winning.ron", FIXTURE)
}

/// The campaign as Chapters 1 to 4's recorded playthroughs leave it
/// (every rewind charge unused, as the game counts them), Hedda joined as
/// the house is attacked (`ch05_prebattle`), and Chapter 5's battle as it
/// starts with it, Preparations left as they are.
fn chapter_5(content: &Content) -> (Campaign, BattleState) {
    let tables = content.tables();
    let (mut campaign, mut state) = chapter_4(content);
    playthrough::play(&mut state, content, &chapter_4_commands());
    let unused = state.rewind_charges();
    campaign
        .apply_result(&content.battles["ch04"], &state, unused)
        .unwrap_or_else(|e| panic!("{e:?}"));
    for hedda in roster_units(content, &[CharacterId("shieldbearer".into())]) {
        campaign.joins_in_scene(hedda);
    }
    let def = &content.battles["ch05"];
    let state = BattleState::new(campaign.battle_setup(def, &tables)).0;
    (campaign, state)
}

/// The rules alone: the fixture's commands hold the house until the last
/// phase of turn 8 ends, in Classic, losing nobody.
#[test]
fn the_recorded_playthrough_holds_the_house_till_light() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, mut state) = chapter_5(&content);
    assert_eq!(state.units().len(), 8 + 9, "eight and the Hounds");
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    assert!(state.turn() >= 8, "won as turn 8 ends, not before");
}

/// Acceptance: New Game reaches Chapter 5 through Chapters 1 to 4, and
/// Chapter 5 plays through the real game: the road south, the Act 1
/// world map, the road to Saltmere and the Envoy's House, the house, the
/// sea steps and the attack (where Hedda joins), Preparations, the hold
/// won with the recorded commands, the results, the morning, the quay,
/// the boat, the Hound post, "Save your progress?" (0869: the map won't
/// come back), a world-map save, "To be continued".
#[test]
fn chapter_5_plays_from_new_game_to_to_be_continued() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_4_won(&mut h);
    // Find, Story battles, the next one from Veyne Hall where the army
    // is: the Envoy's House; close the list and travel there. Arriving
    // plays the house, the sea steps and the attack, then Preparations;
    // Left wraps to `Fight!`.
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
    assert_eq!(battle(&h).units().len(), 8 + 9, "eight and the Hounds");
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
    // Yes, into the first empty slot: a save on the map, the house held.
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
    let hedda = CharacterId("shieldbearer".into());
    let joined = campaign
        .roster
        .iter()
        .any(|u| u.character.as_ref() == Some(&hedda));
    assert!(joined, "Hedda joined");
    assert_eq!(campaign.roster.len(), 8, "nobody was lost");
    let world = campaign.world.as_ref();
    let cleared = world.is_some_and(|w| w.cleared.contains("envoys_house"));
    assert!(cleared, "the Envoy's House is held");
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
            "// Hedda. Played back by ch05_winnable.rs, which says how to record a new one.",
            "// Units 1-8: the lead, Hedda, Hollis, Tamsin, Aske, Maud, Rue, Joss.",
        ],
        "tests/it/fixtures/ch05_winning.ron",
    );
}
