//! The Playdate's look of the attack forecast (ticket 1039,
//! `docs/design/playdate.md`, *Layout*): a window down one side of the
//! screen, the side away from the two units, with every line of the PC's
//! forecast (`look-and-feel.md`, *Attack forecast*): the attacker in the
//! left column and the target in the right, names, arms and their uses
//! (an art's cost or `(broken)` on a row under them), `HP 19 → 6` over a
//! bar blinking what would be lost (ticket 0480), Hit and Crit; the
//! strikes numbered down the middle, `!` after an effective one, the
//! skull on the kill and the strikes after it in grey dots; then the
//! totals, a row per unit an art strikes and the notes. Too many rows for
//! the screen: the rules go first, then the last notes. When left and
//! right swap the weapon, the `◂▸` mark before it and `Left/Right weapon`
//! in the bottom of the frame, as on the PC (0463).

use crate::prelude::*;
use trpg_core::{Side, StatValue};

use super::tile_cells;
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::map_view::MapScene;
use crate::screen::Ctx;
use crate::screens::battle::glyph::forecast::{
    SIDES, Under, lost_shaded, skull, swap_arrows, under, uses_after_name,
};
use crate::screens::battle::units::{lit_fill, meter_cells};
use crate::screens::battle::view::{ForecastSideView, ForecastView};
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

/// Length of each side's HP bar, in cells: its column (0480).
const HP_BAR_CELLS: i32 = 10;

/// A row of the forecast, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line {
    Skill,
    Names,
    Arms,
    Under,
    Hp,
    Bar,
    Hit,
    Crit,
    Affinity,
    Rule,
    Strike(usize),
    Total,
    Struck(usize),
    Note(usize),
}

/// The rows `view` shows in `room` rows: every one it has, less the rules
/// and then the last notes while there are too many.
fn lines(view: &ForecastView, room: usize) -> Vec<Line> {
    let mut lines = Vec::new();
    lines.extend(view.skill.as_ref().map(|_| Line::Skill));
    lines.extend([Line::Names, Line::Arms]);
    let mut sides = view.sides.iter().zip(SIDES);
    if sides.any(|(s, by)| under(view, s, by).is_some()) {
        lines.push(Line::Under);
    }
    lines.extend([Line::Hp, Line::Bar, Line::Hit, Line::Crit]);
    if view.sides.iter().any(|s| s.affinity.is_some()) {
        lines.push(Line::Affinity);
    }
    lines.push(Line::Rule);
    lines.extend((0..view.strikes.len()).map(Line::Strike));
    lines.extend([Line::Rule, Line::Total]);
    lines.extend((0..view.struck.len()).map(Line::Struck));
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
    // A tile is never across the middle: the halves are the windows.
    let target_left = tiles[1].is_some_and(|t| t.x < area.x + area.w / 2);
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
    if let Some(swap) = &view.swap {
        pd::footer(ctx, buf, rect, swap);
    }
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
            Line::Struck(i) => paint_struck(pen, view, i, buf),
            Line::Note(i) => pen.text(buf, 0, TEXT_W, &view.notes[i]),
            Line::Arms if view.swap.is_some() => {
                for o in swap_arrows(rect.x + 1, y, inks(ctx).text) {
                    buf.add_overlay(o);
                }
                paint_sides(pen, view, line, buf);
            }
            side_line => paint_sides(pen, view, side_line, buf),
        }
    }
}

/// Row `line` of both sides' columns, from `pen`.
fn paint_sides(pen: Pen<'_>, view: &ForecastView, line: Line, buf: &mut GlyphBuffer) {
    let sides = view.sides.iter().zip(SIDES).zip([0, RIGHT]);
    for ((side, by), dx) in sides {
        let pen = Pen {
            x: pen.x + dx,
            ..pen
        };
        paint_side_line(pen, view, (side, by), line, buf);
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

/// The art's or combat active's line: `Guard Break` (the uses it spends
/// show under the weapon, 0480).
fn paint_skill(pen: Pen<'_>, view: &ForecastView, buf: &mut GlyphBuffer) {
    if let Some(skill) = &view.skill {
        // check-text: not a data name (the view's own)
        pen.text(buf, 0, TEXT_W, &skill.name);
    }
}

/// The weapon's name and its uses after it (`Iron S… 20`), the name cut to
/// leave them room.
fn arms_text(name: &str, uses: Option<u32>) -> String {
    match uses {
        Some(uses) => {
            let uses = uses.to_string();
            let room = COLUMN_W.saturating_sub(uses.chars().count() + 1);
            format!("{} {uses}", cut(name, room))
        }
        None => cut(name, COLUMN_W),
    }
}

/// The HP bar at `pen`, [`HP_BAR_CELLS`] long: `after`
/// HP black, the HP that would be lost (`hp - after`) a checker, or black
/// on the blink's other half, the rest white. A unit with HP keeps a cell
/// (`lit_fill`).
fn paint_bar(
    pen: Pen<'_>,
    buf: &mut GlyphBuffer,
    (hp, after, max): (StatValue, StatValue, StatValue),
) {
    let ink = inks(pen.ctx);
    let (kept, _) = lit_fill(after, max, HP_BAR_CELLS);
    let (shown, _) = lit_fill(hp.max(after), max, HP_BAR_CELLS);
    let full = Cell::new('█', ink.text, ink.inside);
    let lost = if lost_shaded(pen.ctx.clock_s) {
        Cell::new('▒', ink.text, ink.inside)
    } else {
        full
    };
    let empty = Cell::new(' ', ink.text, ink.inside);
    meter_cells(
        buf,
        Rect::new(pen.x, pen.y, HP_BAR_CELLS, 1),
        shown,
        lost,
        empty,
    );
    meter_cells(buf, Rect::new(pen.x, pen.y, shown, 1), kept, full, lost);
}

/// `side`'s part of the row `line`, in its column.
fn paint_side_line(
    pen: Pen<'_>,
    view: &ForecastView,
    (side, by): (&ForecastSideView, Side),
    line: Line,
    buf: &mut GlyphBuffer,
) {
    let shown = |v: Option<u8>| v.map_or_else(|| "--".to_owned(), |v| v.to_string());
    match line {
        // check-text: not a data name (the view's own)
        Line::Names => pen.text(buf, 0, COLUMN_W, &side.name),
        Line::Arms => {
            let uses = uses_after_name(view, side, by);
            pen.text(buf, 0, COLUMN_W, &arms_text(&side.weapon, uses));
        }
        Line::Under => match under(view, side, by) {
            Some(Under::Spends(from, to)) => {
                pen.text(buf, 0, COLUMN_W, &format!("{from} → {to}"));
            }
            Some(Under::Broken(broken)) => pen.text(buf, 0, COLUMN_W, broken),
            None => {}
        },
        Line::Hp => {
            let hp = side.hp;
            pen.text(buf, 0, 2, &view.hp_name);
            let change = format!("{:>2} → {}", hp.now, hp.after);
            pen.text(buf, 3, COLUMN_W - 3, &change);
        }
        Line::Bar => {
            let hp = side.hp;
            paint_bar(pen, buf, (hp.now, hp.after, hp.max));
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

/// Row `i` of the units an art strikes: the name, the hit and the HP
/// before and after a hit (`Hound     82 11 → 4`).
fn paint_struck(pen: Pen<'_>, view: &ForecastView, i: usize, buf: &mut GlyphBuffer) {
    let row = &view.struck[i];
    // check-text: not a data name (the view's own)
    pen.text(buf, 0, 9, &row.name);
    pen.text(buf, 10, 3, &format!("{:>3}", row.hit));
    let (before, after) = row.hp;
    pen.text(buf, 14, 7, &format!("{before:>2} → {after}"));
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
