//! The chapter card on the PC: two rules across the middle of a black
//! screen, `Chapter 1` between them in Cinzel Medium in the text colour,
//! and the chapter's name under it in Cinzel Bold in the highlight colour
//! (Nick, 0812). Each Cinzel glyph is a sprite from its sheet, painted in
//! the line's colour ([`Paint::Solid`]); a line with a character the
//! sheet lacks is printed in the cell font in its row instead.

use crate::prelude::*;

use trpg_content::lettering::Sheet;

use super::ChapterCardView;
use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Paint, PxRect, Rect, Sprite};
use crate::screen::Ctx;
use crate::screens::print_centred;

/// The rules' rows, from the console's middle row: above and below.
pub const RULE_ROWS: (i32, i32) = (-5, 4);
/// The `Chapter 1` line's row, from the middle row.
pub const LINE_ROW: i32 = -3;
/// The name's row, from the middle row (its cell-font fallback's; the
/// Cinzel name is centred a little lower, [`NAME_DROP`]).
pub const NAME_ROW: i32 = 0;
/// Console pixels the Cinzel name's centre is below its row's top: the
/// name is taller than a row.
pub const NAME_DROP: i32 = 12;
/// Each half of a rule, in cells: `═` that many, then ` ◆ `, then again.
pub const RULE_HALF: usize = 30;

/// A rule: double lines broken by a small diamond.
pub fn rule(half: usize) -> String {
    let side = "═".repeat(half);
    format!("{side} ◆ {side}")
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ChapterCardView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    let mid = i32::from(buf.height()) / 2;
    let rule = rule(RULE_HALF);
    for dy in [RULE_ROWS.0, RULE_ROWS.1] {
        print_centred(buf, mid + dy, &rule, c(UiColor::PanelBorder), black);
    }
    let cell_h = i32::from(buf.spec().cell_h);
    let lettering = ctx.game.content.title.lettering.as_ref();
    let lines = [
        (&view.line, LINE_ROW, cell_h / 2, UiColor::Text),
        (&view.heading, NAME_ROW, NAME_DROP, UiColor::TextHighlight),
    ];
    for (i, (text, row, drop, color)) in lines.into_iter().enumerate() {
        let y = mid + row;
        let sheet = lettering.map(|l| if i == 0 { &l.line } else { &l.heading });
        let centre = (buf.pixel_bounds().w / 2, y * cell_h + drop);
        let drawn = sheet.is_some_and(|s| letter(buf, s, text, centre, c(color)));
        if !drawn {
            print_centred(buf, y, text, c(color), black);
        }
    }
}

/// The glyphs of `text` in `sheet`, each with the console pixels it
/// takes and its source rectangle; `None` if the sheet lacks one. A glyph
/// is its inked columns, `gap` pixels apart; a space is a quarter of a
/// cell.
pub fn layout(sheet: &Sheet, text: &str) -> Option<Vec<(char, PxRect)>> {
    let atlas = &sheet.atlas;
    let ink = atlas.ink.as_ref()?;
    let to_i32 = |v: u32| i32::try_from(v).unwrap_or(0);
    let gap = to_i32(atlas.cell_h) / 14 + 1;
    let mut x = 0;
    let mut glyphs = Vec::new();
    for ch in text.chars() {
        let cell = atlas.glyph_rect(ch)?;
        if ch == ' ' {
            x += to_i32(atlas.cell_w) / 4;
            continue;
        }
        let (left, right) = ink.columns_of(ch);
        let w = to_i32(right) - to_i32(left) + 1;
        glyphs.push((ch, Rect::new(x, 0, w, to_i32(cell.h))));
        x += w + gap;
    }
    Some(glyphs)
}

/// Draws `text` in `sheet`'s lettering, in `color`, its capitals centred
/// on console pixel `centre`. Returns `false`, drawing nothing, if the
/// sheet lacks a glyph of it.
fn letter(
    buf: &mut GlyphBuffer,
    sheet: &Sheet,
    text: &str,
    centre: (i32, i32),
    color: Rgb,
) -> bool {
    let atlas = &sheet.atlas;
    let (Some(glyphs), Some(ink)) = (layout(sheet, text), atlas.ink.as_ref()) else {
        return false;
    };
    let to_i32 = |v: u32| i32::try_from(v).unwrap_or(0);
    let width = glyphs.last().map_or(0, |(_, r)| r.x + r.w);
    let caps = to_i32(ink.capitals.top + ink.capitals.bottom) / 2;
    let (x0, y0) = (centre.0 - width / 2, centre.1 - caps);
    for (ch, at) in glyphs {
        let Some(cell) = atlas.glyph_rect(ch) else {
            continue;
        };
        let (left, _) = ink.columns_of(ch);
        let src = Rect::new(to_i32(cell.x + left), to_i32(cell.y), at.w, at.h);
        let dest = Rect::new(x0 + at.x, y0, at.w, at.h);
        let sprite = Sprite::new(sheet.image, src, dest, Layer::Over);
        buf.add_sprite(sprite.painted(Paint::Solid(color)));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::screen::tests::ctx;

    fn buffer(c: &Ctx) -> GlyphBuffer {
        let blank = Cell::new(
            ' ',
            c.palette.get(UiColor::Text),
            c.palette.get(UiColor::Black),
        );
        GlyphBuffer::with_spec(ConsoleSpec::PC, blank)
    }

    fn sheets() -> trpg_content::lettering::Lettering {
        ctx().game.content.title.lettering.clone().unwrap()
    }

    #[test]
    fn a_rule_is_two_double_lines_and_a_diamond() {
        assert_eq!(rule(2), "══ ◆ ══");
    }

    #[test]
    fn glyphs_are_laid_left_to_right_by_their_ink() {
        let name = sheets().heading;
        let glyphs = layout(&name, "IW I").unwrap();
        let chars: String = glyphs.iter().map(|(c, _)| c).collect();
        assert_eq!(chars, "IWI", "a space has no sprite");
        let (i, w) = (glyphs[0].1, glyphs[1].1);
        assert_eq!(i.x, 0);
        assert!(i.w < w.w, "an I is narrower than a W");
        assert!(w.x > i.x + i.w, "a gap between two glyphs");
        assert!(glyphs[2].1.x > w.x + w.w + 4, "a space between words");
        // A character the sheet lacks: no layout.
        assert_eq!(layout(&name, "Né"), None);
    }

    #[test]
    fn the_lines_are_sprites_in_their_colours_between_the_rules() {
        let c = ctx();
        let mut buf = buffer(&c);
        let view = ChapterCardView::of_title("Chapter 1: The Letter");
        paint(&c, &view, &mut buf);
        let sprites = buf.sprites();
        let count = |text: &str| text.chars().filter(|ch| *ch != ' ').count();
        assert_eq!(sprites.len(), count("Chapter 1") + count("The Letter"));
        let gold = Paint::Solid(c.palette.get(UiColor::TextHighlight));
        let text = Paint::Solid(c.palette.get(UiColor::Text));
        assert!(sprites[..8].iter().all(|s| s.paint == text));
        assert!(sprites[8..].iter().all(|s| s.paint == gold));
        let lettering = sheets();
        assert!(sprites[..8].iter().all(|s| s.image == lettering.line.image));
        assert!(
            sprites[8..]
                .iter()
                .all(|s| s.image == lettering.heading.image)
        );
        // Both lines are between the rules, the name below the line.
        let cell_h = i32::from(buf.spec().cell_h);
        let mid = i32::from(buf.height()) / 2;
        let (top, bottom) = (
            (mid + RULE_ROWS.0 + 1) * cell_h,
            (mid + RULE_ROWS.1) * cell_h,
        );
        assert!(
            sprites
                .iter()
                .all(|s| s.dest.y >= top && s.dest.y + s.dest.h <= bottom)
        );
        assert!(sprites[8].dest.y > sprites[0].dest.y);
        // Each line is centred across the screen.
        let middle = buf.pixel_bounds().w / 2;
        for line in [&sprites[..8], &sprites[8..]] {
            let (first, last) = (line[0].dest, line[line.len() - 1].dest);
            let centre = (first.x + last.x + last.w) / 2;
            assert!((centre - middle).abs() <= 1, "{centre} {middle}");
        }
    }

    #[test]
    fn a_line_the_sheet_cannot_draw_is_in_the_cell_font() {
        let c = ctx();
        let mut buf = buffer(&c);
        let view = ChapterCardView::of_title("Kapitel 1: Über");
        paint(&c, &view, &mut buf);
        assert_eq!(buf.sprites().len(), "Kapitel1".len());
        let mid = i32::from(buf.height()) / 2;
        let row: String = (0..i32::from(buf.width()))
            .map(|x| buf.get(x, mid + NAME_ROW).map_or(' ', |cell| cell.glyph))
            .collect();
        assert_eq!(row.trim(), "Über");
    }
}
