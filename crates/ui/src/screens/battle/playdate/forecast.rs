//! The Playdate's look of the attack forecast (ticket 1039,
//! `docs/design/playdate.md`, *Layout*): a window down one side of the
//! screen, the side away from the two units, with every line of the PC's
//! forecast (`look-and-feel.md`, *Attack forecast*): the attacker in the
//! left column and the target in the right, names, arms, what is left of
//! them, HP with a bar shading what would be lost, Hit and Crit; the
//! strikes numbered down the middle, `!` after an effective one, the
//! skull on the kill and the strikes after it in grey dots; then the
//! totals and the notes. Too many rows for the screen: the rules go
//! first, then the last notes.

use crate::prelude::*;
use trpg_core::Side;

use super::{hp_bar, tile_cells};
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::map_view::MapScene;
use crate::screen::Ctx;
use crate::screens::battle::glyph::forecast::skull;
use crate::screens::battle::view::{ForecastSideView, ForecastView, LeftView};
use crate::screens::playdate::{self as pd, cut, inks, to_i32, to_usize};

/// The window's width, in cells: half the screen, so the two sides it can
/// go on never overlap. Two columns and the strikes' numbers between
/// them, a cell either side inside the frame.
pub const W: i32 = 25;

/// Widest text in a column.
const COLUMN_W: usize = 10;

/// Where the target's column starts, from the window's text column.
const RIGHT: i32 = 11;

/// The strikes' numbers' column, from the window's text column.
const NUMBER: i32 = 10;

/// Widest text across the window.
const TEXT_W: usize = 21;

/// Length of each side's HP bar, in cells.
const HP_BAR_CELLS: i32 = 4;

/// A row of the forecast, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line {
    Skill,
    Names,
    Arms,
    Left,
    Hp,
    Hit,
    Crit,
    Affinity,
    Rule,
    Strike(usize),
    Total,
    Note(usize),
}

/// The rows `view` shows in `room` rows: every one it has, less the rules
/// and then the last notes while there are too many.
fn lines(view: &ForecastView, room: usize) -> Vec<Line> {
    let mut lines = Vec::new();
    lines.extend(view.skill.as_ref().map(|_| Line::Skill));
    lines.extend([Line::Names, Line::Arms]);
    if view.sides.iter().any(|s| s.left.is_some()) {
        lines.push(Line::Left);
    }
    lines.extend([Line::Hp, Line::Hit, Line::Crit]);
    if view.sides.iter().any(|s| s.affinity.is_some()) {
        lines.push(Line::Affinity);
    }
    lines.push(Line::Rule);
    lines.extend((0..view.strikes.len()).map(Line::Strike));
    lines.extend([Line::Rule, Line::Total]);
    lines.extend((0..view.notes.len()).map(Line::Note));
    if lines.len() > room {
        lines.retain(|&l| l != Line::Rule);
    }
    lines.truncate(room);
    lines
}

/// Where the forecast `view` goes over `area`, the map showing `scene`:
/// down the half whose window covers fewer of the two units' tiles, away
/// from the target if both cover one; from the top, as tall as its rows.
pub fn rect(ctx: &Ctx, scene: &MapScene, view: &ForecastView, area: Rect) -> Rect {
    let h = to_i32(lines(view, to_usize(area.h - 2)).len()) + 2;
    let left = Rect::new(area.x, area.y, W.min(area.w), h);
    let right = Rect::new(area.x + area.w - left.w, area.y, left.w, h);
    let tiles = view.tiles.map(|pos| tile_cells(ctx, scene, area, pos));
    let covered = |r: Rect| tiles.iter().flatten().filter(|t| overlap(r, **t)).count();
    let target_left = tiles[1].is_some_and(|t| t.x + t.w / 2 < area.x + area.w / 2);
    let (on_left, on_right) = (covered(left), covered(right));
    if on_right < on_left || (on_right == on_left && target_left) {
        right
    } else {
        left
    }
}

/// Whether `a` and `b` share a cell.
fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Paints the forecast `view` in its window over `area`.
pub fn paint(ctx: &Ctx, scene: &MapScene, view: &ForecastView, area: Rect, buf: &mut GlyphBuffer) {
    let rect = rect(ctx, scene, view, area);
    pd::window(ctx, buf, rect, &view.title);
    let x = rect.x + 2;
    let shown = lines(view, to_usize(rect.h - 2));
    for (y, line) in (rect.y + 1..).zip(shown) {
        let pen = Pen { ctx, x, y };
        match line {
            Line::Skill => paint_skill(pen, view, buf),
            Line::Rule => {
                let ink = inks(ctx);
                let rule = Cell::new('─', ink.frame, ink.inside);
                buf.fill_rect(Rect::new(rect.x + 1, y, rect.w - 2, 1), rule);
                buf.set(rect.x, y, Cell::new('╟', ink.frame, ink.inside));
                buf.set(
                    rect.x + rect.w - 1,
                    y,
                    Cell::new('╢', ink.frame, ink.inside),
                );
            }
            Line::Strike(i) => paint_strike(pen, view, i, buf),
            Line::Total => paint_totals(pen, view, buf),
            Line::Note(i) => pen.text(buf, 0, TEXT_W, &view.notes[i]),
            side_line => {
                for (side, dx) in view.sides.iter().zip([0, RIGHT]) {
                    paint_side_line(Pen { x: x + dx, ..pen }, view, side, side_line, buf);
                }
            }
        }
    }
}

/// Where a row is painted: from the window's text column `x`, on row
/// `y`.
#[derive(Clone, Copy)]
struct Pen<'a> {
    ctx: &'a Ctx,
    x: i32,
    y: i32,
}

impl Pen<'_> {
    /// `text`, `dx` cells along, cut to `w` cells: black on the white
    /// inside.
    fn text(self, buf: &mut GlyphBuffer, dx: i32, w: usize, text: &str) {
        let ink = inks(self.ctx);
        buf.print(self.x + dx, self.y, &cut(text, w), ink.text, ink.inside);
    }

    /// [`text`](Self::text) in grey dots.
    fn off(self, buf: &mut GlyphBuffer, dx: i32, w: usize, text: &str) {
        let ink = inks(self.ctx);
        buf.print(self.x + dx, self.y, &cut(text, w), ink.off, ink.inside);
    }
}

/// The art's or combat active's line: `Guard Break (20 → 16)`.
fn paint_skill(pen: Pen<'_>, view: &ForecastView, buf: &mut GlyphBuffer) {
    let Some(skill) = &view.skill else {
        return;
    };
    let cost = skill
        .durability
        .map_or_else(String::new, |(from, to)| format!(" ({from} → {to})"));
    // check-text: not a data name (the view's own)
    pen.text(buf, 0, TEXT_W, &format!("{}{cost}", skill.name));
}

/// `side`'s part of the row `line`, in its column.
fn paint_side_line(
    pen: Pen<'_>,
    view: &ForecastView,
    side: &ForecastSideView,
    line: Line,
    buf: &mut GlyphBuffer,
) {
    let shown = |v: Option<u8>| v.map_or_else(|| "--".to_owned(), |v| v.to_string());
    match line {
        // check-text: not a data name (the view's own)
        Line::Names => pen.text(buf, 0, COLUMN_W, &side.name),
        Line::Arms => pen.text(buf, 0, COLUMN_W, &side.weapon),
        Line::Left => match &side.left {
            Some(LeftView::Broken(broken)) => pen.text(buf, 0, COLUMN_W, broken),
            Some(LeftView::Count { left, max }) => {
                pen.text(buf, 0, COLUMN_W, &format!("{left}/{max}"));
            }
            None => {}
        },
        Line::Hp => {
            let hp = side.hp;
            pen.text(buf, 0, 2, &view.hp_name);
            pen.text(buf, 3, 2, &format!("{:>2}", hp.now));
            let at = (pen.x + 6, pen.y);
            hp_bar(pen.ctx, buf, at, HP_BAR_CELLS, (hp.now, hp.after, hp.max));
        }
        Line::Hit => {
            pen.text(buf, 0, 4, &view.hit_name);
            pen.text(buf, 5, 3, &shown(side.hit));
        }
        Line::Crit => {
            pen.text(buf, 0, 4, &view.crit_name);
            pen.text(buf, 5, 3, &shown(side.crit));
        }
        Line::Affinity => {
            if let Some(marker) = &side.affinity {
                pen.text(buf, 0, COLUMN_W, marker);
            }
        }
        _ => {}
    }
}

/// Strike `i`: its number in the middle, the attacker's damage on the left
/// (`8 dmg →`) or the target's on the right (`← 9 dmg`), `!` after an
/// effective one and the skull on the kill; in grey dots after a kill. The
/// first row says `no counter` on the right when the target can't strike
/// back.
fn paint_strike(pen: Pen<'_>, view: &ForecastView, i: usize, buf: &mut GlyphBuffer) {
    let ink = inks(pen.ctx);
    let s = &view.strikes[i];
    pen.off(buf, NUMBER, 2, &(i + 1).to_string());
    let (dmg, arrow_at, arrow) = match s.by {
        Side::Attacker => (0, 8, "→"),
        Side::Defender => (RIGHT + 2, RIGHT + 1, "←"),
    };
    if s.after_a_fall {
        pen.off(buf, dmg, 6, &s.text);
    } else {
        pen.text(buf, dmg, 6, &s.text);
    }
    pen.text(buf, arrow_at, 1, arrow);
    if s.effective {
        pen.text(buf, dmg + 6, 1, "!");
    }
    if s.kills {
        // The Playdate's cells are the PC's size (8 × 16 pixels).
        for o in skull(pen.x + dmg + 7, pen.y, ink.text) {
            buf.add_overlay(o);
        }
    }
    if i == 0
        && let Some(none) = &view.no_counter
    {
        pen.text(buf, RIGHT, COLUMN_W, none);
    }
}

/// The totals: what the row is called in the middle, each side's damage
/// and strikes (`17 ×2`) in its column.
fn paint_totals(pen: Pen<'_>, view: &ForecastView, buf: &mut GlyphBuffer) {
    pen.text(buf, NUMBER - 4, 5, &view.total_name);
    for (side, dx) in view.sides.iter().zip([0, RIGHT + 2]) {
        if let Some(total) = side.total {
            pen.text(
                buf,
                dx,
                7,
                &format!("{:>2} ×{}", total.damage, total.strikes),
            );
        }
    }
}

#[cfg(test)]
mod tests;
