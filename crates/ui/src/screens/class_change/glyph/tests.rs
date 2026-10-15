//! Tests of where the class-choice screen puts what its view says, on the
//! debug menu's test knight (`class_change::tests`) and on rows made by
//! hand.

use super::*;
use crate::console::ConsoleSpec;
use crate::input::Action::{Cancel, Confirm, CursorLeft, CursorRight};
use crate::screen::tests::ctx;
use crate::screens::class_change::tests::{demo, render, step};
use crate::screens::class_change::{ChangeKind, ClassChangeScreen};
use trpg_core::{ClassId, ClassRecord, Stock};

/// The text of `w` cells of row `y` from column `x`, trimmed.
fn text(buf: &GlyphBuffer, x: i32, y: i32, w: i32) -> String {
    (x..x + w)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The text inside column `slot` (0 = leftmost shown) on row `row` of it.
fn cell(buf: &GlyphBuffer, slot: i32, row: i32) -> String {
    text(buf, COLUMNS_X + slot * COLUMN_W + 2, COLUMNS_Y + row, 20)
}

/// Whether some row of the screen contains `needle`.
fn shows(buf: &GlyphBuffer, needle: &str) -> bool {
    (0..i32::from(ConsoleSpec::PC.rows))
        .any(|y| text(buf, 0, y, i32::from(ConsoleSpec::PC.cols)).contains(needle))
}

/// A stat row made by hand.
fn stat(name: &str, before: i32, after: i32, at_max: bool) -> StatRowView {
    StatRowView {
        name: name.to_owned(),
        before,
        after,
        at_max,
    }
}

/// The knight: HP 28, Str 11, Mag 0, Dex 7, Spd 5, Def 49, Res 0. Bulwark
/// base `28 11 0 6 4 14 2` less Guard `20 7 0 4 2 9 0`; Iron Rider
/// `27 10 0 6 6 11 2`.
#[test]
fn the_columns_show_each_stat_before_and_after_and_mark_a_maxed_one() {
    let c = ctx();
    let s = demo(&c, ChangeKind::Promote);
    let buf = render(&s, &c);
    assert_eq!(text(&buf, 2, 1, 40), "Promotion");
    assert_eq!(text(&buf, 2, 2, 40), "Test Knight  Guard  Lv 12");
    assert_eq!(text(&buf, 70, 2, 28), "Tier 2 Seal ×1");
    let column = |slot| (1..24).map(|row| cell(&buf, slot, row)).collect::<Vec<_>>();
    assert_eq!(
        column(0),
        [
            "Bulwark",
            "Tier 2",
            "",
            "Mov   4     armored",
            "Armored",
            "",
            "Weapons",
            "Spear    D → C",
            "Axe      E → D",
            "",
            "",
            "Active",
            "Fortify",
            "",
            "HP   28 → 36  +8",
            "Str  11 → 15  +4",
            "Mag   0",
            "Dex   7 → 9   +2",
            "Spd   5 → 7   +2",
            // Def +5 is cut at the hard ceiling (50).
            "Def  49 → 50  +1 MAX",
            "Res   0 → 2   +2",
            "",
            "",
        ]
    );
    assert_eq!(
        column(1),
        [
            "Iron Rider",
            "Tier 2",
            "",
            "Mov   4 → 6 mounted",
            "Mounted Armored",
            "",
            "Weapons",
            "Spear    D → C",
            "Axe      E → D",
            "Sword    E → D",
            "",
            "Active",
            "Trample",
            "",
            "HP   28 → 35  +7",
            "Str  11 → 14  +3",
            "Mag   0",
            "Dex   7 → 9   +2",
            "Spd   5 → 9   +4",
            "Def  49 → 50  +1 MAX",
            "Res   0 → 2   +2",
            "",
            "",
        ]
    );
    // Two columns only: nothing past them, no scroll marks.
    assert_eq!(text(&buf, COLUMNS_X + 2 * COLUMN_W, COLUMNS_Y, 48), "");
    assert_eq!(text(&buf, 0, MORE_ROW, 100), "");
    // The gain and the mark are highlighted; the focused column is boxed
    // double, the other single.
    let hi = c.palette.get(UiColor::TextHighlight);
    let at = |x: i32, row: i32| buf.get(COLUMNS_X + 2 + x, COLUMNS_Y + row).unwrap();
    assert_eq!((at(14, 15).glyph, at(14, 15).fg), ('+', hi));
    assert_eq!((at(17, 20).glyph, at(17, 20).fg), ('M', hi));
    assert_eq!(at(0, 15).fg, c.palette.get(UiColor::Text));
    assert_eq!(at(0, 17).fg, c.palette.get(UiColor::TextDim));
    assert_eq!(buf.get(COLUMNS_X, COLUMNS_Y).unwrap().glyph, '╔');
    assert_eq!(buf.get(COLUMNS_X + COLUMN_W, COLUMNS_Y).unwrap().glyph, '┌');
    // The help line, at the bottom.
    let bottom = i32::from(ConsoleSpec::PC.rows) - 1;
    assert_eq!(
        text(&buf, 0, bottom, 100),
        "Left/Right class · f choose · d back"
    );
}

#[test]
fn long_text_is_cut_inside_the_column() {
    assert_eq!(i32::try_from(TEXT_W).unwrap() + 4, COLUMN_W);
    let mut c = ctx();
    if let Some(class) = c
        .content
        .classes
        .classes
        .get_mut(&ClassId("bulwark".into()))
    {
        class.name = "Sentinel of the High Marches".into();
    }
    let s = demo(&c, ChangeKind::Promote);
    let buf = render(&s, &c);
    assert_eq!(cell(&buf, 0, 1), "Sentinel of the High");
    // The borders on both sides of the cut are whole.
    let border = |x: i32| buf.get(x, COLUMNS_Y + 1).unwrap().glyph;
    assert_eq!(border(COLUMNS_X + COLUMN_W - 2), ' ');
    assert_eq!(border(COLUMNS_X + COLUMN_W - 1), '║');
    assert_eq!(border(COLUMNS_X + COLUMN_W), '│');
}

#[test]
fn stat_rows() {
    let line = |row: StatRowView| stat_line(&row, "MAX");
    assert_eq!(line(stat("Str", 9, 12, false)), "Str   9 → 12  +3");
    assert_eq!(line(stat("Mag", 1, 1, false)), "Mag   1");
    assert_eq!(line(stat("Def", 48, 50, true)), "Def  48 → 50  +2 MAX");
    // Already at the ceiling, or past it.
    assert_eq!(line(stat("Res", 50, 50, true)), "Res  50 MAX");
    assert_eq!(line(stat("HP", 81, 81, true)), "HP   81 MAX");
    assert_eq!(line(stat("HP", 70, 79, false)), "HP   70 → 79  +9");
    // What is lit: from the gain on, or just the mark.
    let parts = |row: StatRowView| stat_parts(&row, "MAX");
    assert_eq!(
        parts(stat("Def", 48, 50, true)),
        ("Def  48 → 50  ".to_owned(), "+2 MAX".to_owned())
    );
    assert_eq!(
        parts(stat("Res", 50, 50, true)),
        ("Res  50 ".to_owned(), "MAX".to_owned())
    );
    assert_eq!(
        parts(stat("Mag", 1, 1, false)),
        ("Mag   1".to_owned(), String::new())
    );
    // The mark is the view's, in the player's language.
    assert_eq!(
        stat_line(&stat("Res", 50, 50, true), "最大"),
        "Res  50 最大"
    );
}

#[test]
fn the_question_is_a_box_in_the_middle_and_the_pages_replace_the_columns() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    step(&mut s, &mut c, &[CursorRight, Confirm]);
    let buf = render(&s, &c);
    for line in [
        "║ Promote Test Knight to Iron Rider? ║",
        "║ Uses 1 Tier 2 Seal (1 in stock).   ║",
        "║ f yes / d no                       ║",
    ] {
        assert!(shows(&buf, line), "{line}");
    }
    step(&mut s, &mut c, &[Cancel]);
    assert!(!shows(&render(&s, &c), "Promote Test Knight"));
    step(&mut s, &mut c, &[Confirm, Confirm, Confirm]);
    let buf = render(&s, &c);
    assert_eq!(text(&buf, 43, 7, 20), "PROMOTED!");
    assert_eq!(text(&buf, 43, 9, 20), "Test Knight");
    assert_eq!(text(&buf, 43, 10, 20), "Guard → Iron Rider");
    assert_eq!(text(&buf, 43, 12, 20), "HP   28 → 35  +7");
    assert_eq!(text(&buf, 43, 17, 20), "Def  49 → 50  +1");
    // Nothing else is on screen.
    assert_eq!(text(&buf, 0, 2, 100), "");
}

#[test]
fn the_leads_promotion_page_shows_the_face_the_player_picked() {
    use crate::screens::battle::glyph::progress::shows_face_of;
    use trpg_core::CharacterId;
    use trpg_core::lead::{LEAD_ID, LeadGender, LeadProfile};
    for (gender, face) in [(LeadGender::Male, "lead_m"), (LeadGender::Female, "lead_f")] {
        let mut c = ctx();
        c.lead = LeadProfile::new("Ren", gender);
        let mut s = demo(&c, ChangeKind::Promote);
        s.unit.character = Some(CharacterId(LEAD_ID.into()));
        step(&mut s, &mut c, &[Confirm, Confirm]);
        let buf = render(&s, &c);
        assert_eq!(text(&buf, 43, 7, 20), "PROMOTED!");
        assert!(shows_face_of(&buf, &c, face), "{face}");
        assert!(!shows_face_of(&buf, &c, "test_knight"));
    }
}

#[test]
fn a_refused_class_is_dimmed_with_the_reason() {
    let c = ctx();
    let knight = demo(&c, ChangeKind::Promote).unit().clone();
    // No seal in stock.
    let s = ClassChangeScreen::new(&c, ChangeKind::Promote, knight.clone(), Stock::default());
    let buf = render(&s, &c);
    assert_eq!(cell(&buf, 0, 23), "No Tier 2 Seal");
    // Its stats stay as they are, dimmed like the rest.
    assert_eq!(cell(&buf, 0, 15), "HP   28");
    assert_eq!(cell(&buf, 0, 20), "Def  49");
    let dim = c.palette.get(UiColor::TextDim);
    let at = |x: i32, row: i32| buf.get(COLUMNS_X + 2 + x, COLUMNS_Y + row).unwrap();
    assert_eq!((at(0, 1).fg, at(0, 13).fg), (dim, dim));
    assert_eq!(at(0, 23).fg, c.palette.get(UiColor::HpLow));
    // No seal in stock: the count is in the warning colour.
    assert_eq!(text(&buf, 70, 2, 28), "Tier 2 Seal ×0");
    assert_eq!(buf.get(97, 2).unwrap().fg, c.palette.get(UiColor::HpLow));
    // Not mastered.
    let mut fresh = knight;
    fresh
        .class_records
        .insert(ClassId("guard".into()), ClassRecord::UNLOCKED);
    let mut stock = Stock::default();
    stock.add(trpg_core::ItemId::new("tier_2_seal"));
    let s = ClassChangeScreen::new(&c, ChangeKind::Promote, fresh, stock);
    let buf = render(&s, &c);
    assert_eq!(cell(&buf, 1, 23), "Class not mastered");
    // With one in stock the count is plain.
    assert_eq!(text(&buf, 70, 2, 28), "Tier 2 Seal ×1");
    assert_eq!(buf.get(97, 2).unwrap().fg, c.palette.get(UiColor::Text));
}

#[test]
fn a_unit_with_nowhere_to_go_sees_an_empty_screen() {
    let c = ctx();
    let mut top = demo(&c, ChangeKind::Promote).unit().clone();
    top.class = ClassId("bastion".into());
    let s = ClassChangeScreen::new(&c, ChangeKind::Promote, top, Stock::default());
    let buf = render(&s, &c);
    assert_eq!(text(&buf, 0, COLUMNS_Y + 1, 100), "No class to change to.");
}

#[test]
fn the_columns_scroll_with_marks_over_them() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Reclass);
    let buf = render(&s, &c);
    assert_eq!(text(&buf, 2, 1, 40), "Class change");
    assert_eq!(text(&buf, 70, 2, 28), "Reclass Seal ×1");
    let names = |buf: &GlyphBuffer| (0..4).map(|slot| cell(buf, slot, 1)).collect::<Vec<_>>();
    assert_eq!(names(&buf), ["Archer", "Brawler", "Cleric", "Mage"]);
    assert_eq!(text(&buf, 0, MORE_ROW, 100), "more →");
    // A class never entered is `new`; no stat changes.
    assert_eq!(cell(&buf, 0, 2), "Tier 1   new");
    assert_eq!(cell(&buf, 0, 4), "Mov   4 → 5 foot");
    assert_eq!(cell(&buf, 0, 15), "HP   28");
    assert_eq!(cell(&buf, 0, 20), "Def  49");
    for _ in 0..5 {
        step(&mut s, &mut c, &[CursorRight]);
    }
    let buf = render(&s, &c);
    assert_eq!(names(&buf), ["Cleric", "Mage", "Raider", "Rider"]);
    assert_eq!(
        text(&buf, 0, MORE_ROW, 100).replace("  ", ""),
        "← moremore →"
    );
    // The Rider's saved class level; its box is the double one.
    assert_eq!(cell(&buf, 3, 2), "Tier 1   CL 6");
    assert_eq!(
        buf.get(COLUMNS_X + 3 * COLUMN_W, COLUMNS_Y).unwrap().glyph,
        '╔'
    );
    step(&mut s, &mut c, &[CursorRight]);
    let buf = render(&s, &c);
    assert_eq!(names(&buf), ["Mage", "Raider", "Rider", "Swordsman"]);
    assert!(text(&buf, 0, MORE_ROW, 100).ends_with("more →"));
    for _ in 0..3 {
        step(&mut s, &mut c, &[CursorRight]);
    }
    assert_eq!(cell(&render(&s, &c), 0, 1), "Archer");
    step(&mut s, &mut c, &[CursorLeft]);
    let buf = render(&s, &c);
    assert_eq!(names(&buf), ["Rider", "Swordsman", "Bulwark", "Iron Rider"]);
    assert_eq!(text(&buf, 0, MORE_ROW, 100), "← more");
    assert_eq!(cell(&buf, 3, 15), "HP   28");
    assert_eq!(cell(&buf, 3, 2), "Tier 2   new");
}

#[test]
fn the_reclass_question_and_page() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Reclass);
    step(&mut s, &mut c, &[CursorRight, CursorRight, CursorRight]);
    step(&mut s, &mut c, &[Confirm]);
    let buf = render(&s, &c);
    assert!(shows(&buf, "║ Change Test Knight to Mage?       ║"));
    assert!(shows(&buf, "║ Uses 1 Reclass Seal (1 in stock). ║"));
    step(&mut s, &mut c, &[Confirm]);
    let buf = render(&s, &c);
    assert!(shows(&buf, "Test Knight  Guard → Mage"));
    assert!(shows(&buf, "Learned: Fire"));
    assert!(shows(&buf, "To stock: Chain Mail"));
}

#[test]
fn every_cell_is_painted_and_an_empty_view_shows_only_the_help() {
    let c = ctx();
    let s = demo(&c, ChangeKind::Promote);
    for view in [
        s.view(&c),
        ClassChangeView {
            body: ClassChangeBody::Done,
            help: "help".to_owned(),
        },
    ] {
        let buf = render_view(&c, &view);
        let left = (0..i32::from(ConsoleSpec::PC.rows))
            .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
            .filter(|&(x, y)| buf.get(x, y).is_some_and(|cell| cell.glyph == STALE))
            .count();
        assert_eq!(left, 0, "{left} stale cells on {:?}", view.body);
    }
    let done = ClassChangeView {
        body: ClassChangeBody::Done,
        help: "help".to_owned(),
    };
    let buf = render_view(&c, &done);
    let bottom = i32::from(ConsoleSpec::PC.rows) - 1;
    assert_eq!(text(&buf, 0, bottom, 100), "help");
    assert!(!shows(&buf, "Promotion"));
}

/// What fills a cell nothing has painted (the screen never shows it).
const STALE: char = '¤';

/// `view` painted over a buffer of stale cells.
fn render_view(c: &Ctx, view: &ClassChangeView) -> GlyphBuffer {
    let enemy = c.palette.get(UiColor::Enemy);
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(STALE, enemy, enemy));
    paint(c, view, &mut buf);
    buf
}
