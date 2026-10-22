//! The Playdate's look of the unit info screen (ticket 1039,
//! `docs/design/playdate.md`, *Layout*): a window over the whole screen,
//! the unit's name in the top of its frame, showing what the PC's info
//! screen shows a page at a time; Left and Right turn the pages
//! ([`info::turn_page`](crate::screens::battle::info::turn_page)), the
//! page and how many there are in the bottom of the frame. Each page's
//! lists stop where the window does, as on the PC:
//!
//! 1. the class and level, HP with a bar, EXP, Mov with the movement type
//!    and the class's tags; the stats, a raised or lowered one as the
//!    bar, and the timed effects beside them;
//! 2. the weapons, the armour and the accessory, the weapon ranks and the
//!    affinities (each list on a line);
//! 3. the skills, then the spells two to a row.

use crate::prelude::*;

use super::hp_bar;
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::battle::view::{GearView, InfoView, InfoWeaponView};
use crate::screens::playdate::{self as pd, cells, cut, inks, to_i32, to_usize};

/// How many pages the screen has.
pub const PAGES: u8 = 3;

/// Length of the HP bar, in cells.
const HP_BAR_CELLS: i32 = 10;

/// Where the second column starts, from the window's text column.
const SECOND: i32 = 24;

/// Widest text in either column.
const HALF_W: usize = 22;

/// Most timed effects on the first page, three rows each.
const EFFECT_MAX: usize = 2;

/// Shown for an empty list or slot.
const NOTHING: &str = "--";

/// Where a page is painted: the window's text column and its rows.
struct Page<'a> {
    ctx: &'a Ctx,
    buf: &'a mut GlyphBuffer,
    /// The window's first text column.
    x: i32,
    /// The window's first row inside the frame.
    top: i32,
    /// The row under the window's last one inside the frame.
    bottom: i32,
    /// The window's widest line of text.
    width: usize,
}

impl Page<'_> {
    /// `text` from `dx` cells along row `dy` of the window (counted from
    /// its first row inside the frame), cut to `w` cells: nothing past
    /// the window's last row.
    fn text(&mut self, (dx, dy): (i32, i32), w: usize, text: &str) {
        let y = self.top + dy;
        if y >= self.bottom {
            return;
        }
        let ink = inks(self.ctx);
        let w = w.min(self.width.saturating_sub(to_usize(dx)));
        self.buf
            .print(self.x + dx, y, &cut(text, w), ink.text, ink.inside);
    }

    /// `text` ending at `right` cells along row `dy`.
    fn right(&mut self, (right, dy): (i32, i32), text: &str) {
        self.text((right - cells(text), dy), to_usize(cells(text)), text);
    }

    /// `text` as the bar at `dx` along row `dy`: white letters on black,
    /// where the PC lights it.
    fn lit(&mut self, (dx, dy): (i32, i32), text: &str) {
        let y = self.top + dy;
        if y < self.bottom {
            let (bg, fg) = inks(self.ctx).bar;
            self.buf.print(self.x + dx, y, text, fg, bg);
        }
    }
}

/// Paints the info screen `view` over `area`: its page
/// [`page`](InfoView::page).
pub fn paint(ctx: &Ctx, view: &InfoView, area: Rect, buf: &mut GlyphBuffer) {
    // check-text: not a data name (the view's own)
    pd::window(ctx, buf, area, &view.name);
    let page = view.page.min(PAGES - 1);
    pd::footer(ctx, buf, area, &format!("◀ {}/{PAGES} ▶", page + 1));
    let mut pen = Page {
        ctx,
        buf,
        x: area.x + 2,
        top: area.y + 1,
        bottom: area.y + area.h - 1,
        width: to_usize(area.w - 4),
    };
    match page {
        0 => paint_unit(&mut pen, view),
        1 => paint_loadout(&mut pen, view),
        _ => paint_skills_and_spells(&mut pen, view),
    }
}

/// The first page: the class and level, HP, EXP, movement and tags, then
/// the stats and the timed effects.
fn paint_unit(pen: &mut Page<'_>, view: &InfoView) {
    pen.text((0, 0), pen.width, &view.class_level);
    let hp = format!("{} {}/{}", view.hp_name, view.hp, view.max);
    pen.text((0, 1), 10, &hp);
    let y = pen.top + 1;
    if y < pen.bottom {
        let at = (pen.x + 11, y);
        hp_bar(
            pen.ctx,
            pen.buf,
            at,
            HP_BAR_CELLS,
            (view.hp, view.hp, view.max),
        );
    }
    let exp = view
        .exp
        .map_or_else(|| NOTHING.to_owned(), |e| e.to_string());
    pen.text((0, 2), pen.width, &format!("{} {exp}", view.exp_name));
    let mut moves = vec![format!("{} {}", view.mov_name, view.mov)];
    moves.extend(view.movement.clone());
    moves.extend(view.tags.iter().cloned());
    pen.text((0, 3), pen.width, &moves.join("  "));
    pen.text((0, 5), HALF_W, &view.stats_title);
    let rows = view.stats.len().div_ceil(2);
    for (i, stat) in view.stats.iter().enumerate() {
        let (dx, dy) = (to_i32(i / rows.max(1)) * 10, 6 + to_i32(i % rows.max(1)));
        // check-text: not a data name (the view's own)
        pen.text((dx, dy), 4, &stat.name);
        let value = format!("{:>3}", stat.value);
        if stat.changed {
            pen.lit((dx + 4, dy), &value);
        } else {
            pen.text((dx + 4, dy), 3, &value);
        }
    }
    if view.effects.is_empty() {
        return;
    }
    pen.text((SECOND, 5), HALF_W, &view.effects_title);
    for (dy, effect) in (6..).step_by(3).zip(view.effects.iter().take(EFFECT_MAX)) {
        // check-text: not a data name (the view's own)
        pen.text((SECOND, dy), HALF_W, &effect.name);
        pen.text((SECOND, dy + 1), HALF_W, &effect.mods);
        pen.text((SECOND, dy + 2), HALF_W, &effect.until);
    }
}

/// The second page: the weapon slots (the equipped one marked, its
/// durability at the right, its numbers under it), then the armour and
/// the accessory side by side, then the weapon ranks and the affinities.
fn paint_loadout(pen: &mut Page<'_>, view: &InfoView) {
    pen.text((0, 0), pen.width, &view.weapons_title);
    let mut dy = 1;
    for weapon in &view.weapons {
        match weapon {
            Some(w) => paint_weapon(pen, view, w, dy),
            None => pen.text((2, dy), 2, NOTHING),
        }
        dy += 2;
    }
    for (gear, dx) in [(&view.armour, 0), (&view.accessory, SECOND)] {
        paint_gear(pen, gear, (dx, dy + 1));
    }
    let pairs = |list: &[(String, String)]| {
        let shown: Vec<String> = list.iter().map(|(a, b)| format!("{a} {b}")).collect();
        shown.join("  ")
    };
    pen.text((0, dy + 4), HALF_W, &view.ranks_title);
    pen.text((2, dy + 5), HALF_W - 2, &pairs(&view.ranks));
    if !view.affinities.is_empty() {
        pen.text((SECOND, dy + 4), HALF_W, &view.affinities_title);
        pen.text((SECOND + 2, dy + 5), HALF_W - 2, &pairs(&view.affinities));
    }
}

/// One weapon on rows `dy` and `dy + 1`.
fn paint_weapon(pen: &mut Page<'_>, view: &InfoView, w: &InfoWeaponView, dy: i32) {
    if w.equipped {
        pen.text((0, dy), 1, &view.equipped_mark);
    }
    let dur = w
        .broken
        .clone()
        .unwrap_or_else(|| format!("{:>2}/{}", w.left, w.max));
    let right = to_i32(pen.width);
    // check-text: not a data name (the view's own)
    pen.text((2, dy), to_usize(right - cells(&dur) - 3), &w.name);
    pen.right((right, dy), &dur);
    pen.text((2, dy + 1), pen.width - 2, &w.numbers);
}

/// A piece of gear from row `dy`, `dx` cells along: its heading, name and
/// bonus.
fn paint_gear(pen: &mut Page<'_>, gear: &GearView, (dx, dy): (i32, i32)) {
    pen.text((dx, dy), HALF_W, &gear.title);
    match &gear.item {
        Some((name, bonus)) => {
            pen.text((dx + 2, dy + 1), HALF_W - 2, name);
            pen.text((dx + 2, dy + 2), HALF_W - 2, bonus);
        }
        None => pen.text((dx + 2, dy + 1), 2, NOTHING),
    }
}

/// The last page: each skill's mark, name and an active's cost, its
/// effect under it; then the spells and their uses left (the equipped one
/// marked), two to a row. What doesn't fit the window is left out, as the
/// PC's lists stop where its box does.
fn paint_skills_and_spells(pen: &mut Page<'_>, view: &InfoView) {
    pen.text((0, 0), pen.width, &view.skills_title);
    if view.skills.is_empty() {
        pen.text((2, 1), 2, NOTHING);
    }
    let right = to_i32(pen.width);
    let mut dy = 1;
    for skill in &view.skills {
        pen.text((0, dy), 1, &skill.mark);
        let cost = skill.cost.as_deref().unwrap_or_default();
        let room = right - 2 - cells(cost) - 1;
        // check-text: not a data name (the view's own)
        pen.text((2, dy), to_usize(room), &skill.name);
        pen.right((right, dy), cost);
        pen.text((2, dy + 1), pen.width - 2, &skill.effect);
        dy += 2;
    }
    let dy = dy.max(2) + 1;
    pen.text((0, dy), pen.width, &view.spells_title);
    if view.spells.is_empty() {
        pen.text((2, dy + 1), 2, NOTHING);
    }
    for (i, spell) in view.spells.iter().enumerate() {
        let (dx, row) = (to_i32(i % 2) * SECOND, dy + 1 + to_i32(i / 2));
        if spell.equipped {
            pen.text((dx, row), 1, &view.equipped_mark);
        }
        let uses = format!("{}/{}", spell.left, spell.uses);
        let end = dx + to_i32(HALF_W);
        // check-text: not a data name (the view's own)
        pen.text(
            (dx + 2, row),
            to_usize(end - dx - 2 - cells(&uses) - 1),
            &spell.name,
        );
        pen.right((end, row), &uses);
    }
}

#[cfg(test)]
mod tests;
