//! Chapter 2 can be won (ticket 0848): the player's commands in
//! `fixtures/ch02_winning.ron`, played with the army Chapter 1's recorded
//! playthrough leaves (`ch01_winnable`) and Preparations left as they are,
//! beat Silas Hendry with nobody lost and at least the minimum of Kell's
//! Ford's houses standing; Joss joins in the scene after the battle, and
//! Hendry's axe is the player's. How the fixture is played and recorded:
//! [`playthrough`].
//!
//! **If a balance change breaks this test** (or a new Chapter 1
//! playthrough), the change must update the playthrough on purpose: look
//! at what it did to the chapter, then record a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch02_winnable::record -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns, how
//! many houses).

use trpg_content::{Content, new_campaign};
use trpg_core::{BattleState, Campaign, CharacterId, GameMode, ItemId, Outcome};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

use crate::ch01_winnable::{chapter_1_won, lead, winning_commands as chapter_1_commands};
use crate::playthrough::{self, battle, lost, skip_scenes};

const FIXTURE: &str = include_str!("fixtures/ch02_winning.ron");

fn winning_commands() -> Vec<trpg_core::Command> {
    playthrough::commands("ch02_winning.ron", FIXTURE)
}

/// The campaign as Chapter 1's recorded playthrough leaves it (every
/// rewind charge unused, as the game counts them), and Chapter 2's battle
/// as it starts with it, Preparations left as they are.
fn chapter_2(content: &Content) -> (Campaign, BattleState) {
    let tables = content.tables();
    let mut campaign = new_campaign(content, GameMode::Classic, lead());
    let first = &content.battles["ch01"];
    let mut state = BattleState::new(campaign.battle_setup(first, &tables)).0;
    playthrough::play(&mut state, content, &chapter_1_commands());
    let unused = state.rewind_charges();
    campaign
        .apply_result(first, &state, unused)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let def = &content.battles["ch02"];
    let state = BattleState::new(campaign.battle_setup(def, &tables)).0;
    (campaign, state)
}

/// The rules alone: the fixture's commands beat Hendry, in Classic,
/// losing nobody and saving at least three houses; the win gives his axe.
#[test]
fn the_recorded_playthrough_beats_the_lieutenant() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (mut campaign, mut state) = chapter_2(&content);
    playthrough::play(&mut state, &content, &winning_commands());
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    let village = state.village().unwrap_or_else(|| panic!("no village"));
    assert!(village.saved() >= 3, "{} saved", village.saved());
    campaign
        .apply_result(&content.battles["ch02"], &state, 0)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let axe = ItemId::new("houndmasters_axe");
    assert!(campaign.stock.weapons.iter().any(|w| w.def == axe));
}

/// Acceptance: New Game reaches Chapter 2 through Chapter 1, and Chapter
/// 2 plays through the real game: its intro scenes, Preparations, the
/// battle won with the recorded commands, the results, the herald, Joss
/// joining on the far bank, Veyne Hall, the save prompt, "To be
/// continued".
#[test]
fn chapter_2_plays_from_new_game_to_to_be_continued() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    chapter_1_won(&mut h);
    // "Save your progress?" No; Chapter 2's halt and the ford's edge.
    h.keys("Down f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch02"));
    skip_scenes(&mut h);
    // Preparations for the first time; Left wraps to `Fight!`.
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "preparations"],
        "{}",
        h.snapshot()
    );
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(&h).units().len(), 6 + 14, "six and the raid");
    playthrough::play_in_game(&mut h, &winning_commands());
    playthrough::on_to_the_save_prompt(&mut h);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    let joss = CharacterId("defector".into());
    let joined = campaign.member(&joss);
    assert!(joined.is_some(), "Joss joined on the far bank");
    assert_eq!(campaign.roster.len(), 7, "nobody was lost, and Joss");
    let axe = ItemId::new("houndmasters_axe");
    assert!(campaign.stock.weapons.iter().any(|w| w.def == axe));
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    h.keys("f");
    assert_eq!(h.screens(), ["title"]);
}

/// Records a new `fixtures/ch02_winning.ron` ([`playthrough::record`]).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let (_, start) = chapter_2(&content);
    playthrough::record(
        start,
        &content,
        848,
        &[
            "// A winning playthrough of Chapter 2 (ticket 0848): the player's commands,",
            "// phase by phase, with the army Chapter 1's playthrough leaves. Played back by",
            "// ch02_winnable.rs, which says how to record a new one. Units 1-6: the lead,",
            "// Hollis, Tamsin, Aske, Maud, Rue.",
        ],
        "tests/it/fixtures/ch02_winning.ron",
    );
}
