//! Where the Playdate's rewind list goes and how it looks, in cells. What
//! it says is tested on its view (`battle/rewind/tests.rs`,
//! `battle/rewind_tests.rs`).

use trpg_content::Ink;

use super::*;
use crate::map_view::playdate::tests::{SCREEN, plains};
use crate::screens::battle::playdate::tests::ctx;
use crate::screens::playdate::tests::{blank, c as ink, cell, find, is_bar};

fn list(entries: &[&str], focus: usize) -> RewindView {
    RewindView {
        title: "Rewind".into(),
        charges: "2 of 3 charges left".into(),
        spent: false,
        entries: entries.iter().map(|&e| e.to_owned()).collect(),
        focus,
        nothing: None,
        prompt: None,
    }
}

fn screen(list: RewindView) -> RewindScreenView {
    RewindScreenView {
        scene: plains(&ctx()),
        list,
        lesson: None,
        help: "A rewind · B back".into(),
        debug_hint: None,
    }
}

fn painted(view: &RewindScreenView) -> GlyphBuffer {
    let mut buf = blank();
    paint(&ctx(), view, &mut buf);
    buf
}

/// The list window's row `y`, from its left edge.
fn window_row(buf: &GlyphBuffer, y: i32) -> String {
    let at = list_rect(SCREEN);
    (at.x..at.x + at.w).map(|x| cell(buf, x, y).glyph).collect()
}

#[test]
fn the_list_is_a_window_down_the_right_side_over_the_map() {
    let view = screen(list(
        &[
            "Turn 2 · Test Lord attacked Brigand (hit, 8 dmg)",
            "Turn 1 · Archer waited",
        ],
        1,
    ));
    let buf = painted(&view);
    assert_eq!(list_rect(SCREEN), Rect::new(25, 0, W, 15));
    let shown: Vec<String> = (0..9).map(|y| window_row(&buf, y)).collect();
    assert_eq!(
        shown,
        [
            "╔═ Rewind ══════════════╗",
            "║ 2 of 3 charges left   ║",
            "║                       ║",
            "║ Turn 2 · Test Lord    ║",
            "║   attacked Brigand    ║",
            "║   (hit, 8 dmg)        ║",
            "║ Turn 1 · Archer       ║",
            "║   waited              ║",
            "║                       ║",
        ]
    );
    // The highlighted entry is the bar; the map shows left of the window.
    assert!(is_bar(&buf, 6, "Turn 1") && is_bar(&buf, 7, "waited"));
    assert!(!is_bar(&buf, 3, "Turn 2"));
    assert_eq!(cell(&buf, 25, 14).glyph, '╚');
    let ground = cell(&buf, 0, 7);
    assert_eq!(ground.bg, ink(Ink::White));
}

#[test]
fn a_long_list_scrolls_to_the_highlighted_entry_and_marks_the_rest() {
    let lines: Vec<String> = (1..=20).map(|i| format!("Turn {i} · Ana waited")).collect();
    let entries: Vec<&str> = lines.iter().map(String::as_str).collect();
    // The first: the top of the list, more below.
    let buf = painted(&screen(list(&entries, 0)));
    assert!(is_bar(&buf, 3, "Turn 1 "));
    assert_eq!(window_row(&buf, 14), format!("╚{}▼═╝", "═".repeat(21)));
    assert_eq!(
        window_row(&buf, 0),
        format!("╔═ Rewind {}╗", "═".repeat(14))
    );
    // The last: the list's end in view, more above.
    let buf = painted(&screen(list(&entries, 19)));
    assert!(is_bar(&buf, 13, "Turn 20"));
    assert_eq!(
        window_row(&buf, 0),
        format!("╔═ Rewind {}▲═╗", "═".repeat(12))
    );
    assert_eq!(window_row(&buf, 14), format!("╚{}╝", "═".repeat(23)));
}

#[test]
fn nothing_to_rewind_says_so_under_the_charges() {
    let mut nothing = list(&[], 0);
    nothing.nothing = Some("Nothing to rewind yet".into());
    let buf = painted(&screen(nothing));
    assert_eq!(window_row(&buf, 3), "║ Nothing to rewind yet ║");
}

#[test]
fn the_charge_question_is_a_window_in_the_middle_with_its_buttons_inside() {
    let mut asked = list(&["Turn 1 · Archer waited"], 0);
    asked.prompt = Some("Use 1 of 2 charges?".into());
    let buf = painted(&screen(asked));
    let y = (0..15).find(|&y| find(&buf, y, "Use 1 of 2 charges?").is_some());
    let y = y.expect("the question");
    assert!(find(&buf, y + 2, "A rewind · B back").is_some());
    // In the middle, over the map and the list.
    let x = find(&buf, y, "Use 1 of 2 charges?").unwrap();
    assert!(x < 25 && x + 19 > 25, "{x}");
}
