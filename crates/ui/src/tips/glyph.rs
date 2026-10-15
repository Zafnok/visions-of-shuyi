//! The glyph look of a tip box: paints a [`TipView`] as a double-line box
//! near the top of the map view, the guide's name over its top border, the
//! face (if any) on the left at twice the size of a 64×64 bust, and the
//! text to its right. A lesson's box ([`LessonView`], 0456) is the same
//! box against the map's top or bottom edge, its checklist under the text:
//! `[ ]` before an item to do, `[x]` in the done colour (`hp_high`) with
//! the label dimmed before one done, then how many are done. Everything
//! about the look lives here; nothing here decides what a tip or a lesson
//! says.

use super::view::{FaceView, LessonView, TipView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::MAP_VIEW;

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

/// The column a box `w` cells wide starts at: centred on the map view; a
/// box wider than the map starts at the screen's left edge and reaches
/// over the side panel. Centred on the map, the widest box (82 cells)
/// ends at most 41 cells right of the map's middle: still on the console.
fn left(w: i32) -> i32 {
    (MAP_VIEW.x + (MAP_VIEW.w - w) / 2).max(0)
}

/// Where the box of `view` goes: near the top of the map view, centred on
/// it (`left`).
pub fn rect(view: &TipView) -> Rect {
    let wide = widest(view.lines.iter().chain([&view.heading, &view.close]));
    // A blank row, the heading, the lines, a blank row, the close hint.
    let rows = view.lines.len() + 4;
    let (w, h) = size(&view.title, view.face.as_ref(), wide, rows);
    Rect::new(left(w), MAP_VIEW.y + 1, w, h)
}

/// Where the box of lesson `view` goes: against the top edge of the map
/// view, or its bottom edge if `at_bottom`, centred on it (`left`).
pub fn lesson_rect(view: &LessonView, at_bottom: bool) -> Rect {
    let text = widest(view.lines.iter().chain([&view.heading, &view.progress]));
    let items = MARK_CELLS + widest(view.items.iter().map(|i| &i.label));
    // A blank row, the heading, the lines, a blank row, the items, a blank
    // row, the count.
    let rows = view.lines.len() + view.items.len() + 5;
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
    paint_text(ctx, (x, rect.y + 2), &view.heading, &view.lines, buf);
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
    // The checklist starts after a blank row.
    let mut y = paint_text(ctx, (x, rect.y + 2), &view.heading, &view.lines, buf) + 1;
    for item in &view.items {
        y += 1;
        let (mark, label) = if item.done {
            (c(UiColor::HpHigh), c(UiColor::TextDim))
        } else {
            (c(UiColor::Text), c(UiColor::Text))
        };
        let glyphs: String = MARKS[usize::from(item.done)].iter().collect();
        buf.print(x, y, &glyphs, mark, bg);
        buf.print(x + MARK_CELLS, y, &item.label, label, bg);
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
    use crate::console::ConsoleSpec;
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

    #[test]
    fn sixty_character_lines_beside_the_face_stay_on_the_console() {
        let c = with_guide();
        let line = "x".repeat(trpg_content::tip::MAX_LINE_CHARS);
        let v = view(true, "Guide", &[&line, &line, &line]);
        let r = rect(&v);
        assert_eq!(r.w, 16 + 2 + 60 + 4);
        // Wider than the map: from the left edge, over the side panel.
        assert_eq!(r.x, 0);
        assert!(
            r.x >= 0 && r.x + r.w <= i32::from(ConsoleSpec::PC.cols),
            "{r:?}"
        );
        assert!(r.y + r.h <= MAP_VIEW.y + MAP_VIEW.h, "{r:?}");
        let buf = painted(&c, &v);
        let glyph = |x, y| buf.get(x, y).map(|c| c.glyph);
        assert_eq!(glyph(r.x + r.w - 3, r.y + 5), Some('x'));
        assert_eq!(glyph(r.x + r.w - 1, r.y + 5), Some('║'));
    }
}
