//! The glyph look of a tip box: paints a [`TipView`] as a double-line box
//! near the top of the map view, the guide's name over its top border, the
//! face (if any) on the left at twice the size of a 64×64 bust, and the
//! text to its right. Everything about the look lives here; nothing here
//! decides what a tip says.

use super::view::TipView;
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::screen::Ctx;
use crate::screens::battle::layout::MAP_VIEW;

/// The face's area, in cells: 128×128 console pixels, a 64×64 bust at 2×
/// (ADR-0043: the largest whole scale that fits).
pub const FACE_CELLS: (i32, i32) = (16, 8);

/// Cells between the border and what is inside, and between the face and
/// the text.
const MARGIN: i32 = 2;

/// Where the box of `view` goes: near the top of the map view, centred on
/// it; a box wider than the map starts at the screen's left edge and
/// reaches over the side panel.
pub fn rect(view: &TipView) -> Rect {
    let cells = |n: usize| i32::try_from(n).unwrap_or(0);
    let widest = view
        .lines
        .iter()
        .chain([&view.heading, &view.close])
        .map(|l| cells(l.chars().count()))
        .chain([cells(view.title.chars().count()) + 2])
        .max()
        .unwrap_or(0);
    let face_w = if view.face.is_some() {
        FACE_CELLS.0 + MARGIN
    } else {
        0
    };
    let w = MARGIN + face_w + widest + MARGIN;
    // Border, blank row, the heading, the lines, blank row, the close
    // hint, border; at least as tall as the face and the border.
    let text_h = cells(view.lines.len()) + 6;
    let h = match view.face {
        Some(_) => text_h.max(FACE_CELLS.1 + 2),
        None => text_h,
    };
    // Centred on the map, the widest box (82 cells) ends at most 41
    // cells right of the map's middle: still on the console.
    let x = (MAP_VIEW.x + (MAP_VIEW.w - w) / 2).max(0);
    Rect::new(x, MAP_VIEW.y + 1, w, h)
}

/// Paints `view`'s box.
pub fn paint(ctx: &Ctx, view: &TipView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let rect = rect(view);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    buf.print(
        rect.x + MARGIN,
        rect.y,
        &format!(" {} ", view.title),
        c(UiColor::TextHighlight),
        bg,
    );
    let mut x = rect.x + MARGIN;
    if let Some(face) = &view.face
        && let Some(portrait) = ctx.content.portraits.get(&face.portrait)
    {
        let at = (x, rect.y + 1);
        draw_portrait_in(buf, at, FACE_CELLS, portrait, &face.expression, 0.0, false);
    }
    if view.face.is_some() {
        x += FACE_CELLS.0 + MARGIN;
    }
    let mut y = rect.y + 2;
    buf.print(x, y, &view.heading, c(UiColor::TextHighlight), bg);
    for line in &view.lines {
        y += 1;
        buf.print(x, y, line, c(UiColor::Text), bg);
    }
    buf.print(x, rect.y + rect.h - 2, &view.close, c(UiColor::TextDim), bg);
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
