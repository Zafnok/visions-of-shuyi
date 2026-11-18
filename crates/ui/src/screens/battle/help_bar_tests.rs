//! Tests of the bar under the battle map (ticket 0445): the danger-zone
//! hint from where the key help starts, the auto-end hint in the column
//! where the help's end-turn hint starts (the browsing help puts end turn
//! in one column whatever the cursor is on), both hidden while the help
//! has no end turn; a combat's or a preview's line at the left of that
//! row, a note on a strip over the map, a warning in a box. Columns are
//! read from the painted frame, never from the code under test.

use proptest::prelude::*;
use trpg_core::Pos;

use super::layout::{HELP_BAR, HELP_ROW, MAP_VIEW};
use super::tests::{quick, render, step};
use super::view::TogglesView;
use super::*;
use crate::harness::Harness;
use crate::input::{Chord, Device, Key, PadKind};
use crate::screens::battle::testing::wait;
use crate::widgets::help::{SEPARATOR, key_name};

/// Where [`TogglesView::new`] puts the auto-end hint `auto_end` after
/// `left`, end turn in cell `end_turn`, in plain cells.
fn row_of(
    left: impl Into<StyledText>,
    auto_end: impl Into<StyledText>,
    end_turn: usize,
) -> Option<(String, usize)> {
    let widths = shuyi_assets::font::CellWidths::NARROW;
    let t = TogglesView::new(left.into(), auto_end.into(), end_turn, &widths);
    t.auto_end.map(|(text, column)| (text.plain(), column))
}

/// The test context with no key caps: [`render`]'s frame draws a key in
/// its own cells, as the help's columns are then measured. With caps the
/// frame and the columns both count a cap's extra cells (the key-caps
/// integration test checks that frame).
fn ctx() -> Ctx {
    let mut c = crate::screen::tests::ctx();
    c.game.key_caps = false;
    c
}

/// The glyphs of row `y`.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..HELP_BAR.w)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect()
}

/// The column (in cells) where `text` starts on row `y`, if it is there.
fn column_of(buf: &GlyphBuffer, y: i32, text: &str) -> Option<usize> {
    let r = row(buf, y);
    let at = r.find(text)?;
    Some(r[..at].chars().count())
}

/// Checks the bar of `s`'s frame: the danger-zone hint from where the help
/// starts, auto-end starting where end turn starts below it, each hint
/// found by its key's name in `c`'s keymap.
fn assert_lined_up(s: &BattleScreen, c: &Ctx) {
    let name = |action| key_name(c.help_keys(), action);
    let danger = format!("{} danger zone: OFF", name(Action::DangerZone));
    let auto_end = format!("{} auto-end: OFF", name(Action::ToggleAutoEnd));
    let end = format!("{} end turn", name(Action::EndTurn));
    let buf = render(s, c);
    let (bar, help) = (row(&buf, HELP_BAR.y), row(&buf, HELP_ROW));
    let first = help.chars().position(|ch| ch != ' ');
    assert_eq!(column_of(&buf, HELP_BAR.y, &danger), first, "{bar}\n{help}");
    let end_at = column_of(&buf, HELP_ROW, &end);
    assert!(end_at.is_some(), "{help}");
    assert_eq!(
        column_of(&buf, HELP_BAR.y, &auto_end),
        end_at,
        "{bar}\n{help}"
    );
}

#[test]
fn over_a_ready_unit_auto_end_starts_above_end_turn() {
    let c = ctx();
    let s = quick();
    assert_lined_up(&s, &c);
    // The right-handed keys, named here as the player reads them.
    let buf = render(&s, &c);
    assert_eq!(column_of(&buf, HELP_BAR.y, "w danger zone: OFF"), Some(1));
    let end = column_of(&buf, HELP_ROW, "Space end turn");
    assert_eq!(
        column_of(&buf, HELP_BAR.y, "Shift+Space auto-end: OFF"),
        end
    );
}

#[test]
fn the_columns_hold_with_the_left_handed_keys() {
    let mut c = ctx();
    c.use_layout(crate::input::LEFT_HANDED);
    assert_lined_up(&quick(), &c);
}

#[test]
fn the_columns_hold_with_end_turn_on_a_longer_key() {
    let mut c = ctx();
    let mut keys = c.layout_bindings(crate::input::RIGHT_HANDED);
    keys.bind(Action::EndTurn, 0, Chord::shifted(Key::PageDown))
        .unwrap();
    c.set_layout_bindings(crate::input::RIGHT_HANDED, keys)
        .unwrap();
    let end = key_name(c.help_keys(), Action::EndTurn);
    assert!(end.chars().count() > "Space".len(), "{end}");
    assert_lined_up(&quick(), &c);
}

#[test]
fn the_columns_hold_with_a_controller() {
    let mut c = ctx();
    c.device = Device::Pad(PadKind::Xbox);
    assert_lined_up(&quick(), &c);
}

#[test]
fn end_turn_is_in_one_column_whatever_the_cursor_is_on() {
    let c = ctx();
    let mut state = quick_battle(&c.game.content).unwrap();
    // The archer waits where it stands.
    wait(&mut state, 2);
    let mut s = BattleScreen::new(state);
    let mut columns = vec![];
    // A ready unit of ours, one that has acted, an enemy, an empty tile.
    for pos in [
        Pos::new(3, 5),
        Pos::new(2, 4),
        Pos::new(8, 2),
        Pos::new(0, 0),
    ] {
        s.cursor.jump(pos);
        let buf = render(&s, &c);
        columns.push(column_of(&buf, HELP_ROW, "Space end turn"));
        let auto_end = column_of(&buf, HELP_BAR.y, "Shift+Space auto-end");
        assert_eq!(auto_end, columns[columns.len() - 1], "{pos:?}");
    }
    assert!(columns[0].is_some());
    assert!(columns.iter().all(|c| *c == columns[0]), "{columns:?}");
}

#[test]
fn with_no_end_turn_hint_the_toggles_are_hidden() {
    let mut c = ctx();
    let mut s = quick();
    // A unit selected, then its action menu.
    for actions in [&[Action::Confirm][..], &[Action::Confirm]] {
        step(&mut s, &mut c, actions);
        let buf = render(&s, &c);
        assert!(!row(&buf, HELP_ROW).contains("end turn"));
        assert_eq!(row(&buf, HELP_BAR.y).trim(), "", "{:?}", s.mode());
        assert!(s.view(&c).toggles.is_none());
    }
}

#[test]
fn a_preview_takes_the_left_of_the_row_and_nothing_meets_it() {
    let mut c = ctx();
    let mut s = quick();
    // The mage (the next unit) casts Fire at a forest: no enemy in reach.
    step(
        &mut s,
        &mut c,
        &[Action::NextUnit, Action::Confirm, Action::Confirm],
    );
    step(
        &mut s,
        &mut c,
        &[Action::CursorUp, Action::CursorUp, Action::Confirm],
    );
    step(&mut s, &mut c, &[Action::Confirm]);
    let view = s.view(&c);
    let said = view.message.as_ref().expect("a preview");
    assert_eq!(said.kind, MessageKind::Said);
    assert!(view.toggles.is_none());
    let buf = render(&s, &c);
    assert_eq!(row(&buf, HELP_BAR.y).trim_end(), format!(" {}", said.text));
}

/// The battle screen of `h` painted.
fn frame(h: &Harness) -> GlyphBuffer {
    render(h.battle().unwrap(), h.game().ctx())
}

#[test]
fn flipping_the_toggles_reads_on_the_row_and_the_note_shows_over_the_map() {
    let mut h = Harness::with_screen(Box::new(quick()));
    h.keys("w Shift+Space");
    let buf = frame(&h);
    let bar = row(&buf, HELP_BAR.y);
    assert!(bar.starts_with(" w danger zone: ON "), "{bar}");
    assert!(
        bar.trim_end().ends_with("Shift+Space auto-end: ON"),
        "{bar}"
    );
    let above: String = row(&buf, HELP_BAR.y - 1)
        .chars()
        .take(usize::try_from(MAP_VIEW.w).unwrap())
        .collect();
    assert_eq!(above.trim_end(), " Auto-end: ON");
    // It goes after a moment, the map back under it.
    h.wait(5.0);
    assert!(!row(&frame(&h), HELP_BAR.y - 1).contains("Auto-end"));
}

#[test]
fn a_failed_save_is_a_warning() {
    let c = ctx();
    let mut s = quick();
    let text = "Save storage error: localStorage is turned off or blocked in this browser";
    s.suspend_failed(text.to_owned());
    let view = s.view(&c);
    assert_eq!(view.message, Some(MessageView::warning(text.to_owned())));
    // The toggles stay: the warning is over the map, not on their row.
    let buf = render(&s, &c);
    let bar = row(&buf, HELP_BAR.y);
    assert!(bar.starts_with(" w danger zone: OFF "), "{bar}");
    assert!(!bar.contains("Save"), "{bar}");
}

#[test]
fn auto_end_falls_back_one_separator_after_danger_zone() {
    let sep = SEPARATOR.chars().count();
    // End turn's column would overlap the danger-zone hint.
    let t = row_of("w danger zone: OFF".to_owned(), "a".to_owned(), 10);
    assert_eq!(t, Some(("a".to_owned(), 18 + sep)));
    // ... or run off the row.
    let t = row_of("d".to_owned(), "auto-end".to_owned(), TogglesView::ROOM - 3);
    assert_eq!(t, Some(("auto-end".to_owned(), 1 + sep)));
    // In its column when it fits.
    let t = row_of("d".to_owned(), "auto-end".to_owned(), 40);
    assert_eq!(t, Some(("auto-end".to_owned(), 40)));
    // The row holds 98 characters (columns 1 to 98 of 100): ending on the
    // 98th fits, one more falls back.
    assert_eq!(TogglesView::ROOM, 98);
    let t = row_of("d".to_owned(), "auto-end".to_owned(), 90);
    assert_eq!(t, Some(("auto-end".to_owned(), 90)));
    let t = row_of("d".to_owned(), "auto-end!".to_owned(), 90);
    assert_eq!(t, Some(("auto-end!".to_owned(), 1 + sep)));
    // Left out when it fits nowhere.
    let long = "x".repeat(TogglesView::ROOM);
    let t = row_of(long.clone(), "auto-end".to_owned(), 40);
    assert_eq!(t, None);
}

proptest! {
    /// For any key names up to well past the longest a keymap gives
    /// (`Shift+Ctrl+PageDown`, a controller's `Right stick`), the two
    /// hints never overlap and stay inside the row.
    #[test]
    fn the_toggles_never_overlap_and_stay_in_the_row(
        danger in 1usize..40,
        auto_end in 1usize..40,
        end_turn in 0usize..100,
    ) {
        let t = row_of("d".repeat(danger), "a".repeat(auto_end), end_turn);
        if let Some((text, column)) = &t {
            let sep = SEPARATOR.chars().count();
            prop_assert!(*column >= danger + sep);
            prop_assert!(column + text.chars().count() <= TogglesView::ROOM);
        }
    }
}
