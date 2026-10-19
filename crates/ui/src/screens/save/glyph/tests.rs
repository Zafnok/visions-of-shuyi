use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::screen::tests::ctx;
use crate::widgets::{MenuItemView, MenuView};
use trpg_core::GameMode;

/// A buffer full of a colour no screen uses, to see what is left unpainted.
fn stale(c: &Ctx) -> GlyphBuffer {
    let enemy = c.palette.get(UiColor::Enemy);
    GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new('x', enemy, enemy))
}

fn row(buf: &GlyphBuffer, y: i32) -> String {
    let cells = (0..i32::from(buf.width())).map(|x| buf.get(x, y).map_or(' ', |c| c.glyph));
    cells.collect::<String>().trim_end().to_owned()
}

fn shows(buf: &GlyphBuffer, text: &str) -> bool {
    (0..i32::from(buf.height())).any(|y| row(buf, y).contains(text))
}

fn assert_all_painted(buf: &GlyphBuffer) {
    let left = (0..i32::from(ConsoleSpec::PC.rows))
        .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
        .filter(|&(x, y)| buf.get(x, y).is_some_and(|c| c.glyph == 'x'))
        .count();
    assert_eq!(left, 0, "{left} stale cells");
}

fn item(label: &str, focused: bool) -> MenuItemView {
    MenuItemView {
        label: label.to_owned(),
        suffix: None,
        note: None,
        enabled: true,
        focused,
    }
}

fn saved(n: usize, focused: bool) -> SlotView {
    SlotView {
        number: n,
        focused,
        entry: SlotEntryView::Saved {
            chapter: "Chapter One".to_owned(),
            mode: GameMode::Classic,
            mode_name: "Classic".to_owned(),
            army: "3 units".to_owned(),
            playtime: "1:02:05".to_owned(),
        },
    }
}

fn view() -> SlotPickerView {
    SlotPickerView {
        title: "Load Game".to_owned(),
        columns: SlotColumnsView {
            slot: "Slot".to_owned(),
            chapter: "Chapter".to_owned(),
            mode: "Mode".to_owned(),
            army: "Army".to_owned(),
            time: "Time".to_owned(),
        },
        slots: vec![
            SlotView {
                number: 1,
                focused: false,
                entry: SlotEntryView::Empty("Empty".to_owned()),
            },
            saved(2, true),
            SlotView {
                number: 3,
                focused: false,
                entry: SlotEntryView::Unreadable("Broken".to_owned()),
            },
        ],
        range: "1-3 of 3".to_owned(),
        message: None,
        question: None,
        help: "help".to_owned(),
    }
}

#[test]
fn the_prompt_is_a_question_a_menu_and_a_help_line() {
    let c = ctx();
    let view = SavePromptView {
        question: "SAVE?".to_owned(),
        menu: MenuView {
            items: vec![item("Yes", true), item("No", false)],
            first_shown: 0,
            shown: 2,
        },
        help: "help".to_owned(),
    };
    let mut buf = stale(&c);
    paint_prompt(&c, &view, &mut buf);
    assert_all_painted(&buf);
    assert_eq!(row(&buf, QUESTION_ROW).trim(), "SAVE?");
    assert_eq!(row(&buf, MENU_ROW + 1).trim(), "│ Yes │");
    assert_eq!(row(&buf, MENU_ROW + 2).trim(), "│ No  │");
    assert_eq!(
        row(&buf, i32::from(ConsoleSpec::PC.rows) - 1).trim(),
        "help"
    );
}

#[test]
fn the_panel_has_the_title_range_headings_and_a_row_per_slot() {
    let c = ctx();
    let mut buf = stale(&c);
    paint_slots(&c, &view(), &mut buf);
    assert_all_painted(&buf);
    // The title and the range sit in the borders.
    assert!(row(&buf, PANEL.y).contains(" Load Game "));
    assert!(row(&buf, PANEL.y + PANEL.h - 1).contains(" 1-3 of 3 "));
    let headings = row(&buf, HEADING_ROW);
    for name in ["Slot", "Chapter", "Mode", "Army", "Time"] {
        assert!(headings.contains(name), "{name} not in {headings:?}");
    }
    // Slots go from the first row, one to a row.
    assert!(row(&buf, FIRST_ROW).contains("01"));
    assert!(row(&buf, FIRST_ROW).contains("Empty"));
    let second = row(&buf, FIRST_ROW + 1);
    for part in ["02", "Chapter One", "Classic", "3 units", "1:02:05"] {
        assert!(second.contains(part), "{part:?} not in {second:?}");
    }
    assert!(row(&buf, FIRST_ROW + 2).contains("Broken"));
    assert_eq!(
        row(&buf, i32::from(ConsoleSpec::PC.rows) - 1).trim(),
        "help"
    );
}

/// The columns line up with their headings: the playtimes end where the
/// time heading does.
#[test]
fn the_playtime_is_right_aligned_under_its_heading() {
    let c = ctx();
    let mut buf = stale(&c);
    paint_slots(&c, &view(), &mut buf);
    let end = |text: &str, y: i32| {
        let line = row(&buf, y);
        let at = line[..line.find(text).unwrap()].chars().count() + text.chars().count();
        i32::try_from(at).unwrap()
    };
    assert_eq!(end("1:02:05", FIRST_ROW + 1), TIME_END_X);
    assert_eq!(end("Time", HEADING_ROW), TIME_END_X);
}

#[test]
fn the_focused_slot_is_a_bar_and_an_unreadable_one_is_in_the_warning_colour() {
    let c = ctx();
    let mut buf = stale(&c);
    paint_slots(&c, &view(), &mut buf);
    let bar = c.palette.get(UiColor::PanelBorderFocus);
    let at = |x, y| *buf.get(x, y).unwrap();
    // The bar fills the row inside the panel, with the panel's colour for
    // text.
    assert_eq!(at(PANEL.x + 1, FIRST_ROW + 1).bg, bar);
    assert_eq!(at(PANEL.x + PANEL.w - 2, FIRST_ROW + 1).bg, bar);
    assert_eq!(
        at(SLOT_X, FIRST_ROW + 1).fg,
        c.palette.get(UiColor::PanelBg)
    );
    assert_ne!(at(SLOT_X, FIRST_ROW).bg, bar);
    // An empty slot is dim, an unreadable one is in the warning colour.
    assert_eq!(at(CHAPTER_X, FIRST_ROW).fg, c.palette.get(UiColor::TextDim));
    assert_eq!(
        at(CHAPTER_X, FIRST_ROW + 2).fg,
        c.palette.get(UiColor::HpLow)
    );
    // On the bar it keeps the bar's text colour.
    let mut focused = view();
    focused.slots[2].focused = true;
    focused.slots[1].focused = false;
    let mut buf = stale(&c);
    paint_slots(&c, &focused, &mut buf);
    assert_eq!(
        buf.get(CHAPTER_X, FIRST_ROW + 2).unwrap().fg,
        c.palette.get(UiColor::PanelBg)
    );
}

#[test]
fn a_long_chapter_title_is_cut_to_its_column() {
    let c = ctx();
    let mut long = view();
    long.slots[1] = saved(2, true);
    let SlotEntryView::Saved { chapter, .. } = &mut long.slots[1].entry else {
        unreachable!();
    };
    *chapter = "x".repeat(CHAPTER_W + 10);
    let mut buf = stale(&c);
    paint_slots(&c, &long, &mut buf);
    let line = row(&buf, FIRST_ROW + 1);
    assert_eq!(line.matches('x').count(), CHAPTER_W);
    assert!(line.contains("Classic"));
}

#[test]
fn a_message_goes_under_the_panel() {
    let c = ctx();
    let mut with = view();
    with.message = Some("Could not save".to_owned());
    let mut buf = stale(&c);
    paint_slots(&c, &with, &mut buf);
    assert_eq!(row(&buf, MESSAGE_ROW).trim(), "Could not save");
    assert_eq!(MESSAGE_ROW, 28);
    assert!(!shows(
        &{
            let mut buf = stale(&c);
            paint_slots(&c, &view(), &mut buf);
            buf
        },
        "Could not save"
    ));
}

#[test]
fn the_overwrite_question_is_a_double_box_in_the_middle() {
    let c = ctx();
    let mut asking = view();
    asking.question = Some(OverwriteView {
        question: "Overwrite slot 02?".to_owned(),
        answers: "f yes / d no".to_owned(),
    });
    let mut buf = stale(&c);
    paint_slots(&c, &asking, &mut buf);
    assert!(shows(&buf, "Overwrite slot 02?"));
    assert!(shows(&buf, "f yes / d no"));
    // Centred, with a double border: its width is the question plus 4.
    let w = i32::try_from("Overwrite slot 02?".chars().count()).unwrap() + 4;
    let top = (i32::from(ConsoleSpec::PC.rows) - OVERWRITE_H) / 2;
    let left = (i32::from(ConsoleSpec::PC.cols) - w) / 2;
    assert_eq!(buf.get(left, top).unwrap().glyph, '╔');
    assert_eq!(
        buf.get(left + w - 1, top + OVERWRITE_H - 1).unwrap().glyph,
        '╝'
    );
    assert_eq!(
        buf.get(left, top).unwrap().fg,
        c.palette.get(UiColor::PanelBorderFocus)
    );
}
