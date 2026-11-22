use trpg_core::{Command, UnitAction, UnitId};

use super::*;

fn wait(unit: u32) -> Command {
    Command::Act {
        unit: UnitId(unit),
        dest: trpg_core::Pos::new(1, 2),
        action: UnitAction::Wait,
    }
}

fn recorded(phases: Vec<Vec<Command>>) -> Recorded {
    Recorded {
        phases,
        summary: "Some(Victory) on turn 2; lost: []".to_owned(),
    }
}

#[test]
fn tries_one_to_eight_field_lines_one_to_eight_then_line_one_again() {
    let lines: Vec<usize> = (1..=9).map(|seed| line_of(seed, 8)).collect();
    assert_eq!(lines, [0, 1, 2, 3, 4, 5, 6, 7, 0]);
    assert_eq!(
        line_of(17, 8),
        line_of(1, 8),
        "--seed 17 plays try 17's army"
    );
    assert_eq!(line_of(5, 1), 0, "one army");
    assert_eq!(line_of(5, 0), 0, "none counts as one");
    assert_eq!(line_of(0, 8), 7, "seed 0 comes before seed 1");
}

/// A file reads back as the lines it was written from, each line's
/// commands in order, and says which seed and how each line ended.
#[test]
fn a_file_reads_back_as_its_lines() {
    let first = recorded(vec![
        vec![wait(1), Command::EndPhase],
        vec![Command::EndPhase],
    ]);
    let second = recorded(vec![vec![wait(2), wait(1), Command::EndPhase]]);
    let text = file_text("ch01", &[(1, first.clone()), (2, second.clone())]).unwrap();
    let lines = parse("ch01", &text).unwrap();
    assert_eq!(lines, [first.commands(), second.commands()]);
    assert!(text.contains("// Line 2 (seed 2): Some(Victory) on turn 2; lost: []"));
    assert!(text.contains("never edit it by hand"));
}

#[test]
fn a_missing_battle_or_line_says_how_to_record_it() {
    let mut wins = Wins::default();
    let none = wins.line("ch01", 0).unwrap_err();
    assert!(none.contains("`cargo xtask bot-wins ch01`"), "{none}");
    wins.set("ch01", 1, vec![wait(1)]);
    assert_eq!(wins.lines("ch01"), 2);
    assert_eq!(
        wins.line("ch01", 0).unwrap(),
        &[] as &[Command],
        "left empty"
    );
    assert_eq!(wins.line("ch01", 1).unwrap(), &[wait(1)]);
    let short = wins.line("ch01", 5).unwrap_err();
    assert!(short.contains("has 2 recorded lines, not 6"), "{short}");
}

/// Every file in `crates/bots/wins/` reads, with a line each of the bots'
/// [`LINES`].
#[test]
fn every_embedded_file_reads_with_every_line() {
    let wins = Wins::embedded().unwrap();
    for (battle, _) in FILES {
        assert_eq!(wins.lines(battle), LINES, "{battle}");
        for line in 0..LINES {
            assert_ne!(wins.line(battle, line).unwrap().len(), 0, "{battle}");
        }
    }
}
