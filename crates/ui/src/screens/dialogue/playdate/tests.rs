//! Tests of the dialogue screen's Playdate look: the faces, names, text
//! box, replies, skip question and caption on the 50 × 15 screen, and the
//! place darkened in one bit. What the screen shows is tested on its view
//! in `../tests.rs`.

use super::*;
use crate::glyph_buffer::Item as BufItem;
use crate::screens::dialogue::view::{OpeningView, SkipView};
use crate::screens::playdate::tests::{blank, c, cell, find, is_bar, pd_ctx, row};
use crate::widgets::menu::{MenuItemView, MenuView};

fn face(portrait: &str, name: &str) -> FaceView {
    FaceView {
        portrait: portrait.to_owned(),
        expression: "neutral".to_owned(),
        name: name.to_owned(),
    }
}

/// A page of `lines`, all shown, waiting for nothing.
fn text(lines: &[&str]) -> TextView {
    let lines: Vec<String> = lines.iter().map(|&l| l.to_owned()).collect();
    let shown = lines.iter().map(|l| l.chars().count()).sum();
    TextView::new(lines, shown)
}

/// Lord on the left saying `lines` to the knight on the right.
fn two_speakers(lines: &[&str]) -> DialogueView {
    DialogueView {
        overlay: false,
        place: None,
        opening: None,
        bright: None,
        caption: None,
        left: Some(face("test_lord", "Test Lord")),
        right: Some(face("test_knight", "Test Knight")),
        speaker: Some(Side::Left),
        text: text(lines),
        log: None,
    }
}

/// Replies labelled `labels`, the `focus`th focused.
fn replies(labels: &[&str], focus: usize) -> MenuView {
    let item = |(i, &label): (usize, &&str)| MenuItemView {
        label: label.into(),
        suffix: None,
        note: None,
        enabled: true,
        focused: i == focus,
    };
    MenuView {
        items: labels.iter().enumerate().map(item).collect(),
        first_shown: 0,
        shown: labels.len(),
    }
}

fn painted(view: &DialogueView) -> GlyphBuffer {
    let mut buf = blank();
    paint(&pd_ctx(), view, &mut buf);
    buf
}

/// The sprites painted, as (top-left cell, opacity).
fn sprites(buf: &GlyphBuffer) -> Vec<((i32, i32), u8)> {
    let at = |s: &crate::glyph_buffer::Sprite| ((s.dest.x / 8, s.dest.y / 16), s.opacity);
    buf.items()
        .iter()
        .filter_map(|item| match item {
            BufItem::Sprite(s) => Some(at(s)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_faces_at_2x_their_names_under_them_and_a_three_line_box_below() {
    let line = "x".repeat(super::super::PLAYDATE_TEXT_W);
    let buf = painted(&two_speakers(&["Hi.", "Two.", &line]));
    // Whoever speaks in a double frame, the listener in a single one.
    assert_eq!(cell(&buf, 1, 0).glyph, '╔');
    assert_eq!(cell(&buf, 31, 0).glyph, '┌');
    assert_eq!(cell(&buf, 48, 9).glyph, '┘');
    // Each face at 2× (16 × 8 cells) in its frame, the listener's faded.
    let faces = sprites(&buf);
    assert_eq!(faces.len(), 2, "{faces:?}");
    let ((x, y), lit) = faces[0];
    assert!((2..=3).contains(&x) && y >= 1, "{faces:?}");
    assert_eq!(lit, 255);
    assert!(faces[1].1 < 255, "{faces:?}");
    let big = buf.sprites()[0].dest;
    assert_eq!((big.w, big.h), (128, 128));
    // The names in the bottom of the frames.
    assert_eq!(find(&buf, 9, "Test Lord"), Some(5));
    assert!(find(&buf, 9, "Test Knight").is_some());
    // The text box: rows 10 to 14, a window, three lines from column 2;
    // a 47-letter line ends a cell before its right edge.
    assert_eq!(cell(&buf, 0, 10).glyph, '╔');
    assert_eq!(cell(&buf, 49, 14).glyph, '╝');
    assert_eq!(find(&buf, 11, "Hi."), Some(TEXT_X));
    assert_eq!(find(&buf, 12, "Two."), Some(TEXT_X));
    assert_eq!(cell(&buf, 48, 13).glyph, 'x');
    assert_eq!(cell(&buf, 49, 13).glyph, '║');
    assert_eq!(cell(&buf, TEXT_X, 11).fg, c("text"));
    assert_eq!(cell(&buf, TEXT_X, 11).bg, c("panel_bg"));
}

#[test]
fn a_name_too_long_for_its_frame_is_cut_inside_the_corners() {
    let mut view = two_speakers(&["Hi."]);
    view.left = Some(face("test_lord", "Lord Aldric Fairweather"));
    let buf = painted(&view);
    assert_eq!(cell(&buf, 1, 9).glyph, '╚');
    assert_eq!(cell(&buf, 18, 9).glyph, '╝');
    assert_eq!(find(&buf, 9, "Lord Aldric"), Some(3));
    // The bust in the frame fills it: 16 × 8 cells.
    assert_eq!(BUST, (FRAME.0 - 2, FRAME.1 - 2));
}

#[test]
fn narration_lights_neither_face_and_reads_as_speech() {
    let mut view = two_speakers(&["The rain stops."]);
    view.speaker = None;
    view.text.narration = true;
    let buf = painted(&view);
    assert_eq!(cell(&buf, 1, 0).glyph, '┌');
    assert_eq!(cell(&buf, 31, 0).glyph, '┌');
    assert!(sprites(&buf).iter().all(|&(_, opacity)| opacity < 255));
    assert_eq!(find(&buf, 11, "The rain stops."), Some(TEXT_X));
    assert_eq!(cell(&buf, TEXT_X, 11).fg, c("text"));
}

#[test]
fn the_right_face_is_mirrored_and_a_lone_face_leaves_the_other_side_empty() {
    let mut view = two_speakers(&["Hi."]);
    let buf = painted(&view);
    let flipped: Vec<bool> = buf.sprites().iter().map(|s| s.flip_x).collect();
    assert_eq!(flipped, [false, true]);
    view.right = None;
    let buf = painted(&view);
    assert_eq!(sprites(&buf).len(), 1);
    assert_eq!(cell(&buf, 31, 0).glyph, ' ');
}

#[test]
fn the_caption_goes_across_the_top_row() {
    let mut view = two_speakers(&["Hi."]);
    view.caption = Some("Dunmere, dawn".to_owned());
    let buf = painted(&view);
    let x = find(&buf, 0, "Dunmere, dawn").unwrap();
    assert_eq!(x, (50 - 13) / 2);
    assert_eq!(cell(&buf, x, 0).bg, c("panel_bg"));
}

#[test]
fn a_choice_grows_the_box_over_the_faces_and_bars_the_focused_reply() {
    let mut view = two_speakers(&["Will you come?"]);
    view.text.choices = Some(replies(&["Yes.", "No."], 1));
    let buf = painted(&view);
    // One line, a blank row, two replies: four rows above the bottom
    // border, so the box's top is row 9.
    let (rect, text_y) = box_rect(&view.text);
    assert_eq!((rect.y, rect.h, text_y), (9, 6, 10));
    assert_eq!(cell(&buf, 0, 9).glyph, '╔');
    assert_eq!(find(&buf, 10, "Will you come?"), Some(TEXT_X));
    assert_eq!(row(&buf, 11), format!("║{}║", " ".repeat(48)));
    assert_eq!(find(&buf, 12, "Yes."), Some(TEXT_X));
    assert!(!is_bar(&buf, 12, "Yes."));
    assert!(is_bar(&buf, 13, "No."));
    // The bar runs across the box, inside its frame.
    assert_eq!(cell(&buf, 48, 13).bg, c("text"));
    assert_eq!(cell(&buf, 49, 13).glyph, '║');
    // Replies with no line before them: no blank row either.
    let mut view = two_speakers(&[]);
    view.text.choices = Some(replies(&["1", "2", "3", "4"], 0));
    assert_eq!(box_rect(&view.text).0.y, 9);
    // A page of three lines and three replies: the box from row 6.
    let mut view = two_speakers(&["a", "b", "c"]);
    view.text.choices = Some(replies(&["1", "2", "3"], 0));
    assert_eq!(box_rect(&view.text).0.y, 6);
}

#[test]
fn more_lines_than_a_page_grow_the_box_upward() {
    // A talk shot's text in the Playdate's narrower lines.
    let view = two_speakers(&["1", "2", "3", "4", "5"]);
    let (rect, text_y) = box_rect(&view.text);
    assert_eq!((rect.y, rect.y + rect.h, text_y), (8, 15, 9));
    let buf = painted(&view);
    assert_eq!(find(&buf, 13, "5"), Some(TEXT_X));
}

#[test]
fn the_waiting_page_shows_its_key_and_marker_in_the_bottom_of_the_frame() {
    let mut view = two_speakers(&["Hi."]);
    let mut wait = WaitView::new("A", true);
    view.text.waiting = Some(wait.clone());
    let buf = painted(&view);
    assert!(row(&buf, 14).ends_with("═ A ▼ ═╝"), "{}", row(&buf, 14));
    // The marker blinks: unlit, a gap in the frame.
    wait.marker = false;
    wait.log_hint = Some("Menu log".to_owned());
    view.text.waiting = Some(wait);
    let buf = painted(&view);
    assert!(
        row(&buf, 14).ends_with("═ Menu log · A   ═╝"),
        "{}",
        row(&buf, 14)
    );
}

#[test]
fn the_skip_question_is_asked_in_the_box() {
    let mut view = two_speakers(&["Hi."]);
    view.text.skip = Some(SkipView {
        question: "Skip this scene?".to_owned(),
        answers: "A yes · B no".to_owned(),
    });
    let buf = painted(&view);
    assert_eq!(find(&buf, 11, "Skip this scene?"), Some(TEXT_X));
    assert_eq!(find(&buf, 12, "A yes · B no"), Some(TEXT_X));
    assert!(find(&buf, 11, "Hi.").is_none());
}

/// The test map `test_small` as the Playdate screen's place.
fn small_place(ctx: &Ctx) -> crate::map_view::MapScene {
    let place = crate::map_view::Place::of_map(&ctx.game.content.maps["test_small"]);
    let tiles = ctx.game.map_skin.view_tiles(Rect::new(0, 0, 50, 15));
    place.scene(trpg_core::Pos::new(7, 4), tiles)
}

/// The place as the Playdate's map skin paints it, on the white ground.
fn place_alone(ctx: &Ctx, scene: &crate::map_view::MapScene) -> GlyphBuffer {
    let mut buf = blank();
    ctx.game.map_skin.paint(ctx, scene, buf.bounds(), &mut buf);
    buf
}

#[test]
fn the_place_shows_around_the_talk_its_white_ground_dotted() {
    let ctx = pd_ctx();
    let scene = small_place(&ctx);
    let alone = place_alone(&ctx, &scene);
    let mut view = two_speakers(&["Hi."]);
    view.place = Some(scene);
    let mut buf = blank();
    paint(&ctx, &view, &mut buf);
    // Between the frames the place shows: its glyphs as they are, its
    // white ground light dots.
    let mut dotted = 0;
    for y in 0..10 {
        for x in 19..31 {
            let (was, now) = (cell(&alone, x, y), cell(&buf, x, y));
            assert_eq!((now.glyph, now.fg), (was.glyph, was.fg), "{x},{y}");
            if was.bg == c("panel_bg") {
                assert_eq!(now.bg, c("move_range"), "{x},{y}");
                dotted += 1;
            } else {
                assert_eq!(now.bg, was.bg, "{x},{y}");
            }
        }
    }
    assert!(dotted > 0);
    // The faces and the box are on white, over it.
    assert_eq!(cell(&buf, 5, 12).bg, c("panel_bg"));
}

#[test]
fn an_opening_shows_the_place_undarkened_then_cuts_to_dark_halfway() {
    let ctx = pd_ctx();
    let scene = small_place(&ctx);
    let alone = place_alone(&ctx, &scene);
    let mut view = two_speakers(&["Hi."]);
    view.place = Some(scene);
    view.caption = Some("The village".to_owned());
    view.opening = Some(OpeningView { darkened: 0.4 });
    let mut buf = blank();
    paint(&ctx, &view, &mut buf);
    // No faces and no text box while it plays; the caption is up.
    assert_eq!(sprites(&buf), []);
    assert!(find(&buf, 0, "The village").is_some());
    assert_eq!(buf.get(25, 12), alone.get(25, 12));
    view.opening = Some(OpeningView { darkened: 0.5 });
    let mut buf = blank();
    paint(&ctx, &view, &mut buf);
    let white = (0..50).find(|&x| cell(&alone, x, 12).bg == c("panel_bg"));
    let x = white.expect("a white tile on row 12");
    assert_eq!(cell(&buf, x, 12).bg, c("move_range"));
}

#[test]
fn a_battles_scene_darkens_the_battle_below_it() {
    let ctx = pd_ctx();
    let below = place_alone(&ctx, &small_place(&ctx));
    let mut buf = below.clone();
    let mut view = two_speakers(&["Hi."]);
    view.overlay = true;
    paint(&ctx, &view, &mut buf);
    let white = (0..10).find(|&y| cell(&below, 25, y).bg == c("panel_bg"));
    let y = white.expect("a white tile between the frames");
    assert_eq!(cell(&buf, 25, y).bg, c("move_range"));
    assert_eq!(cell(&buf, 25, y).glyph, cell(&below, 25, y).glyph);
}
