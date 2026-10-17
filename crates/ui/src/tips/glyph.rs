//! The glyph look of a tip box: paints a [`TipView`] as a double-line box
//! near the top of the map view, the guide's name over its top border, the
//! face (if any) on the left at twice the size of a 64×64 bust, and the
//! text to its right. A lesson's box ([`LessonView`], 0456) is the same
//! box against the map's top or bottom edge, its checklist under the text:
//! `[ ]` before an item to do, `[x]` in the done colour (`hp_high`) with
//! the label dimmed before one done, then how many are done. Everything
//! about the look lives here; nothing here decides what a tip or a lesson
//! says.
//!
//! The box stays over the map view (0459): text with a line too long for
//! the room beside the face is wrapped again at its spaces, and so is an
//! item, its further rows under its label. Without the face every line of
//! the formats fits.

use super::view::{FaceView, LessonView, TipView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::MAP_VIEW;
use crate::widgets::wrap::word_wrap;

/// The face's area, in cells: 128×128 console pixels, a 64×64 bust at 2×
/// (ADR-0043: the largest whole scale that fits).
pub const FACE_CELLS: (i32, i32) = (16, 8);

/// Cells between the border and what is inside, and between the face and
/// the text.
const MARGIN: i32 = 2;

/// The mark before an item to do, and before one done: glyphs of the box,
/// like its borders, not text.
const MARKS: [[char; 3]; 2] = [['[', ' ', ']'], ['[', 'x', ']']];

/// Cells across the mark before an item, with the gap after it.
const MARK_CELLS: i32 = 4;

/// The widest text a box with `face` or none holds without reaching past
/// the map view: its width less the margins and the face's place.
fn text_room(face: Option<&FaceView>) -> usize {
    let face_w = if face.is_some() {
        FACE_CELLS.0 + MARGIN
    } else {
        0
    };
    usize::try_from(MAP_VIEW.w - MARGIN - face_w - MARGIN).unwrap_or(1)
}

/// `lines` fitted to `room` cells. A paragraph (lines between blank ones)
/// whose lines all fit stays as written; one with a longer line is joined
/// and wrapped at its spaces, so no word is left alone on a row.
fn wrapped(lines: &[String], room: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for (i, paragraph) in lines.split(|l| l.trim().is_empty()).enumerate() {
        if i > 0 {
            rows.push(String::new());
        }
        if paragraph.iter().all(|l| l.chars().count() <= room) {
            rows.extend_from_slice(paragraph);
        } else {
            rows.extend(word_wrap(&paragraph.join(" "), room));
        }
    }
    rows
}

/// A count of cells as an `i32`.
fn cells(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(0)
}

/// The size of a box with `title` over its border, `face` or none, text
/// `widest` cells wide beside it, and `rows` rows of text between its
/// borders: at least as tall as the face and the borders.
fn size(title: &str, face: Option<&FaceView>, widest: i32, rows: usize) -> (i32, i32) {
    let widest = widest.max(cells(title.chars().count()) + 2);
    let face_w = if face.is_some() {
        FACE_CELLS.0 + MARGIN
    } else {
        0
    };
    let w = MARGIN + face_w + widest + MARGIN;
    let text_h = cells(rows) + 2;
    let h = match face {
        Some(_) => text_h.max(FACE_CELLS.1 + 2),
        None => text_h,
    };
    (w, h)
}

/// The widest of `texts`, in cells.
fn widest<'a>(texts: impl Iterator<Item = &'a String>) -> i32 {
    texts.map(|t| cells(t.chars().count())).max().unwrap_or(0)
}

/// The column a box `w` cells wide starts at: centred on the map view.
/// Its text wraps to fit the map (`text_room`), so only a title or a hint
/// longer than any the formats allow could make it wider; it would start
/// at the screen's left edge.
fn left(w: i32) -> i32 {
    (MAP_VIEW.x + (MAP_VIEW.w - w) / 2).max(0)
}

/// The rows of `view`'s text: its lines, wrapped to fit beside the face.
fn tip_lines(view: &TipView) -> Vec<String> {
    wrapped(&view.lines, text_room(view.face.as_ref()))
}

/// The rows of lesson `view`'s text and of each of its items, wrapped to
/// fit beside the face (an item's after its mark).
fn lesson_rows(view: &LessonView) -> (Vec<String>, Vec<Vec<String>>) {
    let room = text_room(view.face.as_ref());
    let item_room = room.saturating_sub(usize::try_from(MARK_CELLS).unwrap_or(0));
    let items = view
        .items
        .iter()
        .map(|i| wrapped(core::slice::from_ref(&i.label), item_room))
        .collect();
    (wrapped(&view.lines, room), items)
}

/// Where the box of `view` goes: near the top of the map view, centred on
/// it (`left`).
pub fn rect(view: &TipView) -> Rect {
    let lines = tip_lines(view);
    let wide = widest(lines.iter().chain([&view.heading, &view.close]));
    // A blank row, the heading, the lines, a blank row, the close hint.
    let rows = lines.len() + 4;
    let (w, h) = size(&view.title, view.face.as_ref(), wide, rows);
    Rect::new(left(w), MAP_VIEW.y + 1, w, h)
}

/// Where the box of lesson `view` goes: against the top edge of the map
/// view, or its bottom edge if `at_bottom`, centred on it (`left`).
pub fn lesson_rect(view: &LessonView, at_bottom: bool) -> Rect {
    let (lines, items) = lesson_rows(view);
    let text = widest(lines.iter().chain([&view.heading, &view.progress]));
    let item_rows = items.iter().map(Vec::len).sum::<usize>();
    let items = MARK_CELLS + widest(items.iter().flatten());
    // A blank row, the heading, the lines, a blank row, the items, a blank
    // row, the count.
    let rows = lines.len() + item_rows + 5;
    let (w, h) = size(&view.title, view.face.as_ref(), text.max(items), rows);
    let y = if at_bottom {
        MAP_VIEW.y + MAP_VIEW.h - h
    } else {
        MAP_VIEW.y
    };
    Rect::new(left(w), y, w, h)
}

/// Paints a box at `rect` with `title` over its top border and `face` (if
/// the game has its portrait) at its top left. Returns the column its text
/// starts at: right of the face's place, if it has one.
fn paint_frame(
    ctx: &Ctx,
    rect: Rect,
    title: &str,
    face: Option<&FaceView>,
    buf: &mut GlyphBuffer,
) -> i32 {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    buf.print(
        rect.x + MARGIN,
        rect.y,
        &format!(" {title} "),
        c(UiColor::TextHighlight),
        bg,
    );
    let x = rect.x + MARGIN;
    let Some(face) = face else {
        return x;
    };
    if let Some(portrait) = ctx.content.portraits.get(&face.portrait) {
        let at = (x, rect.y + 1);
        draw_portrait_in(buf, at, FACE_CELLS, portrait, &face.expression, 0.0, false);
    }
    x + FACE_CELLS.0 + MARGIN
}

/// Paints `heading` highlighted at `(x, y)`, then `lines` under it.
/// Returns the row of the last.
fn paint_text(
    ctx: &Ctx,
    (x, mut y): (i32, i32),
    heading: &str,
    lines: &[String],
    buf: &mut GlyphBuffer,
) -> i32 {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.print(x, y, heading, c(UiColor::TextHighlight), bg);
    for line in lines {
        y += 1;
        buf.print(x, y, line, c(UiColor::Text), bg);
    }
    y
}

/// Paints `view`'s box.
pub fn paint(ctx: &Ctx, view: &TipView, buf: &mut GlyphBuffer) {
    let rect = rect(view);
    let x = paint_frame(ctx, rect, &view.title, view.face.as_ref(), buf);
    paint_text(ctx, (x, rect.y + 2), &view.heading, &tip_lines(view), buf);
    let (dim, bg) = (
        ctx.palette.get(UiColor::TextDim),
        ctx.palette.get(UiColor::PanelBg),
    );
    buf.print(x, rect.y + rect.h - 2, &view.close, dim, bg);
}

/// Paints lesson `view`'s box, against the map view's bottom edge if
/// `at_bottom`, else its top edge.
pub fn paint_lesson(ctx: &Ctx, view: &LessonView, at_bottom: bool, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let rect = lesson_rect(view, at_bottom);
    let x = paint_frame(ctx, rect, &view.title, view.face.as_ref(), buf);
    let (lines, item_rows) = lesson_rows(view);
    // The checklist starts after a blank row.
    let mut y = paint_text(ctx, (x, rect.y + 2), &view.heading, &lines, buf) + 1;
    for (item, rows) in view.items.iter().zip(&item_rows) {
        let (mark, label) = if item.done {
            (c(UiColor::HpHigh), c(UiColor::TextDim))
        } else {
            (c(UiColor::Text), c(UiColor::Text))
        };
        let glyphs: String = MARKS[usize::from(item.done)].iter().collect();
        buf.print(x, y + 1, &glyphs, mark, bg);
        for row in rows {
            y += 1;
            buf.print(x + MARK_CELLS, y, row, label, bg);
        }
    }
    buf.print(
        x,
        rect.y + rect.h - 2,
        &view.progress,
        c(UiColor::TextDim),
        bg,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rgb;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::screen::tests::ctx;
    use crate::tips::view::FaceView;

    fn view(face: bool, title: &str, lines: &[&str]) -> TipView {
        TipView {
            title: title.into(),
            face: face.then(|| FaceView {
                portrait: "guide".into(),
                expression: "smile".into(),
            }),
            heading: "Head".into(),
            lines: lines.iter().map(|&l| l.to_owned()).collect(),
            close: "x".into(),
        }
    }

    fn painted(c: &Ctx, view: &TipView) -> GlyphBuffer {
        let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
        paint(c, view, &mut buf);
        buf
    }

    /// A context whose guide is the test lord's portrait, smiling.
    fn with_guide() -> Ctx {
        let mut c = ctx();
        let mut guide = c.content.portraits["test_lord"].clone();
        let smile = guide.expressions[1].clone();
        guide.expressions.push(trpg_content::portrait::Expression {
            name: "smile".into(),
            ..smile
        });
        c.content.portraits.insert("guide".into(), guide);
        c
    }

    #[test]
    fn the_box_is_as_wide_as_its_widest_line_or_title() {
        let cx = ctx();
        // A title of 20 cells gives 22 inside the borders, plus 2 margins.
        let buf = painted(&cx, &view(false, &"T".repeat(20), &["a", "bb"]));
        let (w, h) = (26, 2 + 6);
        let x = MAP_VIEW.x + (MAP_VIEW.w - w) / 2;
        let y = MAP_VIEW.y + 1;
        let glyph = |x, y| buf.get(x, y).map(|c| c.glyph);
        assert_eq!(glyph(x, y), Some('╔'));
        assert_eq!(glyph(x + w - 1, y), Some('╗'));
        assert_eq!(glyph(x + w, y), Some(' '));
        assert_eq!(glyph(x, y + h - 1), Some('╚'));
        assert_eq!(glyph(x + 2, y + 2), Some('H'));
        assert_eq!(glyph(x + 2, y + 3), Some('a'));
        assert_eq!(glyph(x + 3, y + 4), Some('b'));
        assert_eq!(glyph(x + 2, y + 6), Some('x'));
        // The heading is highlighted, the lines are not.
        let fg = |x, y| buf.get(x, y).map(|c| c.fg);
        assert_eq!(
            fg(x + 2, y + 2),
            Some(cx.palette.get(UiColor::TextHighlight))
        );
        assert_eq!(fg(x + 2, y + 3), Some(cx.palette.get(UiColor::Text)));
        assert!(buf.sprites().is_empty());
    }

    /// A lesson's box with `lines` lines of text and `items` items.
    fn lesson(lines: usize, items: usize) -> LessonView {
        LessonView {
            title: "Guide".into(),
            face: None,
            heading: "Head".into(),
            lines: vec!["text".to_owned(); lines],
            items: (0..items)
                .map(|i| super::super::view::LessonItemView {
                    label: "x".repeat(i + 1),
                    done: i % 2 == 0,
                })
                .collect(),
            progress: "1 of 3 done".into(),
            clear_of: trpg_core::Pos::new(0, 0),
        }
    }

    /// Ticket 0456: the lesson's box is its border, a blank row, the
    /// heading, the text, a blank row, the items, a blank row and the
    /// count; against the map's top edge, or its bottom one.
    #[test]
    fn the_lesson_box_holds_its_rows_at_the_top_or_the_bottom() {
        for (lines, items) in [(1, 3), (2, 1), (2, 6), (1, 1)] {
            let top = lesson_rect(&lesson(lines, items), false);
            let h = i32::try_from(lines + items + 7).unwrap();
            assert_eq!((top.y, top.h), (MAP_VIEW.y, h), "{lines} {items}");
            let bottom = lesson_rect(&lesson(lines, items), true);
            assert_eq!(bottom.y + bottom.h, MAP_VIEW.y + MAP_VIEW.h);
            assert_eq!((bottom.x, bottom.w, bottom.h), (top.x, top.w, top.h));
        }
        // An item as the widest line: its mark and its label, in the margins.
        let mut wide = lesson(1, 1);
        wide.items[0].label = "l".repeat(30);
        assert_eq!(lesson_rect(&wide, false).w, 2 + 4 + 30 + 2);
        // Painted: the items under the text, the count above the border.
        let c = ctx();
        let v = lesson(1, 3);
        let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
        paint_lesson(&c, &v, false, &mut buf);
        let r = lesson_rect(&v, false);
        let row = |y: i32| -> String {
            (r.x + 2..r.x + r.w - 2)
                .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
                .collect::<String>()
                .trim_end()
                .to_owned()
        };
        let rows: Vec<String> = (r.y + 1..r.y + r.h - 1).map(row).collect();
        assert_eq!(
            rows,
            [
                "",
                "Head",
                "text",
                "",
                "[x] x",
                "[ ] xx",
                "[x] xxx",
                "",
                "1 of 3 done"
            ]
        );
        let fg = |x, y| buf.get(x, y).map(|c| c.fg);
        assert_eq!(fg(r.x + 3, r.y + 5), Some(c.palette.get(UiColor::HpHigh)));
        assert_eq!(fg(r.x + 6, r.y + 5), Some(c.palette.get(UiColor::TextDim)));
        assert_eq!(fg(r.x + 6, r.y + 6), Some(c.palette.get(UiColor::Text)));
    }

    #[test]
    fn the_face_is_on_the_left_at_128_pixels_and_the_text_beside_it() {
        let c = with_guide();
        let v = view(true, "Guide", &["a"]);
        let buf = painted(&c, &v);
        let r = rect(&v);
        // Face, margin, the widest line (the title " Guide "), and the
        // margins.
        assert_eq!((r.w, r.h), (16 + 2 + 7 + 4, 10));
        let sprites = buf.sprites();
        assert_eq!(sprites.len(), 1);
        let face = &sprites[0];
        assert_eq!(face.image.path(), "portraits/test_lord/happy.png");
        let (px, py) = (8, 16);
        assert_eq!(
            face.dest,
            Rect::new((r.x + 2) * px, (r.y + 1) * py, 128, 128)
        );
        let glyph = |x, y| buf.get(x, y).map(|c| c.glyph);
        assert_eq!(glyph(r.x + 20, r.y + 2), Some('H'));
        assert_eq!(glyph(r.x + 20, r.y + 3), Some('a'));
        assert_eq!(glyph(r.x + 20, r.y + r.h - 2), Some('x'));
        // Without the portrait the face's place stays empty.
        let buf = painted(&ctx(), &v);
        assert!(buf.sprites().is_empty());
    }

    /// Ticket 0459: beside the face a 60-character line wraps, and the box
    /// stays over the map view, clear of the side panel.
    #[test]
    fn sixty_character_lines_beside_the_face_stay_over_the_map() {
        let c = with_guide();
        let line = "word ".repeat(12).trim_end().to_owned();
        assert_eq!(line.len(), trpg_content::tip::MAX_LINE_CHARS - 1);
        let line = format!("{line}.");
        let v = view(true, "Guide", &[&line, "Short."]);
        let r = rect(&v);
        assert!(
            r.x >= MAP_VIEW.x && r.x + r.w <= MAP_VIEW.x + MAP_VIEW.w,
            "{r:?}"
        );
        assert!(r.y + r.h <= MAP_VIEW.y + MAP_VIEW.h, "{r:?}");
        // The text wrapped again: 48 cells of words, then the rest; as
        // tall as the face.
        assert_eq!(r.h, 2 + FACE_CELLS.1);
        let buf = painted(&c, &v);
        let row = |y: i32| -> String {
            (r.x + 20..r.x + r.w - 1)
                .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
                .collect::<String>()
                .trim_end()
                .to_owned()
        };
        assert_eq!(row(r.y + 3), "word ".repeat(9).trim_end());
        assert_eq!(row(r.y + 4), "word word word. Short.");
        assert_eq!(row(r.y + 5), "");
        // Without the face the same line fits on one row.
        let r = rect(&view(false, "Guide", &[&line]));
        assert_eq!(r.w, 2 + 60 + 2);
        assert_eq!(r.h, 2 + 5);
    }

    /// Text that fits stays as written; a paragraph with a longer line is
    /// wrapped again as one, the blank line between paragraphs kept.
    #[test]
    fn only_a_paragraph_with_a_long_line_is_wrapped_again() {
        let lines = |ls: &[&str]| ls.iter().map(|&l| l.to_owned()).collect::<Vec<_>>();
        let fits = lines(&["a  b", "c"]);
        assert_eq!(wrapped(&fits, 5), fits);
        assert_eq!(
            wrapped(&lines(&["aaa bbb", "ccc", "", "dd", "e"]), 5),
            lines(&["aaa", "bbb", "ccc", "", "dd", "e"])
        );
        assert_eq!(
            wrapped(&lines(&["aa bb cc dd"]), 5),
            lines(&["aa bb", "cc dd"])
        );
        assert!(wrapped(&[], 5).is_empty());
    }

    /// Ticket 0459: an item too long for the room beside the face goes on
    /// under its label, its mark only on the first row.
    #[test]
    fn a_long_item_beside_the_face_wraps_under_its_label() {
        let mut v = lesson(1, 1);
        v.face = Some(FaceView {
            portrait: "guide".into(),
            expression: "smile".into(),
        });
        v.items[0].label = format!("{} end", "item ".repeat(9).trim_end());
        let r = lesson_rect(&v, false);
        assert!(r.x + r.w <= MAP_VIEW.x + MAP_VIEW.w, "{r:?}");
        assert_eq!(r.h, 1 + 1 + 1 + 1 + 2 + 2 + 1 + 1);
        let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
        paint_lesson(&with_guide(), &v, false, &mut buf);
        let row = |y: i32| -> String {
            (r.x + 20..r.x + r.w - 1)
                .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
                .collect::<String>()
                .trim_end()
                .to_owned()
        };
        assert_eq!(
            row(r.y + 5),
            format!("[x] {}", "item ".repeat(9).trim_end())
        );
        assert_eq!(row(r.y + 6), "    end");
        assert_eq!(row(r.y + 8), "1 of 3 done");
    }

    /// Ticket 0459: every tip and lesson the game has, its keys named,
    /// stays over the map view beside the face.
    #[test]
    fn every_tip_and_lesson_fits_beside_the_face() {
        let c = with_guide();
        let face = || {
            Some(FaceView {
                portrait: "guide".into(),
                expression: "smile".into(),
            })
        };
        let inside = |r: Rect| {
            r.x >= MAP_VIEW.x
                && r.x + r.w <= MAP_VIEW.x + MAP_VIEW.w
                && r.y >= MAP_VIEW.y
                && r.y + r.h <= MAP_VIEW.y + MAP_VIEW.h
        };
        assert!(!c.content.tips.tips.is_empty());
        for tip in &c.content.tips.tips {
            let mut v = TipView::new(&c, tip);
            v.face = face();
            assert!(inside(rect(&v)), "{:?}: {:?}", tip.trigger, rect(&v));
        }
        assert!(!c.content.lessons.lessons.is_empty());
        for lesson in &c.content.lessons.lessons {
            let mut v = LessonView::new(&c, lesson, &[], trpg_core::Pos::new(0, 0));
            v.face = face();
            for at_bottom in [false, true] {
                let r = lesson_rect(&v, at_bottom);
                assert!(inside(r), "{}: {r:?}", lesson.id);
            }
        }
    }
}
