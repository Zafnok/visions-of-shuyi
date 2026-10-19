//! The Playdate's look of the lead screen (ticket 1045): the two
//! portraits side by side at 2× (`docs/design/playdate.md`: the bust at
//! 2×, 128 × 128 pixels), the chosen one in a double frame and the other
//! faded in a single one; the name and `[ Start ]` under them; the help on
//! the bottom row. The focus is the bar of the one-bit look
//! ([`crate::screens::playdate`]): the chosen gender's name while the
//! cursor is on the portraits, the first name, or Start. The letter grid
//! (or the typing box) opens in a window over the screen.

use super::Row;
use super::view::{GenderView, LeadSelectView};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{bar, cells, centred, clear, inks};
use crate::widgets::name_entry::playdate::{grid_height, paint_grid, paint_typing};

/// Row of the heading.
const HEADING_ROW: i32 = 0;
/// A portrait's frame: the 2× bust (16 × 8 cells) and its border.
const FRAME: (i32, i32) = (18, 10);
/// Top row of the portraits' frames.
const FRAME_Y: i32 = 1;
/// Left column of the male and female frames: centred, four cells apart.
const FRAME_X: [i32; 2] = [5, 27];
/// Row of the genders' names under the frames.
const LABEL_ROW: i32 = 11;
/// Row of the name.
const NAME_ROW: i32 = 12;
/// Row of the Start button.
const START_ROW: i32 = 13;
/// Left column and width of the letter grid's and the typing box's
/// windows.
const BOX_X: (i32, i32) = (2, 46);
/// The typing box's top row and height.
const TYPE_BOX: (i32, i32) = (4, 7);
/// How much the portrait not chosen fades (0 = all of it, 1 = gone): on
/// one bit, a share of its pixels left out (the PC's).
const DIM_NOT_CHOSEN: f32 = 0.45;

/// Paints the lead screen over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &LeadSelectView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    centred(ctx, buf, HEADING_ROW, &view.heading);
    let on_genders = view.cursor == Some(Row::Gender);
    for (gender, x) in view.genders.iter().zip(FRAME_X) {
        paint_gender(ctx, buf, gender, on_genders, x);
    }
    paint_rows(ctx, view, buf);
    let height = i32::from(buf.height());
    if let Some(grid) = &view.grid {
        let y = (height - grid_height(grid)) / 2;
        paint_grid(ctx, buf, Rect::new(BOX_X.0, y.max(0), BOX_X.1, 0), grid);
    }
    if let Some(typing) = &view.typing {
        let rect = Rect::new(BOX_X.0, TYPE_BOX.0, BOX_X.1, TYPE_BOX.1);
        paint_typing(ctx, buf, rect, typing);
    }
    centred(ctx, buf, height - 1, &view.help);
}

/// One gender's frame, portrait and name at column `x`: the chosen one in
/// a double frame, its name the bar while the cursor is on the portraits
/// (`focused`); the other in a single frame, faded.
fn paint_gender(ctx: &Ctx, buf: &mut GlyphBuffer, gender: &GenderView, focused: bool, x: i32) {
    let ink = inks(ctx);
    let style = if gender.chosen {
        BoxStyle::Double
    } else {
        BoxStyle::Single
    };
    let frame = Rect::new(x, FRAME_Y, FRAME.0, FRAME.1);
    buf.fill_rect(frame, Cell::new(' ', ink.text, ink.inside));
    buf.draw_box(frame, style, ink.frame, ink.inside);
    if let Some(art) = ctx.game.content.portraits.get(gender.portrait) {
        let dim = if gender.chosen { 0.0 } else { DIM_NOT_CHOSEN };
        let inside = (FRAME.0 - 2, FRAME.1 - 2);
        draw_portrait_in(
            buf,
            (x + 1, FRAME_Y + 1),
            inside,
            art,
            gender.expression,
            dim,
            false,
        );
    }
    let label_x = x + (FRAME.0 - cells(&gender.label)) / 2;
    if gender.chosen && focused {
        bar(
            ctx,
            buf,
            label_x - 1,
            LABEL_ROW,
            cells(&gender.label) + 2,
            &gender.label,
        );
    } else {
        buf.print(label_x, LABEL_ROW, &gender.label, ink.text, ink.ground);
    }
}

/// The name row and the Start button, the one with the cursor the bar.
fn paint_rows(ctx: &Ctx, view: &LeadSelectView, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let name = &view.name_row;
    let full = format!("{}   {} {}", name.label, name.first, name.family);
    let x = (i32::from(buf.width()) - cells(&full)) / 2;
    buf.print(x, NAME_ROW, &name.label, ink.text, ink.ground);
    let first_x = x + cells(&name.label) + 3;
    if view.cursor == Some(Row::Name) {
        bar(
            ctx,
            buf,
            first_x - 1,
            NAME_ROW,
            cells(&name.first) + 2,
            &name.first,
        );
    } else {
        buf.print(first_x, NAME_ROW, &name.first, ink.text, ink.ground);
    }
    let family_x = first_x + cells(&name.first) + 1;
    buf.print(family_x, NAME_ROW, &name.family, ink.text, ink.ground);
    let start = format!("[ {} ]", view.start);
    let x = (i32::from(buf.width()) - cells(&start)) / 2;
    if view.cursor == Some(Row::Start) {
        bar(ctx, buf, x - 1, START_ROW, cells(&start) + 2, &start);
    } else {
        buf.print(x, START_ROW, &start, ink.text, ink.ground);
    }
}

#[cfg(test)]
mod tests;
