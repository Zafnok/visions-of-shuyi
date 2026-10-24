//! The Playdate's look of the battle screen (ticket 1038, ADR-0054,
//! `docs/design/playdate.md`, *Layout*): the map fills the whole screen
//! (painted by the map skin, ADR-0038), with no side panel and no help
//! bar. Like Fire Emblem on the GBA, two small windows in the corners tell
//! of the unit and the terrain under the cursor, and move to the bottom
//! corners when the cursor comes near the top. Menus, questions, the
//! objective page, the battle notes and the banners are windows over the
//! map in the one-bit look the Playdate's screens share
//! ([`crate::screens::playdate`]): white inside a double black frame,
//! black letters, the focus a bar.
//!
//! The forecast, the unit info screen, the fight box, the EXP and level-up
//! pages and the rewind screen have a module each (ticket 1039), and so
//! has the combat picture (1041), which covers the whole screen while a
//! fight plays. The guide's box (1058) is still the PC's paint until its
//! ticket gives it the Playdate's. Nothing here decides what the screen
//! shows.

pub mod fight_box;
pub mod forecast;
pub mod info;
pub mod progress;
pub mod rewind;
pub mod stage;

use crate::prelude::*;

use super::glyph::{self, BANNER_H, BANNER_W, box_top_in, menu_origin_in};
use super::layout::map_area;
use super::notes::{unwrap_notes, wrap_notes};
use super::units::{hp_fill, meter_cells};
use super::view::{
    BannerView, BattleMenuView, BattleView, DialogView, MapInfoView, MenuHeader, MenuPlace,
    MessageView, NotesView, PanelView,
};
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::map_view::MapScene;
use crate::screen::Ctx;
use crate::screens::playdate::{
    self as pd, Answers, Item, MORE_ABOVE, MORE_BELOW, cells, cut, inks, to_i32, to_usize,
};
use crate::tips as tip_box;
use crate::widgets::{MenuItemView, MenuView};
use trpg_core::{Pos, StatValue};

/// While the cursor is on one of the map's top this many rows, the corner
/// pop-ups sit in the bottom corners (`docs/design/playdate.md`: they
/// "move to the bottom ones when the cursor comes near"). *Tunable.*
pub const TOP_ROWS: i32 = 5;

/// Paints `view` on the Playdate's screen: the map over all of it, or
/// the combat picture while a fight plays, then the windows over them.
pub fn paint(ctx: &Ctx, view: &BattleView, buf: &mut GlyphBuffer) {
    pd::clear(ctx, buf);
    let area = map_area(ctx);
    match &view.stage {
        // It covers the whole screen: the map isn't painted under it.
        Some(fight) => stage::paint(ctx, fight, area, buf),
        None => paint_map(ctx, view, area, buf),
    }
    match &view.dialog {
        Some(DialogView::Objective(o)) => {
            let mut lines = vec![o.objective.clone(), o.turn.clone()];
            if let Some(notes) = &o.notes {
                lines.push(String::new());
                lines.push(notes.title.clone());
                lines.extend(note_lines(notes, area));
            }
            paint_box(ctx, view, &o.title, &lines, buf);
        }
        Some(DialogView::Question(q)) => {
            pd::ask(ctx, buf, &q.question, Answers::Hint(&q.answers));
        }
        None => {}
    }
    if let Some(i) = &view.info {
        info::paint(ctx, i, area, buf);
    }
    if let Some(notes) = &view.notes {
        paint_box(ctx, view, &notes.title, &note_lines(notes, area), buf);
    }
    if let Some(b) = &view.banner {
        paint_banner(ctx, b, area, buf);
    }
    if let Some(lesson) = &view.lesson {
        glyph::paint_lesson(ctx, &view.scene, lesson, buf);
    }
    if let Some(tip) = &view.tip {
        tip_box::glyph::paint(ctx, tip, buf);
    }
}

/// Paints the map over `area`, the numbers and pop-ups over it, and the
/// menus, the fight box, the EXP and level-up pages and the forecast.
fn paint_map(ctx: &Ctx, view: &BattleView, area: Rect, buf: &mut GlyphBuffer) {
    ctx.game.map_skin.paint(ctx, &view.scene, area, buf);
    paint_numbers(ctx, view, area, buf);
    if let Some(panel) = view.panel.as_ref().filter(|_| browsing(view)) {
        paint_corners(ctx, &view.scene, panel, area, buf);
    }
    if let Some(m) = &view.message {
        paint_message(ctx, &view.scene, m, area, buf);
    }
    if let Some(m) = &view.menu
        && let MenuPlace::Beside(tile) = m.place
    {
        paint_menu_beside(ctx, view, m, tile, buf);
    }
    paint_parts(ctx, view, area, buf);
    if let Some(m) = &view.menu
        && m.place == MenuPlace::Centred
    {
        let (w, h) = menu_size(ctx, &m.menu, area);
        let at = (area.x + (area.w - w) / 2, area.y + (area.h - h) / 2);
        paint_menu(ctx, m, at, area, buf);
    }
}

/// Paints the fight box, the EXP and level-up pages and the forecast over
/// `area`.
fn paint_parts(ctx: &Ctx, view: &BattleView, area: Rect, buf: &mut GlyphBuffer) {
    if let Some(b) = &view.fight_box {
        fight_box::paint(ctx, b, area, buf);
    }
    if let Some(p) = &view.progress {
        progress::paint(ctx, p, area, buf);
    }
    if let Some(f) = &view.forecast {
        forecast::paint(ctx, &view.scene, f, area, buf);
    }
}

/// An HP bar of `w` cells from `at`, black on the white inside: the HP
/// `after` kept `█`, what would be lost (from `now`) `▒`, the rest blank.
/// One bit can't tell `▒` from the PC's `░` for the rest, so it is white,
/// as the enemies' HP line has the HP lost (1037).
fn hp_bar(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    (x, y): (i32, i32),
    w: i32,
    (now, after, max): (StatValue, StatValue, StatValue),
) {
    let ink = inks(ctx);
    let (kept, _) = hp_fill(after, max, w);
    let (shown, _) = hp_fill(now.max(after), max, w);
    let lost = Cell::new('▒', ink.text, ink.inside);
    let empty = Cell::new(' ', ink.text, ink.inside);
    meter_cells(buf, Rect::new(x, y, w, 1), shown, lost, empty);
    let full = Cell::new('█', ink.text, ink.inside);
    meter_cells(buf, Rect::new(x, y, shown, 1), kept, full, lost);
}

/// Whether the player is looking over the map: no menu, box or banner is
/// open, and no fight or page is on screen. The corner pop-ups show only
/// then.
pub fn browsing(view: &BattleView) -> bool {
    view.menu.is_none()
        && view.dialog.is_none()
        && view.notes.is_none()
        && view.banner.is_none()
        && view.info.is_none()
        && view.fight_box.is_none()
        && view.progress.is_none()
        && view.stage.is_none()
}

/// The map area's cells of `pos`, where the map skin paints it in `scene`.
fn tile_cells(ctx: &Ctx, scene: &MapScene, area: Rect, pos: Pos) -> Option<Rect> {
    ctx.game.map_skin.tile_cells(scene, area, pos)
}

/// Whether the corner pop-ups go in the bottom corners of `area`: while
/// the cursor is on one of its [`TOP_ROWS`] top rows.
pub fn corners_at_bottom(ctx: &Ctx, scene: &MapScene, area: Rect) -> bool {
    let cursor = scene.cursor.as_ref().map(|c| c.pos);
    let tile = cursor.and_then(|pos| tile_cells(ctx, scene, area, pos));
    tile.is_some_and(|t| t.y < area.y + TOP_ROWS)
}

/// Paints the corner pop-ups of `panel` while the cursor is on the map:
/// the unit under it (if any) in the left corner, its terrain in the
/// right.
fn paint_corners(
    ctx: &Ctx,
    scene: &MapScene,
    panel: &PanelView,
    area: Rect,
    buf: &mut GlyphBuffer,
) {
    if scene.cursor.is_none() {
        return;
    }
    let bottom = corners_at_bottom(ctx, scene, area);
    if let Some(unit) = &panel.unit {
        let hp = format!("{} {}/{}", unit.hp_name, unit.hp, unit.max);
        // check-text: not a data name (the view's own)
        corner(ctx, buf, area, &[unit.name.clone(), hp], (false, bottom));
    }
    if let Some(t) = &panel.terrain {
        // check-text: not a data name (the view's own)
        let mut lines = vec![t.name.clone(), t.bonuses.clone()];
        lines.extend(t.heals.clone());
        corner(ctx, buf, area, &lines, (true, bottom));
    }
}

/// A window of `lines` in a corner of `area`: the right one if `right`,
/// the bottom one if `bottom`. Returns the window.
fn corner(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    area: Rect,
    lines: &[String],
    (right, bottom): (bool, bool),
) -> Rect {
    let (w, h) = window_size(lines, area);
    let x = if right { area.x + area.w - w } else { area.x };
    let y = if bottom { area.y + area.h - h } else { area.y };
    let rect = Rect::new(x, y, w, h);
    window_of(ctx, buf, rect, "", lines);
    rect
}

/// The size of a window of `lines`: the longest with a cell either side
/// inside the frame, a row each; no bigger than `area`.
fn window_size(lines: &[String], area: Rect) -> (i32, i32) {
    let widest = lines.iter().map(|l| cells(l)).max().unwrap_or(0);
    let w = (widest + 4).min(area.w);
    let h = (to_i32(lines.len()) + 2).min(area.h);
    (w, h)
}

/// A window at `rect`, `title` in its frame, `lines` inside it from its
/// top (as many as fit, each cut to its width).
fn window_of(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, title: &str, lines: &[String]) {
    let ink = inks(ctx);
    pd::window(ctx, buf, rect, title);
    let room = to_usize(rect.h - 2);
    for (y, line) in (rect.y + 1..).zip(lines.iter().take(room)) {
        let shown = cut(line, to_usize(rect.w - 4));
        buf.print(rect.x + 2, y, &shown, ink.text, ink.inside);
    }
}

/// Paints the heal and burn numbers a row over the units they tell of.
fn paint_numbers(ctx: &Ctx, view: &BattleView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    for p in &view.popups {
        let Some(tile) = tile_cells(ctx, &view.scene, area, p.pos) else {
            continue;
        };
        let y = (tile.y - 1).max(area.y);
        buf.print(tile.x, y, &p.text(), ink.text, ink.inside);
    }
}

/// Paints the message (a combat's, the preview of what is being aimed, a
/// toast) in a one-line window across the middle of the edge of `area`
/// away from the cursor: the bottom while the cursor is in the top half.
fn paint_message(
    ctx: &Ctx,
    scene: &MapScene,
    message: &MessageView,
    area: Rect,
    buf: &mut GlyphBuffer,
) {
    let cursor = scene.cursor.as_ref().map(|c| c.pos);
    let tile = cursor.and_then(|pos| tile_cells(ctx, scene, area, pos));
    let bottom = tile.is_some_and(|cells| cells.y < area.y + area.h / 2);
    let lines = [message.text.clone()];
    let (w, h) = window_size(&lines, area);
    let top = if bottom { area.y + area.h - h } else { area.y };
    let rect = Rect::new(area.x + (area.w - w) / 2, top, w, h);
    window_of(ctx, buf, rect, "", &lines);
}

/// Paints the menu `m` beside where the map skin has `tile`, as on the PC:
/// right of it, or left where it would run off the screen.
fn paint_menu_beside(
    ctx: &Ctx,
    view: &BattleView,
    m: &BattleMenuView,
    tile: Pos,
    buf: &mut GlyphBuffer,
) {
    let area = map_area(ctx);
    let Some(cells) = tile_cells(ctx, &view.scene, area, tile) else {
        return;
    };
    let info = view.map_info.as_ref();
    let at = menu_origin_in(area, cells, framed_size(ctx, m, info, area));
    paint_menu_framed(ctx, m, info, at, area, buf);
}

/// The size of the window of `m` in `area` ([`menu_size`]), wide enough
/// for `info` in its frame too.
fn framed_size(
    ctx: &Ctx,
    m: &BattleMenuView,
    info: Option<&MapInfoView>,
    area: Rect,
) -> (i32, i32) {
    let (w, h) = menu_size(ctx, &m.menu, area);
    let framed = info.map_or(0, |i| cells(&i.turn).max(cells(&i.charges)) + 6);
    (w.max(framed).min(area.w), h)
}

/// An item's line: its label in the player's language and what follows.
fn item_text(ctx: &Ctx, item: &MenuItemView) -> String {
    let label = ctx.label(&item.label);
    match &item.suffix {
        Some((suffix, _)) => format!("{label} {suffix}"),
        None => label,
    }
}

/// Whether some item of `menu` has a note (the menu then has a note row).
fn has_notes(menu: &MenuView) -> bool {
    menu.items.iter().any(|i| i.note.is_some())
}

/// The items `menu`'s window shows in `area`, from the first: the menu's
/// own, or, if they don't fit the screen, as many as do around the focus.
fn shown(menu: &MenuView, area: Rect) -> (usize, usize) {
    let notes = if has_notes(menu) { 2 } else { 0 };
    let room = to_usize(area.h - 2 - notes);
    if menu.shown <= room {
        return (menu.first_shown.min(menu.items.len()), menu.shown);
    }
    let focus = menu.items.iter().position(|i| i.focused);
    (pd::first_shown(menu.items.len(), room, focus), room)
}

/// The size, in cells, of `menu`'s window in `area`: its longest line or
/// note with a cell either side, inside the frame (no wider than `area`);
/// a row per item shown, and a rule and a row for the focused item's note
/// if any item has a note.
pub fn menu_size(ctx: &Ctx, menu: &MenuView, area: Rect) -> (i32, i32) {
    let lines = menu.items.iter().map(|i| cells(&item_text(ctx, i)));
    let notes = menu
        .items
        .iter()
        .filter_map(|i| i.note.as_deref())
        .map(cells);
    let widest = lines.chain(notes).max().unwrap_or(0);
    let note_rows = if has_notes(menu) { 2 } else { 0 };
    let (_, count) = shown(menu, area);
    ((widest + 4).min(area.w), to_i32(count) + 2 + note_rows)
}

/// Paints the menu `m` in a window from `at`: an item a row (the focused
/// one the bar, one that can't be chosen in grey dots), the header in the
/// top of the frame, the focused item's note under a rule, and marks in
/// the frame where more items are above or below.
fn paint_menu(ctx: &Ctx, m: &BattleMenuView, at: (i32, i32), area: Rect, buf: &mut GlyphBuffer) {
    paint_menu_framed(ctx, m, None, at, area, buf);
}

/// [`paint_menu`] with the battle's turn in the top of the frame and the
/// rewind charges in its bottom (the map menu's `info`), for the turn and
/// charges the PC has room to show elsewhere.
fn paint_menu_framed(
    ctx: &Ctx,
    m: &BattleMenuView,
    info: Option<&MapInfoView>,
    at: (i32, i32),
    area: Rect,
    buf: &mut GlyphBuffer,
) {
    let ink = inks(ctx);
    let menu = &m.menu;
    let (w, h) = framed_size(ctx, m, info, area);
    let rect = Rect::new(at.0, at.1, w, h);
    pd::window(ctx, buf, rect, info.map_or("", |i| i.turn.as_str()));
    if let Some(i) = info {
        pd::footer(ctx, buf, rect, &i.charges);
    }
    if let Some(header) = &m.header {
        paint_header(ctx, header, rect, buf);
    }
    let (first, count) = shown(menu, area);
    let items = menu.items.iter().skip(first).take(count);
    for (y, it) in (rect.y + 1..).zip(items) {
        let state = Item::of(it.focused, it.enabled);
        pd::item(ctx, buf, (rect.x + 2, y), w - 4, &item_text(ctx, it), state);
    }
    let mut below = rect.y + h - 1;
    if has_notes(menu) {
        let rule = rect.y + h - 3;
        let line = Cell::new('─', ink.frame, ink.inside);
        buf.fill_rect(Rect::new(rect.x + 1, rule, w - 2, 1), line);
        buf.set(rect.x, rule, Cell::new('╟', ink.frame, ink.inside));
        buf.set(rect.x + w - 1, rule, Cell::new('╢', ink.frame, ink.inside));
        if let Some(note) = menu.focused().and_then(|i| i.note.as_deref()) {
            let shown = cut(note, to_usize(w - 4));
            buf.print(rect.x + 2, rule + 1, &shown, ink.text, ink.inside);
        }
        below = rule;
    }
    let mark = |buf: &mut GlyphBuffer, y, glyph| {
        buf.print(rect.x + w - 3, y, glyph, ink.text, ink.inside);
    };
    if first > 0 {
        mark(buf, rect.y, MORE_ABOVE);
    }
    if first + count < menu.items.len() {
        mark(buf, below, MORE_BELOW);
    }
}

/// Prints `header` in the top of the frame of the menu at `rect`: the
/// weapon (or the pack's size) and the uses left.
fn paint_header(ctx: &Ctx, header: &MenuHeader, rect: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let text = match &header.uses {
        // check-text: not a data name (the view's own)
        Some(uses) => format!(" {} {} ", header.name, uses.text),
        // check-text: not a data name (the view's own)
        None => format!(" {} ", header.name),
    };
    let shown = cut(&text, to_usize(rect.w - 4));
    buf.print(rect.x + 2, rect.y, &shown, ink.text, ink.inside);
}

/// The lines of `notes`, wrapped again to fit a window across `area`.
fn note_lines(notes: &NotesView, area: Rect) -> Vec<String> {
    let texts = unwrap_notes(&notes.lines);
    wrap_notes(texts.iter().map(String::as_str), to_usize(area.w - 4))
}

/// Paints a window of `lines` under `title`, centred across the map and
/// off the rows of the units the battle notes are about, as on the PC.
fn paint_box(ctx: &Ctx, view: &BattleView, title: &str, lines: &[String], buf: &mut GlyphBuffer) {
    let area = map_area(ctx);
    let (w, h) = window_size(lines, area);
    // Wide enough for the title in the frame too.
    let w = w.max(cells(title) + 6).min(area.w);
    let rows: Vec<i32> = view
        .noted
        .iter()
        .filter_map(|&pos| tile_cells(ctx, &view.scene, area, pos))
        .map(|cells| cells.y)
        .collect();
    let top = box_top_in(area, h, &rows).max(area.y);
    let rect = Rect::new(area.x + (area.w - w) / 2, top, w, h);
    window_of(ctx, buf, rect, title, lines);
}

/// Paints the banner in a window in the middle of `area`, its big line
/// over its second (the turn).
fn paint_banner(ctx: &Ctx, banner: &BannerView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let rect = Rect::new(
        area.x + (area.w - BANNER_W) / 2,
        area.y + (area.h - BANNER_H) / 2,
        BANNER_W,
        BANNER_H,
    );
    pd::window(ctx, buf, rect, "");
    let row = |dy| Rect::new(rect.x, rect.y + dy, rect.w, 1);
    buf.print_centred(row(2), &banner.title, ink.text);
    buf.print_centred(row(3), &banner.subtitle, ink.text);
}

#[cfg(test)]
mod tests;
