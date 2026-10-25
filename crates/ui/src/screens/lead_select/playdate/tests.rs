//! The Playdate's look of the lead screen, on the screen's own views.

use super::*;
use crate::input::Action::{CursorDown, CursorRight, CursorUp};
use crate::screens::lead_select::LeadSelectScreen;
use crate::screens::lead_select::tests::{spelling, update};
use crate::screens::playdate::tests::{blank, cell, find, is_bar, pd_ctx, rows};

/// The screen's view, painted on the Playdate's screen.
fn painted(c: &Ctx, s: &LeadSelectScreen) -> GlyphBuffer {
    let mut buf = blank();
    paint(c, &s.view(c), &mut buf);
    buf
}

#[test]
fn the_portraits_side_by_side_the_name_start_and_the_help() {
    let c = pd_ctx();
    let buf = painted(&c, &LeadSelectScreen::new());
    let shown = rows(&buf);
    assert_eq!(shown[0], "Choose your lead");
    // The chosen portrait (male) in a double frame, the other single.
    assert_eq!(cell(&buf, FRAME_X[0], FRAME_Y).glyph, '╔');
    assert_eq!(cell(&buf, FRAME_X[1], FRAME_Y).glyph, '┌');
    let bottom = FRAME_Y + FRAME.1 - 1;
    assert_eq!(cell(&buf, FRAME_X[1] + FRAME.0 - 1, bottom).glyph, '┘');
    assert_eq!(bottom, LABEL_ROW - 1);
    // Each portrait at 2× inside its frame, the one not chosen faded.
    let sprites = buf.sprites();
    assert_eq!(sprites.len(), 2);
    let inside = |x: i32| (x + 1) * 8..(x + FRAME.0 - 1) * 8;
    for (sprite, x) in sprites.iter().zip(FRAME_X) {
        assert!(inside(x).contains(&sprite.dest.x), "{sprite:?}");
        assert!(sprite.dest.y >= (FRAME_Y + 1) * 16);
        assert!(sprite.dest.y + sprite.dest.h <= bottom * 16);
    }
    assert_eq!(sprites[0].opacity, 255);
    assert!(sprites[1].opacity < 255);
    // The names under them, the chosen one the bar on the portraits row.
    assert!(is_bar(&buf, LABEL_ROW, " Male "));
    assert!(!is_bar(&buf, LABEL_ROW, "Female"));
    assert_eq!(
        shown[usize::try_from(NAME_ROW).unwrap()],
        "Name   Ellery Veyne"
    );
    assert_eq!(shown[usize::try_from(START_ROW).unwrap()], "[ Start ]");
    assert_eq!(shown[14], LeadSelectScreen::new().view(&c).help);
}

#[test]
fn the_cursor_is_the_bar_on_its_row() {
    let mut c = pd_ctx();
    let mut s = LeadSelectScreen::new();
    // Female chosen, then down to the name: her frame double, no bar on
    // the names.
    update(&mut s, &mut c, &[CursorRight, CursorDown]);
    let buf = painted(&c, &s);
    assert_eq!(cell(&buf, FRAME_X[0], FRAME_Y).glyph, '┌');
    assert_eq!(cell(&buf, FRAME_X[1], FRAME_Y).glyph, '╔');
    assert!(!is_bar(&buf, LABEL_ROW, "Female"));
    assert!(is_bar(&buf, NAME_ROW, " Ellery "));
    assert!(!is_bar(&buf, NAME_ROW, "Veyne"));
    update(&mut s, &mut c, &[CursorDown]);
    let buf = painted(&c, &s);
    assert!(is_bar(&buf, START_ROW, " [ Start ] "));
    assert!(!is_bar(&buf, NAME_ROW, " Ellery "));
}

#[test]
fn the_letter_grid_opens_in_a_window_over_the_screen() {
    let mut c = pd_ctx();
    let s = spelling(&mut c);
    let buf = painted(&c, &s);
    let shown = rows(&buf);
    // Five rows of cells: nine rows, from row 3.
    assert!(shown[3].starts_with("╔═ First name"), "{shown:#?}");
    assert!(shown[4].contains("Ellery_"));
    assert!(shown[11].starts_with('╚'));
    assert!(is_bar(&buf, 6, " A "));
    assert!(find(&buf, 10, "Done").is_some());
    // The help is still on the bottom row.
    assert_ne!(shown[14], "");
}

#[test]
fn the_start_bar_is_its_button_and_a_cell_either_side() {
    let mut c = pd_ctx();
    let mut s = LeadSelectScreen::new();
    update(&mut s, &mut c, &[CursorDown, CursorDown]);
    let buf = painted(&c, &s);
    let black = crate::screens::playdate::tests::c(trpg_content::Ink::Black);
    let x = find(&buf, START_ROW, "[ Start ]").unwrap();
    // Centred: nine cells on fifty.
    assert_eq!(x, 20);
    let bar = |x| cell(&buf, x, START_ROW).bg == black;
    assert!(!bar(x - 2) && bar(x - 1) && bar(x + 9) && !bar(x + 10));
    // On the name: the first name and a cell either side, not the family
    // name.
    update(&mut s, &mut c, &[CursorUp]);
    let buf = painted(&c, &s);
    let first = find(&buf, NAME_ROW, "Ellery").unwrap();
    let family = find(&buf, NAME_ROW, "Veyne").unwrap();
    let bar = |x| cell(&buf, x, NAME_ROW).bg == black;
    assert_eq!(family, first + 7);
    assert!(!bar(first - 2) && bar(first - 1) && bar(first + 6) && !bar(family));
}
