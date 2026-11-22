use trpg_core::{GameMode, LeadGender, LeadProfile};

use super::*;
use crate::testkit::test_story;
use crate::{Wins, start_of};

/// The test story's first battle as its first line starts it: a Seize
/// within 3 turns.
fn first_battle(content: &Content) -> BattleState {
    let def = &content.battles["test_world_story"];
    let lead = LeadProfile::new("Lead", LeadGender::Male);
    let wins = Wins::default();
    let setup = start_of(content, def, GameMode::Classic, lead, &wins, 0);
    BattleState::new(setup.unwrap().unwrap()).0
}

/// A recording wins with nobody lost, and its commands win again when
/// played back as the game plays the other phases.
#[test]
fn a_recording_wins_and_its_commands_win_again() {
    let content = test_story();
    let recorded = record(first_battle(&content), &content, 1, &|_| true).unwrap();
    assert!(
        recorded.summary.starts_with("Some(Victory) on turn "),
        "{}",
        recorded.summary
    );
    assert!(recorded.summary.contains("lost: []"));
    assert!(
        recorded.phases.iter().all(|p| !p.is_empty()),
        "a phase a turn"
    );
    let mut again = first_battle(&content);
    play(&mut again, &content, &recorded.commands()).unwrap();
    assert_eq!(again.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&again), Vec::<&str>::new());
}

/// The same battle and seed give the same recording.
#[test]
fn a_recording_is_the_same_every_time() {
    let content = test_story();
    let once = record(first_battle(&content), &content, 7, &|_| true).unwrap();
    let twice = record(first_battle(&content), &content, 7, &|_| true).unwrap();
    assert_eq!(once, twice);
}

/// A win that isn't wanted counts for nothing: with none wanted, there is
/// no recording, and the error says why.
#[test]
fn a_win_nobody_wants_is_no_recording() {
    let content = test_story();
    let error = record(first_battle(&content), &content, 1, &|_| false).unwrap_err();
    assert_ne!(error, "");
}

/// Replaying commands that don't fit the battle says which command.
#[test]
fn a_command_the_rules_refuse_is_named() {
    let content = test_story();
    let mut state = first_battle(&content);
    let bogus = Command::Act {
        unit: trpg_core::UnitId(999),
        dest: Pos::new(0, 0),
        action: UnitAction::Wait,
    };
    let error = play(&mut state, &content, &[bogus]).unwrap_err();
    assert!(error.starts_with("command 0, "), "{error}");
}
