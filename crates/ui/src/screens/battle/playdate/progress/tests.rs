//! Where the Playdate's EXP bar, level-up and class pages go and how they
//! look, in cells. What they say is tested on their view
//! (`battle/progress/tests.rs`).

use trpg_content::Ink;

use super::*;
use crate::map_view::playdate::tests::SCREEN;
use crate::screens::battle::glyph::progress::shows_face_of;
use crate::screens::battle::playdate::tests::ctx;
use crate::screens::battle::view::StatGainView;
use crate::screens::playdate::tests::{blank, c as ink, cell, find, is_bar, row};
use crate::tips::view::FaceView;

fn painted(view: &ProgressView) -> GlyphBuffer {
    let mut buf = blank();
    paint(&ctx(), view, SCREEN, &mut buf);
    buf
}

fn rows(buf: &GlyphBuffer, top: i32, n: i32) -> Vec<String> {
    (top..top + n).map(|y| row(buf, y)).collect()
}

fn stat(name: &str, before: i32, gain: i32) -> StatGainView {
    StatGainView {
        name: name.into(),
        before,
        gain,
    }
}

fn gains(face: Option<FaceView>, stats: Vec<StatGainView>) -> ProgressView {
    ProgressView::Gains(GainsView {
        face,
        placeholder: "portrait".into(),
        banner: "LEVEL UP!".into(),
        name: "Test Lord".into(),
        line: "Lv 1 → 2".into(),
        stats,
    })
}

/// Every stat that grows, `Str` and `Def` by one.
fn seven() -> Vec<StatGainView> {
    ["HP", "Str", "Mag", "Dex", "Spd", "Def", "Res"]
        .into_iter()
        .zip([19, 6, 1, 7, 7, 4, 3])
        .map(|(name, before)| stat(name, before, i32::from(matches!(name, "Str" | "Def"))))
        .collect()
}

#[test]
fn the_exp_bar_is_a_window_across_the_top_a_cell_per_five_exp() {
    let buf = painted(&ProgressView::Exp(ExpView {
        name: "Test Lord".into(),
        exp_name: "EXP".into(),
        exp: 40,
        max: 100,
    }));
    assert_eq!(
        rows(&buf, 0, 3),
        [
            format!("╔{}╗", "═".repeat(42)),
            format!("║ Test Lord    EXP ████████{} 40 ║", "░".repeat(12)),
            format!("╚{}╝", "═".repeat(42)),
        ]
    );
    assert_eq!(cell(&buf, 3, 0).glyph, '╔');
    let bar = cell(&buf, 22, 1);
    assert_eq!(
        (bar.glyph, bar.fg, bar.bg),
        ('█', ink(Ink::Black), ink(Ink::White))
    );
}

#[test]
fn the_level_up_page_is_a_window_in_the_middle_its_gains_the_bar() {
    let buf = painted(&gains(None, seven()));
    let at = gains_rect(SCREEN);
    assert_eq!(at, Rect::new(5, 2, GAINS_W, 11));
    // No face: the text takes its place.
    assert_eq!(
        rows(&buf, at.y, at.h),
        [
            format!("╔{}╗", "═".repeat(38)),
            format!("║ LEVEL UP!{}║", " ".repeat(28)),
            format!("║{}║", " ".repeat(38)),
            format!("║ Test Lord{}║", " ".repeat(28)),
            format!("║ Lv 1 → 2{}║", " ".repeat(29)),
            format!("║{}║", " ".repeat(38)),
            "║ HP   19            Str   6 → 7   +1  ║".to_owned(),
            "║ Mag   1            Dex   7           ║".to_owned(),
            "║ Spd   7            Def   4 → 5   +1  ║".to_owned(),
            format!("║ Res   3{}║", " ".repeat(30)),
            format!("╚{}╝", "═".repeat(38)),
        ]
    );
    let y = at.y + 6;
    assert!(is_bar(&buf, y, "+1"));
    assert!(!is_bar(&buf, y, "HP"));
    assert_eq!(find(&buf, y, "Str"), Some(at.x + 2 + SECOND));
}

#[test]
fn the_rows_appear_as_they_are_revealed_and_stay_where_they_go() {
    let all = painted(&gains(None, seven()));
    let three = painted(&gains(None, seven()[..3].to_vec()));
    let at = gains_rect(SCREEN);
    assert_eq!(row(&three, at.y + 6), row(&all, at.y + 6));
    assert_eq!(
        row(&three, at.y + 7),
        format!("║ Mag   1{}║", " ".repeat(30))
    );
    assert_eq!(row(&three, at.y + 8), format!("║{}║", " ".repeat(38)));
}

#[test]
fn a_face_is_drawn_at_one_times_in_the_top_left_corner() {
    let c = ctx();
    let art = &c.game.content.portraits["test_lord"];
    let face = FaceView {
        portrait: "test_lord".into(),
        expression: art.expressions[0].name.clone(),
    };
    let buf = painted(&gains(Some(face), seven()));
    assert!(shows_face_of(&buf, &c, "test_lord"));
    let at = gains_rect(SCREEN);
    let sprite = buf.sprites().last().copied().unwrap();
    assert_eq!(
        (sprite.dest.x, sprite.dest.y),
        ((at.x + 2) * 8, (at.y + 1) * 16)
    );
    assert_eq!((sprite.dest.w, sprite.dest.h), (64, 64));
    // The words beside it.
    assert_eq!(
        find(&buf, at.y + 1, "LEVEL UP!"),
        Some(at.x + 2 + FACE.0 + 2)
    );
    assert_eq!(find(&buf, at.y + 3, "Test Lord"), Some(at.x + 12));
}

#[test]
fn the_class_box_lists_what_was_learned_under_the_class_line() {
    let buf = painted(&ProgressView::Class(ClassProgressView {
        name: "Test Lord".into(),
        line: "Exile  CL 3 → 4".into(),
        mastered: Some("CLASS MASTERED!".into()),
        lines: vec!["Learned: Inspire".into()],
    }));
    assert_eq!(
        rows(&buf, 0, 5),
        [
            format!("╔{}╗", "═".repeat(42)),
            format!("║ Test Lord    Exile  CL 3 → 4{}║", " ".repeat(13)),
            format!("║ CLASS MASTERED!{}║", " ".repeat(26)),
            format!("║ Learned: Inspire{}║", " ".repeat(25)),
            format!("╚{}╝", "═".repeat(42)),
        ]
    );
}

#[test]
fn words_too_long_for_a_page_are_cut_inside_its_frame() {
    let long = format!("Test Lord {}", "x".repeat(60));
    // The level-up page: the name up to the frame's inside (36 cells).
    let buf = painted(&ProgressView::Gains(GainsView {
        face: None,
        placeholder: "portrait".into(),
        banner: "LEVEL UP!".into(),
        name: long.clone(),
        line: "Lv 1 → 2".into(),
        stats: Vec::new(),
    }));
    let at = gains_rect(SCREEN);
    assert_eq!(
        row(&buf, at.y + 3),
        format!("║ Test Lord {}… ║", "x".repeat(25))
    );
    // The class box: a learned line up to its inside (40 cells), the
    // class line after the name's column.
    let buf = painted(&ProgressView::Class(ClassProgressView {
        name: "Test Lord".into(),
        line: long.clone(),
        mastered: None,
        lines: vec![long],
    }));
    assert_eq!(
        row(&buf, 1),
        format!("║ Test Lord    Test Lord {}… ║", "x".repeat(16))
    );
    assert_eq!(row(&buf, 2), format!("║ Test Lord {}… ║", "x".repeat(29)));
}
