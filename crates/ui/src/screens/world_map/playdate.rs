//! The Playdate's look of the world map (ticket 1066,
//! `docs/design/playdate.md`): the glyph look's map (`glyph.rs`, Nick's
//! 1007 answer) in one bit, at today's 8 × 16 cells on the 50 × 15
//! screen, told apart without colour by the rules of the battle map
//! (*Telling things apart without colour*): black, grey, outlined.
//!
//! - **The map fills the screen** and scrolls with the cursor, kept in
//!   the middle where it can be (a map smaller than the screen is
//!   centred); there is no side panel and no help line, as on the battle
//!   screen (1038).
//! - **The ground** is the glyph look's terrain glyphs, black on white;
//!   **a road** is the same band, white, outlined by black lines; a road
//!   to a place not open yet is outlined in grey dots.
//! - **A place's plate** says its kind with its symbol, as on the PC, and
//!   with its fill ([`fill`]): a story battle, a side quest and a capital
//!   white symbols on a **black** plate; a fixed or random skirmish its
//!   symbol (`!`, `?`) on a **grey** plate; a town and a battle won an
//!   **outlined** plate; a place not open yet its symbol and outline in
//!   grey dots; a place to pass through a dot. The level marker (`Lv7`)
//!   is black above its place.
//! - **The army** is the lead's two letters white on a black plate under
//!   a black `▼` flag; **the cursor** is the corner marks that flip what
//!   they cover (the battle map's, which doesn't pulse in one bit).
//! - **The place under the cursor** is a window in a corner away from
//!   the cursor (the top one, or the bottom one when it hides fewer
//!   places), its name in the top of the frame, its kind and what the
//!   PC's panel says under it, the act and chapter in the bottom of the
//!   frame. **A list** (the menu, Find, a town's shops) is a window in
//!   that corner, its title in its frame, the place's window under it
//!   when both fit.
//!
//! Nothing here decides what the screen shows (ADR-0054).

use trpg_content::WorldMapDef;
use trpg_core::Pos;

use super::glyph::{
    PLATE_PX, army_px, corner_arms, ground, plate, plate_px, put, road_cells, run_px,
};
use super::view::{ListView, PanelView, PlaceMark, PlaceView, WorldMapView};
use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Overlay, PxRect, Rect};
use crate::map_view::playdate::Inks as MapInks;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{self as pd, cells, to_i32};

/// The widest the place's window gets, in cells: under half the screen,
/// so on the half away from the cursor it never covers the army's plate.
pub const PLACE_W: i32 = 22;

/// How a place's plate is filled, telling the kinds apart without colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// White symbol on a black plate.
    Black,
    /// The symbol, edged in white, on a grey plate.
    Grey,
    /// Black symbol in a black outline.
    Outlined,
    /// Symbol and outline in grey dots: not open yet.
    Locked,
    /// No plate: the symbol alone (a place to pass through).
    Bare,
}

/// How `place`'s plate is filled (*Claude's starting rule*, tunable): the
/// places the story goes through black, the skirmishes grey (as enemies
/// on the battle map), towns and battles won outlined (as a unit that
/// has acted), a place not open yet in grey dots.
pub fn fill(place: &PlaceView) -> Fill {
    match place.mark {
        PlaceMark::Empty => Fill::Bare,
        _ if place.locked => Fill::Locked,
        PlaceMark::StoryBattle | PlaceMark::SideQuest | PlaceMark::Capital => Fill::Black,
        PlaceMark::FixedSkirmish | PlaceMark::RandomSkirmish => Fill::Grey,
        PlaceMark::Town | PlaceMark::Cleared => Fill::Outlined,
    }
}

/// The inks the map is painted with: the battle map's (black, grey,
/// outlined), and the palette's.
struct Inks {
    map: MapInks,
    text: Rgb,
    ground: Rgb,
    off: Rgb,
    cursor: Rgb,
}

impl Inks {
    fn of(ctx: &Ctx) -> Self {
        let screen = pd::inks(ctx);
        Self {
            map: MapInks::of(&ctx.palette),
            text: screen.text,
            ground: screen.ground,
            off: screen.off,
            cursor: ctx.palette.get(UiColor::Cursor),
        }
    }

    /// A plate's fill, its symbol's ink and its outline, for `fill`.
    fn plate(&self, fill: Fill) -> (Option<Rgb>, Rgb, Option<Rgb>) {
        match fill {
            Fill::Black => (Some(self.map.player.0), self.map.player.1, None),
            Fill::Grey => (Some(self.map.enemy.0), self.map.enemy.1, None),
            Fill::Outlined => (None, self.text, Some(self.map.acted.2)),
            Fill::Locked => (None, self.off, Some(self.off)),
            Fill::Bare => (None, self.text, None),
        }
    }
}

/// Where the map's cell `(0, 0)` is on a screen `room` cells big, with the
/// cursor on cell `cursor`: each way, a map smaller than the screen
/// centred, a bigger one scrolled so the cursor is in the middle where it
/// can be.
pub fn origin(size: (i32, i32), room: (i32, i32), cursor: Pos) -> (i32, i32) {
    let axis = |size: i32, room: i32, at: i32| {
        if size <= room {
            (room - size) / 2
        } else {
            -(at - room / 2).clamp(0, size - room)
        }
    };
    (
        axis(size.0, room.0, cursor.x),
        axis(size.1, room.1, cursor.y),
    )
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &WorldMapView, buf: &mut GlyphBuffer) {
    pd::clear(ctx, buf);
    let Some(def) = ctx.game.content.worldmaps.get(&view.map) else {
        return;
    };
    let room = (i32::from(buf.width()), i32::from(buf.height()));
    let at = origin(def.size, room, view.cursor);
    paint_map(ctx, view, def, at, buf);
    paint_windows(ctx, view, at, buf);
}

/// The map, cell `(0, 0)` at `(ox, oy)`: its ground, roads, places, army
/// and cursor.
fn paint_map(
    ctx: &Ctx,
    view: &WorldMapView,
    def: &WorldMapDef,
    (ox, oy): (i32, i32),
    buf: &mut GlyphBuffer,
) {
    let ink = Inks::of(ctx);
    let on_screen = |p: Pos| Pos::new(p.x + ox, p.y + oy);
    for y in 0..i32::from(buf.height()) {
        for x in 0..i32::from(buf.width()) {
            let p = Pos::new(x - ox, y - oy);
            if (0..def.size.0).contains(&p.x) && (0..def.size.1).contains(&p.y) {
                let (glyph, _, _) = ground(def.ground_at(p), p.x, p.y);
                buf.set(x, y, Cell::new(glyph, ink.text, ink.ground));
            }
        }
    }
    let blank = |buf: &mut GlyphBuffer, p: Pos| put(buf, p, ' ', None);
    let runs = |roads: &[Vec<Pos>]| -> Vec<(Pos, Pos)> {
        let pairs = roads.iter().flat_map(|r| r.windows(2));
        pairs.map(|p| (on_screen(p[0]), on_screen(p[1]))).collect()
    };
    let (open, locked) = (runs(&view.roads), runs(&view.locked_roads));
    // The ground's glyphs give way to the roads (a run down is wider than
    // its cell, so its neighbours' too).
    for &(a, b) in locked.iter().chain(&open) {
        for p in road_cells(&[a, b]) {
            blank(buf, p);
            if a.x == b.x {
                blank(buf, Pos::new(p.x - 1, p.y));
                blank(buf, Pos::new(p.x + 1, p.y));
            }
        }
    }
    // Every road's outline, then its band over them, so roads that meet
    // join; the roads not open yet first, so an open road is over them.
    for (runs, edge) in [(&locked, ink.off), (&open, ink.text)] {
        for &(a, b) in runs {
            buf.add_overlay(Overlay::new(run_px(a, b, 1), edge, Layer::Under));
        }
        for &(a, b) in runs {
            buf.add_overlay(Overlay::new(run_px(a, b, 0), ink.ground, Layer::Under));
        }
    }
    for place in &view.places {
        paint_place(buf, &ink, place, on_screen(place.pos));
    }
    let army = on_screen(view.army);
    let (plate_ink, letters) = ink.map.player;
    for dx in [-1, 2] {
        blank(buf, Pos::new(army.x + dx, army.y));
    }
    let mut label = view.army_label.chars();
    for x in army.x..army.x + 2 {
        put(
            buf,
            Pos::new(x, army.y),
            label.next().unwrap_or(' '),
            Some(letters),
        );
    }
    put(buf, Pos::new(army.x, army.y - 1), '▼', Some(ink.text));
    let army_rect = army_px(army);
    buf.add_overlay(Overlay::new(army_rect, plate_ink, Layer::Under));
    let cursor = if view.cursor_on_army {
        army_rect
    } else {
        plate_px(on_screen(view.cursor), PLATE_PX.0)
    };
    for arm in corner_arms(cursor) {
        buf.add_overlay(Overlay::new(arm, ink.cursor, Layer::Over));
    }
}

/// `place` on the screen's cell `p`: the ground cleared beside it, its
/// plate ([`fill`]) with its kind's symbol, its level marker above.
fn paint_place(buf: &mut GlyphBuffer, ink: &Inks, place: &PlaceView, p: Pos) {
    for x in p.x - 1..=p.x + 1 {
        put(buf, Pos::new(x, p.y), ' ', None);
    }
    let (_, symbol) = plate(place.mark);
    let (back, letters, outline) = ink.plate(fill(place));
    put(buf, p, symbol, Some(letters));
    let rect = plate_px(p, PLATE_PX.0);
    if let Some(back) = back {
        buf.add_overlay(Overlay::new(rect, back, Layer::Under));
    }
    if let Some(outline) = outline {
        for side in sides(rect) {
            buf.add_overlay(Overlay::new(side, outline, Layer::Under));
        }
    }
    if let Some(level) = &place.level {
        let x = p.x - (cells(level) - 1) / 2;
        for (at, ch) in (x..).zip(level.chars()) {
            put(buf, Pos::new(at, p.y - 1), ch, Some(ink.text));
        }
    }
}

/// The four 1-pixel sides of `rect`: top, bottom, left, right.
fn sides(rect: PxRect) -> [PxRect; 4] {
    let (right, bottom) = (rect.x + rect.w - 1, rect.y + rect.h - 1);
    [
        PxRect::new(rect.x, rect.y, rect.w, 1),
        PxRect::new(rect.x, bottom, rect.w, 1),
        PxRect::new(rect.x, rect.y, 1, rect.h),
        PxRect::new(right, rect.y, 1, rect.h),
    ]
}

/// The windows over the map, on the side away from the cursor: the list
/// open at the top, then the place under the cursor under it when it
/// fits; with no list, the place's window in the top corner, or the
/// bottom one when that hides fewer places (*Claude's starting rule*).
/// Cell `(0, 0)` of the map is at `origin`.
fn paint_windows(ctx: &Ctx, view: &WorldMapView, origin: (i32, i32), buf: &mut GlyphBuffer) {
    let (screen_w, screen_h) = (i32::from(buf.width()), i32::from(buf.height()));
    let left_half = origin.0 + view.cursor.x < screen_w / 2;
    let x_for = |w: i32| if left_half { screen_w - w } else { 0 };
    let (w, lines) = place_lines(&view.panel);
    let h = to_i32(lines.len()) + 2;
    let mut rect = Rect::new(x_for(w), 0, w, h);
    if let Some(list) = &view.list {
        rect.y = paint_list(ctx, list, x_for, buf).h;
        if rect.y + h > screen_h {
            return;
        }
    } else {
        let bottom = Rect {
            y: screen_h - h,
            ..rect
        };
        if hidden(view, origin, bottom) < hidden(view, origin, rect) {
            rect = bottom;
        }
    }
    pd::window(ctx, buf, rect, &view.panel.title);
    pd::footer(ctx, buf, rect, &view.panel.act);
    let ink = pd::inks(ctx);
    for (row, line) in (rect.y + 1..).zip(&lines) {
        buf.print(rect.x + 2, row, line, ink.text, ink.inside);
    }
}

/// How many of the places and the army (each its cell and the row above,
/// its level marker or flag) a window at `rect` covers, cell `(0, 0)` of
/// the map at `origin`.
fn hidden(view: &WorldMapView, (ox, oy): (i32, i32), rect: Rect) -> usize {
    let inside = |x: i32, y: i32| {
        (rect.x..rect.x + rect.w).contains(&x) && (rect.y..rect.y + rect.h).contains(&y)
    };
    let places = view.places.iter().map(|p| p.pos).chain([view.army]);
    places
        .filter(|p| inside(p.x + ox, p.y + oy) || inside(p.x + ox, p.y + oy - 1))
        .count()
}

/// `list` in a window at the top, `x_for` its width saying where it goes
/// across. Returns the window.
fn paint_list(
    ctx: &Ctx,
    list: &ListView,
    x_for: impl Fn(i32) -> i32,
    buf: &mut GlyphBuffer,
) -> Rect {
    let (w, _) = pd::titled_menu_size(ctx, &list.menu, &list.title);
    pd::titled_menu(ctx, buf, (x_for(w), 0), &list.menu, &list.title)
}

/// The place window's width (room for its lines and for the name and the
/// act in its frame, at most [`PLACE_W`]) and its lines: the place's kind,
/// then what the PC's panel says under it, each cut to fit.
fn place_lines(panel: &PanelView) -> (i32, Vec<String>) {
    let lines: Vec<String> = core::iter::once(&panel.kind)
        .chain(&panel.lines)
        .map(|l| pd::cut(l, usize::try_from(PLACE_W - 4).unwrap_or(0)))
        .collect();
    let widest = lines.iter().map(|l| cells(l) + 4);
    let frame = [cells(&panel.title), cells(&panel.act)].map(|t| t + 6);
    let w = widest.chain(frame).max().unwrap_or(0);
    (w.min(PLACE_W), lines)
}

#[cfg(test)]
mod tests;
