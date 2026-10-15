//! Battle notes (ticket 0411, `docs/design/magic.md`): the battle's
//! strategy hints, shown in a `BATTLE NOTES` box at the start of the battle
//! and again under the objective on the map menu's `Objective` page. While
//! either is up, the units the notes are about blink on the map, and the
//! box keeps off their rows.

use crate::prelude::*;
use trpg_core::{BattleNote, UnitId};

use crate::widgets::wrap::word_wrap;
use crate::words::Words;

/// The key of the notes box's title (`BATTLE NOTES`).
pub const NOTES_TITLE: &str = "battle.notes.title";

/// The key of the heading over the notes on the `Objective` page.
pub const NOTES_HEADING: &str = "battle.notes.heading";

/// The widest a line of a note is, in cells, its marker included.
pub const NOTE_WIDTH: usize = 48;

/// Starts each note; the lines a long note wraps onto are indented by as
/// much.
pub const MARKER: &str = "• ";

/// How long a noted unit stays highlighted, then plain, in seconds.
/// *Tunable.*
pub const BLINK_S: f32 = 0.4;

/// The lines of `notes`: each starts with [`MARKER`] and wraps to
/// [`NOTE_WIDTH`], its further lines indented under its text.
pub fn note_lines(notes: &[BattleNote], words: Words<'_>) -> Vec<String> {
    let indent = MARKER.chars().count();
    notes
        .iter()
        .flat_map(|note| {
            let lines = word_wrap(words.battle_note(note), NOTE_WIDTH - indent);
            lines.into_iter().enumerate().map(move |(i, line)| {
                if i == 0 {
                    format!("{MARKER}{line}")
                } else {
                    format!("{}{line}", " ".repeat(indent))
                }
            })
        })
        .collect()
}

/// Whether a note of `notes` is about `unit`.
pub fn is_noted(notes: &[BattleNote], unit: UnitId) -> bool {
    notes.iter().any(|n| n.units.contains(&unit))
}

/// Whether noted units are highlighted `t` seconds after the notes came
/// up: on for the first [`BLINK_S`], off for the next, and so on.
pub fn blink_on(t: f32) -> bool {
    !t.is_finite() || t.rem_euclid(2.0 * BLINK_S) < BLINK_S
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(text: &str, units: &[u32]) -> BattleNote {
        BattleNote {
            text: text.into(),
            units: units.iter().map(|&u| UnitId(u)).collect(),
        }
    }

    #[test]
    fn notes_wrap_under_their_marker() {
        let notes = [
            note("Frost Elemental: weak to Fire, absorbs Ice.", &[4]),
            note(
                "Burn the forest to cut off the ambush before the riders reach the bridge.",
                &[],
            ),
        ];
        assert_eq!(
            note_lines(&notes, Words::ENGLISH),
            [
                "• Frost Elemental: weak to Fire, absorbs Ice.",
                "• Burn the forest to cut off the ambush before",
                "  the riders reach the bridge.",
            ]
        );
        assert_eq!(note_lines(&[], Words::ENGLISH), Vec::<String>::new());
        // A line as wide as it may be isn't wrapped.
        let full = "x".repeat(NOTE_WIDTH - 2);
        assert_eq!(
            note_lines(&[note(&full, &[])], Words::ENGLISH),
            [format!("• {full}")]
        );
        let over = format!("{full} y");
        assert_eq!(note_lines(&[note(&over, &[])], Words::ENGLISH).len(), 2);
    }

    #[test]
    fn noted_units_are_those_a_note_names() {
        let notes = [note("A.", &[4, 6]), note("B.", &[]), note("C.", &[2])];
        let about: Vec<u32> = (1..=7).filter(|&u| is_noted(&notes, UnitId(u))).collect();
        assert_eq!(about, [2, 4, 6]);
        assert!(!is_noted(&[], UnitId(1)));
    }

    #[test]
    fn the_blink_starts_on_and_alternates() {
        assert!(blink_on(0.0));
        assert!(blink_on(BLINK_S * 0.99));
        assert!(!blink_on(BLINK_S));
        assert!(!blink_on(BLINK_S * 1.99));
        assert!(blink_on(BLINK_S * 2.0));
        assert!(!blink_on(BLINK_S * 3.5));
        assert!(blink_on(f32::NAN));
    }

    /// Every battle's notes fit the map view, on the `Objective` page too
    /// (two lines, a blank and the heading over them, in a box with a
    /// blank row inside its border).
    #[test]
    fn every_battles_notes_fit_the_map_view() {
        use crate::screens::battle::layout::MAP_VIEW;
        let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
        for (id, battle) in &content.battles {
            let lines = note_lines(&battle.battle_notes, Words::ENGLISH);
            let rows = i32::try_from(lines.len()).unwrap_or(i32::MAX);
            assert!(rows + 4 + 4 <= MAP_VIEW.h, "{id}: {rows} lines");
            assert!(
                lines.iter().all(|l| l.chars().count() <= NOTE_WIDTH),
                "{id}"
            );
        }
    }
}
