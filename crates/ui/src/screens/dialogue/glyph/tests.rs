//! Tests of the dialogue screen's glyph look: where the portraits, names,
//! text, replies and the skip question go, the colours, and how the text
//! box grows. What the screen shows is tested on its view in
//! `../tests.rs`.

use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::Sprite;
use crate::portrait::dim_opacity;
use crate::screen::tests::ctx;
use crate::screens::dialogue::log::LogView;
use crate::screens::dialogue::view::{OpeningView, SkipView, TextView, WaitView};
use crate::widgets::menu::{MenuItemView, MenuView};

/// The text box `text` is painted in, and its first text row.
fn text_box(text: &TextView) -> (Rect, i32) {
    text_box::glyph::box_rect(BOX_PLACE, text)
}

/// Row of the name plates: the one under the portrait frames.
const PLATE_Y: i32 = 19;

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
    TextView {
        narration: false,
        shown: lines.iter().map(|l| l.chars().count()).sum(),
        lines,
        choices: None,
        skip: None,
        waiting: None,
    }
}

/// Lord on the left saying `Hi.` to the knight on the right.
fn two_speakers() -> DialogueView {
    DialogueView {
        overlay: false,
        place: None,
        opening: None,
        caption: None,
        left: Some(face("test_lord", "Test Lord")),
        right: Some(face("test_knight", "Test Knight")),
        speaker: Some(Side::Left),
        text: text(&["Hi."]),
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

/// `view` painted over a 100×32 screen of `x`s.
fn painted(view: &DialogueView) -> GlyphBuffer {
    let c = ctx();
    let p = &c.palette;
    let mut buf = GlyphBuffer::new(
        100,
        32,
        Cell::new('x', p.get(UiColor::Text), p.get(UiColor::Black)),
    );
    paint(&c, view, &mut buf);
    buf
}

fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect()
}

fn glyph(buf: &GlyphBuffer, x: i32, y: i32) -> Option<char> {
    buf.get(x, y).map(|c| c.glyph)
}

#[test]
fn speaker_frame_is_double_and_bright() {
    let c = ctx();
    let mut view = two_speakers();
    let buf = painted(&view);
    assert_eq!(glyph(&buf, LEFT_X, FRAME_Y), Some('╔'));
    assert_eq!(glyph(&buf, RIGHT_X, FRAME_Y), Some('┌'));
    assert_eq!(
        buf.get(LEFT_X, FRAME_Y).map(|c| c.fg),
        Some(c.palette.get(UiColor::PanelBorderFocus))
    );
    assert!(row(&buf, PLATE_Y).contains("Test Lord"));
    assert!(row(&buf, PLATE_Y).contains("Test Knight"));
    // The speaker's name is on the text box's top border.
    assert!(row(&buf, TEXT_BOX.y).starts_with("┌── Test Lord ─"));
    // Now the knight speaks: the frames swap.
    view.speaker = Some(Side::Right);
    let buf = painted(&view);
    assert_eq!(glyph(&buf, LEFT_X, FRAME_Y), Some('┌'));
    assert_eq!(glyph(&buf, RIGHT_X, FRAME_Y), Some('╔'));
    assert!(row(&buf, TEXT_BOX.y).starts_with("┌── Test Knight ─"));
}

/// The sprite of the portrait in the frame at column `x`.
fn portrait_sprite(buf: &GlyphBuffer, x: i32) -> Option<Sprite> {
    let left = (x + 1) * i32::from(ConsoleSpec::PC.cell_w);
    let inside = |s: &Sprite| (left..left + 256).contains(&s.dest.x);
    buf.sprites().into_iter().find(inside)
}

#[test]
fn the_speaker_is_bright_and_the_listener_dimmed_and_mirrored() {
    let buf = painted(&two_speakers());
    assert_eq!(buf.sprites().len(), 2);
    // The placeholders are 32×32: 8× fills the 256×256 px inside the frame.
    let top = (FRAME_Y + 1) * i32::from(ConsoleSpec::PC.cell_h);
    let listener = portrait_sprite(&buf, RIGHT_X).unwrap();
    assert_eq!(listener.image.path(), "portraits/test_knight/neutral.png");
    assert_eq!(listener.dest, Rect::new((RIGHT_X + 1) * 8, top, 256, 256));
    assert_eq!(listener.clip, listener.dest);
    assert!(listener.flip_x);
    assert_eq!(listener.opacity, dim_opacity(LISTENER_DIM));
    assert!(listener.opacity < 255);
    let speaker = portrait_sprite(&buf, LEFT_X).unwrap();
    assert_eq!(speaker.image.path(), "portraits/test_lord/neutral.png");
    assert_eq!(speaker.dest, Rect::new((LEFT_X + 1) * 8, top, 256, 256));
    assert!(!speaker.flip_x);
    assert_eq!(speaker.opacity, 255);
}

#[test]
fn the_portrait_is_the_one_the_view_names() {
    let mut view = two_speakers();
    view.left = Some(face("lead_f", "Isolde"));
    let buf = painted(&view);
    assert!(row(&buf, PLATE_Y).contains("Isolde"));
    assert!(row(&buf, TEXT_BOX.y).starts_with("┌── Isolde ─"));
    let sprite = portrait_sprite(&buf, LEFT_X).unwrap();
    assert_eq!(sprite.image.path(), "portraits/lead_f/neutral.png");
    assert_eq!((sprite.opacity, sprite.flip_x), (255, false));
}

#[test]
fn narration_dims_both_and_centres_the_text() {
    let c = ctx();
    let view = DialogueView {
        speaker: None,
        text: TextView {
            narration: true,
            ..text(&["Rain."])
        },
        ..two_speakers()
    };
    let buf = painted(&view);
    assert_eq!(glyph(&buf, LEFT_X, FRAME_Y), Some('┌'));
    assert_eq!(glyph(&buf, RIGHT_X, FRAME_Y), Some('┌'));
    // No name on the border; one line, centred in the middle row.
    assert!(row(&buf, TEXT_BOX.y).starts_with("┌────"));
    let middle = row(&buf, TEXT_Y + 1);
    let x = middle.find("Rain.").map(|b| middle[..b].chars().count());
    assert_eq!(x, Some(47));
    assert_eq!(
        buf.get(47, TEXT_Y + 1).map(|c| c.fg),
        Some(c.palette.get(UiColor::TextDim))
    );
}

#[test]
fn two_lines_of_narration_start_at_the_top() {
    let (a, b) = ("a".repeat(60), "b".repeat(60));
    let view = DialogueView {
        speaker: None,
        text: TextView {
            narration: true,
            ..text(&[&a, &b])
        },
        ..two_speakers()
    };
    let buf = painted(&view);
    assert!(row(&buf, TEXT_Y).contains(&a));
    assert!(row(&buf, TEXT_Y + 1).contains(&b));
}

#[test]
fn only_the_revealed_text_is_shown() {
    let view = DialogueView {
        text: TextView {
            shown: 8,
            ..text(&["Hello", "there, knight."])
        },
        ..two_speakers()
    };
    let buf = painted(&view);
    assert!(row(&buf, TEXT_Y).starts_with("│   Hello  "));
    assert!(row(&buf, TEXT_Y + 1).starts_with("│   the  "));
}

#[test]
fn an_empty_side_draws_nothing() {
    let c = ctx();
    let view = DialogueView {
        right: None,
        ..two_speakers()
    };
    let buf = painted(&view);
    let black = c.palette.get(UiColor::Black);
    for y in 0..=PLATE_Y {
        for x in RIGHT_X..RIGHT_X + FRAME.0 {
            assert_eq!(buf.get(x, y).map(|c| (c.glyph, c.bg)), Some((' ', black)));
        }
    }
}

#[test]
fn the_overlay_leaves_the_rest_of_the_screen() {
    let view = DialogueView {
        overlay: true,
        ..two_speakers()
    };
    let buf = painted(&view);
    // Between the frames the screen below shows, darkened; below the text
    // box (the battle's key help) is blanked.
    let c = ctx();
    let below = c.palette.get(UiColor::Text).scale(PLACE_BRIGHTNESS);
    for (x, y) in [(50, 5), (50, 20)] {
        let cell = buf.get(x, y).map(|c| (c.glyph, c.fg));
        assert_eq!(cell, Some(('x', below)), "cell {x},{y}");
    }
    for y in 28..32 {
        assert_eq!(row(&buf, y).trim(), "", "row {y}");
    }
    let full = painted(&two_speakers());
    assert_eq!(glyph(&full, 50, 5), Some(' '));
    // The talk itself is at full brightness.
    let lit = c.palette.get(UiColor::PanelBorderFocus);
    assert_eq!(buf.get(LEFT_X, FRAME_Y).map(|c| c.fg), Some(lit));
    assert_eq!(portrait_sprite(&buf, LEFT_X).map(|s| s.opacity), Some(255));
}

/// Chapter 1's map centred on Harrowby, as the screen's place: the whole
/// map fits the glyph look's screen, in its middle.
fn harrowby(c: &Ctx) -> crate::map_view::MapScene {
    let place = crate::map_view::Place::of_map(&c.game.content.maps["ch01"]);
    let view = c.game.map_skin.view_tiles(Rect::new(0, 0, 100, 32));
    place.scene(trpg_core::Pos::new(3, 12), view)
}

#[test]
fn the_place_is_painted_by_the_map_skin_darkened_under_the_talk() {
    let c = ctx();
    assert_eq!(c.game.map_skin.name(), "glyph");
    let scene = harrowby(&c);
    let view = DialogueView {
        place: Some(scene.clone()),
        caption: Some("The green".to_owned()),
        ..two_speakers()
    };
    let buf = painted(&view);
    // The place, as the map skin paints it, then darkened: wherever the
    // talk leaves it showing (between the frames, around the box).
    let black = c.palette.get(UiColor::Black);
    let empty = Cell::new(' ', c.palette.get(UiColor::Text), black);
    let mut place = GlyphBuffer::new(100, 32, empty);
    c.game
        .map_skin
        .paint(&c, &scene, place.bounds(), &mut place);
    place.fade(PLACE_BRIGHTNESS);
    for (x, y) in [(40, 10), (50, 20), (60, 3), (36, 16)] {
        assert_eq!(buf.get(x, y), place.get(x, y), "cell {x},{y}");
        assert_ne!(glyph(&buf, x, y), Some('x'), "cell {x},{y}");
    }
    // A map tile there, not the black around the map.
    assert_ne!(
        buf.get(40, 10).map(|c| c.bg),
        Some(black.scale(PLACE_BRIGHTNESS))
    );
    // The place covers the whole screen, below the text box too.
    assert_eq!(buf.get(50, 31), place.get(50, 31));
    // Portraits, plates, box and caption go on top at full brightness.
    let lit = c.palette.get(UiColor::PanelBorderFocus);
    assert_eq!(buf.get(LEFT_X, FRAME_Y).map(|c| c.fg), Some(lit));
    assert_eq!(portrait_sprite(&buf, LEFT_X).map(|s| s.opacity), Some(255));
    assert!(row(&buf, TEXT_BOX.y).starts_with("┌── Test Lord ─"));
    assert!(row(&buf, 0).contains(" The green "));
    let caption_x = row(&buf, 0).find("The green").unwrap_or(0);
    let caption_fg = buf
        .get(i32::try_from(caption_x).unwrap_or(0), 0)
        .map(|c| c.fg);
    assert_eq!(caption_fg, Some(c.palette.get(UiColor::TextHighlight)));
    // An overlay with a place shows the place, not the screen below.
    let over = painted(&DialogueView {
        overlay: true,
        ..view
    });
    assert_eq!(over.get(40, 10), place.get(40, 10));
    assert_eq!(over.get(50, 31), place.get(50, 31));
}

/// While an opening plays (0734) the place shows bright under the caption
/// alone, and darkens to the talk's brightness as it ends.
#[test]
fn an_opening_shows_the_place_bright_then_darkening_without_the_talk() {
    let c = ctx();
    let scene = harrowby(&c);
    let at = |darkened| {
        painted(&DialogueView {
            place: Some(scene.clone()),
            opening: Some(OpeningView { darkened }),
            caption: Some("The green".to_owned()),
            ..two_speakers()
        })
    };
    let black = c.palette.get(UiColor::Black);
    let mut place = GlyphBuffer::new(100, 32, Cell::new(' ', c.palette.get(UiColor::Text), black));
    c.game
        .map_skin
        .paint(&c, &scene, place.bounds(), &mut place);
    let bright = at(0.0);
    // The place as the map skin paints it, where the frames and the text
    // box would be too: no portraits, plates or box.
    for (x, y) in [
        (40, 10),
        (LEFT_X + 5, FRAME_Y + 5),
        (50, TEXT_BOX.y),
        (80, 25),
    ] {
        assert_eq!(bright.get(x, y), place.get(x, y), "cell {x},{y}");
    }
    assert!(portrait_sprite(&bright, LEFT_X).is_none());
    assert!(row(&bright, 0).contains(" The green "));
    // Half way through darkening, then as dark as behind the talk.
    let mut half = place.clone();
    half.fade(1.0 - (1.0 - PLACE_BRIGHTNESS) / 2.0);
    assert_eq!(at(0.5).get(40, 10), half.get(40, 10));
    place.fade(PLACE_BRIGHTNESS);
    assert_eq!(at(1.0).get(40, 10), place.get(40, 10));
}

#[test]
fn a_waiting_page_shows_its_key_and_the_blinking_arrow() {
    let arrow_row = TEXT_BOX.y + TEXT_BOX.h - 2;
    assert_eq!(
        row(&painted(&two_speakers()), arrow_row).trim_matches(['│', ' ']),
        ""
    );
    let mut view = two_speakers();
    view.text.waiting = Some(WaitView {
        key: "f".to_owned(),
        log_hint: None,
        marker: true,
    });
    assert!(row(&painted(&view), arrow_row).ends_with(" f ▼  │"));
    // The arrow out, the key stays.
    if let Some(waiting) = &mut view.text.waiting {
        waiting.marker = false;
    }
    assert!(row(&painted(&view), arrow_row).ends_with(" f    │"));
    // The log's hint, three cells before the key.
    if let Some(waiting) = &mut view.text.waiting {
        waiting.log_hint = Some("b log".to_owned());
    }
    assert!(row(&painted(&view), arrow_row).ends_with(" b log   f    │"));
}

#[test]
fn the_caption_is_top_centre() {
    let view = DialogueView {
        caption: Some("Heth".to_owned()),
        ..two_speakers()
    };
    let buf = painted(&view);
    assert_eq!(row(&buf, 0).trim(), "Heth");
    assert_eq!(row(&buf, 0).find("Heth"), Some(48));
}

#[test]
fn a_character_without_a_portrait_gets_an_empty_frame() {
    let view = DialogueView {
        left: None,
        right: Some(face("test_archer", "Test Archer")),
        speaker: Some(Side::Right),
        ..two_speakers()
    };
    let buf = painted(&view);
    assert_eq!(glyph(&buf, RIGHT_X, FRAME_Y), Some('╔'));
    let inside: String = row(&buf, FRAME_Y + 8).chars().skip(66).take(32).collect();
    assert_eq!(inside.trim(), "");
    assert!(row(&buf, PLATE_Y).contains("Test Archer"));
}

#[test]
fn the_skip_question_replaces_the_text() {
    let c = ctx();
    let mut view = two_speakers();
    view.text.skip = Some(SkipView {
        question: "Skip scene?".to_owned(),
        answers: "f yes / d no".to_owned(),
    });
    let buf = painted(&view);
    assert!(row(&buf, TEXT_Y).starts_with("│   Skip scene?  "));
    assert!(row(&buf, TEXT_Y + 1).starts_with("│   f yes / d no  "));
    assert!(!row(&buf, TEXT_Y).contains("Hi."));
    assert_eq!(
        buf.get(TEXT_X, TEXT_Y + 1).map(|c| c.fg),
        Some(c.palette.get(UiColor::TextDim))
    );
    // The speaker's name stays on the border.
    assert!(row(&buf, TEXT_BOX.y).starts_with("┌── Test Lord ─"));
}

#[test]
fn a_choice_lists_its_replies_under_the_question() {
    let c = ctx();
    let mut view = DialogueView {
        speaker: Some(Side::Right),
        ..two_speakers()
    };
    view.text = TextView {
        choices: Some(replies(&["Yes.", "He knows."], 0)),
        ..text(&["Ready?"])
    };
    // The question, a blank row, then the replies, the focused one
    // marked; this fits the usual box, so it doesn't grow.
    assert_eq!(text_box(&view.text), (TEXT_BOX, TEXT_Y));
    let buf = painted(&view);
    assert!(row(&buf, TEXT_Y).starts_with("│   Ready?  "));
    assert_eq!(row(&buf, TEXT_Y + 1).trim_matches(['│', ' ']), "");
    assert!(row(&buf, TEXT_Y + 2).starts_with("│   > Yes.  "));
    assert!(row(&buf, TEXT_Y + 3).starts_with("│     He knows.  "));
    assert_eq!(
        buf.get(TEXT_X, TEXT_Y + 2).map(|c| c.fg),
        Some(c.palette.get(UiColor::TextHighlight))
    );
    assert_eq!(
        buf.get(TEXT_X, TEXT_Y + 3).map(|c| c.fg),
        Some(c.palette.get(UiColor::Text))
    );
    // The portraits' name plates stay clear.
    assert!(row(&buf, PLATE_Y).contains("Test Lord"));
    // The focus marker follows the view's focus.
    view.text.choices = Some(replies(&["Yes.", "He knows."], 1));
    let buf = painted(&view);
    assert!(row(&buf, TEXT_Y + 2).starts_with("│     Yes.  "));
    assert!(row(&buf, TEXT_Y + 3).starts_with("│   > He knows.  "));
}

/// A long question and three replies: the box grows upward, its bottom
/// staying put.
#[test]
fn the_text_box_grows_to_fit_the_replies() {
    let line = "word ".repeat(18);
    let mut view = DialogueView {
        speaker: Some(Side::Right),
        ..two_speakers()
    };
    view.text = TextView {
        choices: Some(replies(&["A.", "B.", "C."], 0)),
        ..text(&[line.trim(), line.trim(), line.trim()])
    };
    // 3 question rows, a blank one and 3 replies: 7 rows from row 20.
    let (rect, text_y) = text_box(&view.text);
    assert_eq!(text_y, 20);
    assert_eq!(rect, Rect::new(0, 18, 100, 10));
    let buf = painted(&view);
    assert!(row(&buf, 18).starts_with("┌── Test Knight ─"));
    assert!(row(&buf, 20).starts_with("│   word word"));
    assert!(row(&buf, 26).starts_with("│     C.  "));
    assert!(row(&buf, 27).starts_with("└──"));
}

/// Replies with no line before them get no blank row above them.
#[test]
fn replies_without_a_question_start_at_the_top() {
    let mut view = two_speakers();
    view.text = TextView {
        choices: Some(replies(&["A.", "B.", "C.", "D."], 0)),
        ..text(&[])
    };
    assert_eq!(text_box(&view.text), (TEXT_BOX, TEXT_Y));
    let buf = painted(&view);
    assert!(row(&buf, TEXT_Y).starts_with("│   > A.  "));
    assert!(row(&buf, TEXT_Y + 3).starts_with("│     D.  "));
}

/// The open log is painted instead of the scene, as its own skin paints
/// it.
#[test]
fn the_open_log_covers_the_scene() {
    let c = ctx();
    let log = LogView {
        title: "Log".to_owned(),
        rows: vec![],
        back_to_latest: None,
        help: "help keys".to_owned(),
    };
    let view = DialogueView {
        log: Some(log.clone()),
        ..two_speakers()
    };
    let got = painted(&view);
    let mut expected = painted(&two_speakers());
    log::glyph::paint(&c, &log, &mut expected);
    for y in 0..32 {
        assert_eq!(row(&got, y), row(&expected, y), "row {y}");
    }
    assert!(got.sprites().is_empty());
}
