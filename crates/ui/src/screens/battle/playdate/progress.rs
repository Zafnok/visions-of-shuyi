//! The Playdate's look of the EXP bar, the level-up and the class pages
//! (ticket 1039): the EXP bar and the class box where the fight box was,
//! across the top of the screen; the level-up page a window in the middle,
//! the unit's face at 1× in its top-left corner (as the guide's box,
//! 0066), the banner, the name and what changed beside it, and the stat
//! rows under them, two to a row, each gain the bar (where the PC lights
//! it). Without a face, the text takes its place.

use crate::prelude::*;

use super::fight_box::box_rect;
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::screen::Ctx;
use crate::screens::battle::glyph::progress::{EXP_BAR_CELLS, bar_cells};
use crate::screens::battle::progress::stat_row_parts;
use crate::screens::battle::units::meter_cells;
use crate::screens::battle::view::{ClassProgressView, ExpView, GainsView, ProgressView};
use crate::screens::playdate::{self as pd, cells, cut, inks, to_i32};

/// Widest unit name in the EXP and class boxes.
const NAME_W: usize = 12;

/// Where the EXP bar or the class line starts, after the name.
const NAME_COL: i32 = 13;

/// The level-up window's width, in cells: two columns of stat rows.
pub const GAINS_W: i32 = 40;

/// The face's area on the level-up page, in cells: a 64 × 64 bust at 1×.
pub const FACE: (i32, i32) = (8, 4);

/// The stat rows the level-up page keeps room for, two to a row: every
/// stat that grows (HP, Str, Mag, Dex, Spd, Def, Res).
const STAT_ROWS: i32 = 4;

/// Where the second column of stat rows starts, from the text column.
const SECOND: i32 = 19;

/// Paints the page `view` over `area`.
pub fn paint(ctx: &Ctx, view: &ProgressView, area: Rect, buf: &mut GlyphBuffer) {
    match view {
        ProgressView::Exp(p) => paint_exp(ctx, p, area, buf),
        ProgressView::Gains(p) => paint_gains(ctx, p, area, buf),
        ProgressView::Class(p) => paint_class(ctx, p, area, buf),
    }
}

/// The EXP box: `Test Lord    EXP ████████░░░░ 80`.
fn paint_exp(ctx: &Ctx, page: &ExpView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let rect = box_rect(area, 3);
    pd::window(ctx, buf, rect, "");
    let (x, y) = (rect.x + 2, rect.y + 1);
    // check-text: not a data name (the view's own)
    buf.print(x, y, &cut(&page.name, NAME_W), ink.text, ink.inside);
    let bar_x = x + NAME_COL;
    buf.print(bar_x, y, &page.exp_name, ink.text, ink.inside);
    let bar = Rect::new(bar_x + 4, y, EXP_BAR_CELLS, 1);
    let full = Cell::new('█', ink.text, ink.inside);
    let empty = Cell::new('░', ink.text, ink.inside);
    meter_cells(buf, bar, bar_cells(page.exp, page.max), full, empty);
    let n = format!("{:>2}", page.exp);
    buf.print(bar_x + 5 + EXP_BAR_CELLS, y, &n, ink.text, ink.inside);
}

/// The class box: the name and the class line, then the mastery, and a
/// line each for what was learned or stowed.
fn paint_class(ctx: &Ctx, page: &ClassProgressView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let mastery = i32::from(page.mastered.is_some());
    let rect = box_rect(area, 3 + mastery + to_i32(page.lines.len()));
    pd::window(ctx, buf, rect, "");
    let x = rect.x + 2;
    let room = usize::try_from(rect.w - 4).unwrap_or(0);
    let mut y = rect.y + 1;
    // check-text: not a data name (the view's own)
    buf.print(x, y, &cut(&page.name, NAME_W), ink.text, ink.inside);
    let line = cut(&page.line, room.saturating_sub(13));
    buf.print(x + NAME_COL, y, &line, ink.text, ink.inside);
    // The box is as tall as its lines.
    for line in page.mastered.iter().chain(&page.lines) {
        y += 1;
        buf.print(x, y, &cut(line, room), ink.text, ink.inside);
    }
}

/// Where the level-up window goes over `area`: [`GAINS_W`] wide, in the
/// middle; the face's rows, a blank row and the stat rows inside the
/// frame.
pub fn gains_rect(area: Rect) -> Rect {
    let w = GAINS_W.min(area.w);
    let h = (FACE.1 + 1 + STAT_ROWS + 2).min(area.h);
    Rect::new(area.x + (area.w - w) / 2, area.y + (area.h - h) / 2, w, h)
}

/// The level-up page: the face (if any), the banner, the name, what
/// changed, and the stat rows shown so far.
fn paint_gains(ctx: &Ctx, page: &GainsView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let rect = gains_rect(area);
    pd::window(ctx, buf, rect, "");
    let face_at = (rect.x + 2, rect.y + 1);
    let drawn = page.face.as_ref().is_some_and(|face| {
        ctx.game
            .content
            .portraits
            .get(&face.portrait)
            .is_some_and(|art| {
                draw_portrait_in(buf, face_at, FACE, art, &face.expression, 0.0, false)
            })
    });
    let x = if drawn {
        face_at.0 + FACE.0 + 2
    } else {
        face_at.0
    };
    let room = usize::try_from(rect.x + rect.w - 2 - x).unwrap_or(0);
    let print = |buf: &mut GlyphBuffer, dy: i32, text: &str| {
        buf.print(x, rect.y + dy, &cut(text, room), ink.text, ink.inside);
    };
    print(buf, 1, &page.banner);
    // check-text: not a data name (the view's own)
    print(buf, 3, &page.name);
    print(buf, 4, &page.line);
    let top = rect.y + FACE.1 + 2;
    for (i, stat) in page.stats.iter().enumerate() {
        let (col, row) = (to_i32(i % 2), to_i32(i / 2));
        if row >= STAT_ROWS {
            break;
        }
        let at = (rect.x + 2 + col * SECOND, top + row);
        // check-text: not a data name (the view's own)
        let (row, gain) = stat_row_parts(&stat.name, stat.before, stat.gain);
        let n = buf.print(at.0, at.1, &row, ink.text, ink.inside);
        if let Some(gain) = gain {
            let gx = at.0 + i32::from(n);
            pd::bar(ctx, buf, gx - 1, at.1, cells(&gain) + 1, &gain);
        }
    }
}

#[cfg(test)]
mod tests;
