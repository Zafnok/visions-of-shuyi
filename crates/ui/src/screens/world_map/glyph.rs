//! The glyph look of the world map (Nick, ticket 1007,
//! `docs/design/look-and-feel.md` *The world map*): the map's ground in
//! terrain glyphs on the left, the roads as a shaded band outlined by
//! lines (the same width across and down), each place a symbol plate of
//! its kind's colour with its level marker above, the army the lead's two
//! letters on a blue plate under a `▼` flag, the cursor's corner marks round
//! the plate under it; a place not open yet greyed, its road dimmed; the panel on the right, a list open over the map,
//! the help line at the bottom. Everything about the look lives here;
//! nothing here decides what the screen shows (ADR-0054).

use trpg_content::WorldMapDef;
use trpg_core::Pos;

use super::view::{PlaceMark, WorldMapView};
use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Layer, Overlay, PxRect, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::widgets::menu::glyph as menu_glyph;

/// The map's part of the screen, in cells.
pub const MAP: Rect = Rect::new(0, 0, 70, 30);
/// The panel beside the map.
pub const PANEL: Rect = Rect::new(70, 0, 30, 30);
/// Where a list open over the map starts (its title's row).
pub(super) const LIST: (i32, i32) = (36, 4);
/// A cell's size in pixels.
pub(super) const CELL: (i32, i32) = (8, 16);
/// The road band's width in pixels, across and down (TUNABLE).
pub const ROAD_PX: i32 = 12;
/// How far the band's shade goes from the ground toward the road's color.
const ROAD_SHADE: f32 = 0.25;
/// A place's plate: as wide as two cells, centred on its own.
pub(super) const PLATE_PX: (i32, i32) = (16, 16);
/// The army's plate, round its two letters.
const ARMY_PX: i32 = 24;
/// How long each arm of the cursor's corner marks is, in pixels.
const ARM: i32 = 3;

/// Where the map's cell `(0, 0)` is on the screen: the map centred in
/// [`MAP`].
pub fn origin(def: &WorldMapDef) -> (i32, i32) {
    let (w, h) = def.size;
    (
        MAP.x + (MAP.w - w).max(0) / 2,
        MAP.y + (MAP.h - h).max(0) / 2,
    )
}

/// A number from 0 to 99 for cell `(x, y)`, the same every frame: which
/// cells of a ground show a glyph.
fn speckle(x: i32, y: i32) -> u32 {
    let h = (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)).unsigned_abs();
    ((h.wrapping_mul(2_654_435_761) >> 16) & 0xffff) % 100
}

// check-text: not player text (palette colour names)
/// The glyph, color and background names of ground `name` at `(x, y)`.
fn ground(name: &str, x: i32, y: i32) -> (char, &'static str, &'static str) {
    let s = speckle(x, y);
    match name {
        "forest" => (if s < 75 { '♣' } else { ' ' }, "forest", "forest_bg"),
        "thicket" => ('♠', "thicket", "thicket_bg"),
        "mountain" => (if s < 35 { '^' } else { ' ' }, "mountain", "mountain_bg"),
        "peak" => (if s < 60 { '▲' } else { '^' }, "peak", "peak_bg"),
        "water" => ('≈', "water", "water_bg"),
        "sea" => (if s < 25 { '≈' } else { ' ' }, "water", "sea"),
        _ => match s {
            0..12 => ('.', "grass", "grass_bg"),
            12..18 => (',', "grass", "grass_bg"),
            _ => (' ', "grass", "grass_bg"),
        },
    }
}

// check-text: not player text (palette colour names)
/// The plate's color name and symbol of a place's mark.
pub(super) fn plate(mark: PlaceMark) -> (&'static str, char) {
    match mark {
        PlaceMark::StoryBattle => ("text_highlight", '‼'),
        PlaceMark::SideQuest => ("ally", '♥'),
        PlaceMark::Town => ("wood", '⌂'),
        PlaceMark::Capital => ("fort", '⌂'),
        PlaceMark::Cleared => ("panel_border", '·'),
        PlaceMark::Empty => ("path", '●'),
    }
}

// check-text: not player text (palette colour names)
// check-text: not player text (a palette colour name)
/// The colour name of a place not open yet, and of its road's edge.
pub(super) const LOCKED: &str = "text_dim";

// check-text: not player text (palette colour names)
/// The level marker's color name of a place's mark.
pub(super) fn level_ink(mark: PlaceMark) -> &'static str {
    match mark {
        PlaceMark::SideQuest => "ally",
        _ => "text_highlight",
    }
}

/// The pixel rectangle of a plate `w` pixels wide centred on cell `pos`.
pub(super) fn plate_px(pos: Pos, w: i32) -> PxRect {
    let (cw, ch) = CELL;
    PxRect::new(
        pos.x * cw + cw / 2 - w / 2,
        pos.y * ch,
        w,
        PLATE_PX.1.min(ch),
    )
}

/// The cells a road `cells` (its turns) runs through, end to end.
pub(super) fn road_cells(cells: &[Pos]) -> Vec<Pos> {
    let mut out = Vec::new();
    for pair in cells.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let (dx, dy) = ((b.x - a.x).signum(), (b.y - a.y).signum());
        let mut at = a;
        out.push(at);
        while at != b && (dx != 0 || dy != 0) {
            at = Pos::new(at.x + dx, at.y + dy);
            out.push(at);
        }
    }
    out
}

/// The band of a road's run from `a` to `b` (screen cells), `grow` pixels
/// bigger all round.
fn run_px(a: Pos, b: Pos, grow: i32) -> PxRect {
    let (cw, ch) = CELL;
    let centre = |p: Pos| (p.x * cw + cw / 2, p.y * ch + ch / 2);
    let ((ax, ay), (bx, by)) = (centre(a), centre(b));
    let half = ROAD_PX / 2 + grow;
    let (x0, y0) = (ax.min(bx) - half, ay.min(by) - half);
    PxRect::new(
        x0,
        y0,
        (ax - bx).abs() + half * 2,
        (ay - by).abs() + half * 2,
    )
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &WorldMapView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let named = |n: &str| ctx.palette.lookup(n).unwrap_or_else(|| c(UiColor::Text));
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    if let Some(def) = ctx.game.content.worldmaps.get(&view.map) {
        paint_map(ctx, view, def, buf, &named);
    }
    paint_panel(ctx, view, buf);
    if let Some(list) = &view.list {
        // The title on a tab above the list's box.
        let title = format!(" {} ", list.title);
        buf.print(
            LIST.0 + 1,
            LIST.1,
            &title,
            c(UiColor::TextHighlight),
            c(UiColor::PanelBg),
        );
        menu_glyph::paint(ctx, &list.menu, buf, LIST.0, LIST.1 + 1);
    }
    let bottom = i32::from(buf.height()) - 1;
    buf.print(1, bottom, &view.help, c(UiColor::Text), black);
}

/// The map: its ground, roads, places, army and cursor.
fn paint_map(
    ctx: &Ctx,
    view: &WorldMapView,
    def: &WorldMapDef,
    buf: &mut GlyphBuffer,
    named: &dyn Fn(&str) -> Rgb,
) {
    let (ox, oy) = origin(def);
    let on_screen = |p: Pos| Pos::new(p.x + ox, p.y + oy);
    paint_ground(def, (ox, oy), buf, named);
    let blank = |buf: &mut GlyphBuffer, p: Pos| {
        if let Some(cell) = buf.get(p.x, p.y).copied() {
            buf.set(p.x, p.y, Cell { glyph: ' ', ..cell });
        }
    };
    // The ground's glyphs give way to the roads (a run down is wider
    // than its cell, so its neighbours' too).
    for road in view.roads.iter().chain(&view.locked_roads) {
        let cells = road_cells(road);
        for (i, &p) in cells.iter().enumerate() {
            let p = on_screen(p);
            blank(buf, p);
            let down = cells.get(i + 1).is_some_and(|n| n.x == cells[i].x)
                || (i > 0 && cells[i - 1].x == cells[i].x);
            if down {
                blank(buf, Pos::new(p.x - 1, p.y));
                blank(buf, Pos::new(p.x + 1, p.y));
            }
        }
    }
    let ink = named("panel_bg");
    for place in &view.places {
        let p = on_screen(place.pos);
        for dx in -1..=1 {
            blank(buf, Pos::new(p.x + dx, p.y));
        }
        let (color, symbol) = plate(place.mark);
        let color = if place.locked { LOCKED } else { color };
        // An empty place is a dot on the road, with no plate.
        let fg = if place.mark == PlaceMark::Empty {
            named(color)
        } else {
            ink
        };
        let bg = buf.get(p.x, p.y).map_or(ink, |cell| cell.bg);
        buf.set(p.x, p.y, Cell::new(symbol, fg, bg));
        if let Some(level) = &place.level {
            let len = i32::try_from(level.chars().count()).unwrap_or(0);
            let x = p.x - (len - 1) / 2;
            for (i, ch) in level.chars().enumerate() {
                let at = Pos::new(x + i32::try_from(i).unwrap_or(0), p.y - 1);
                let bg = buf.get(at.x, at.y).map_or(ink, |cell| cell.bg);
                buf.set(at.x, at.y, Cell::new(ch, named(level_ink(place.mark)), bg));
            }
        }
    }
    let army = on_screen(view.army);
    let player = ctx.palette.get(UiColor::Player);
    for (i, ch) in view.army_label.chars().take(2).enumerate() {
        let x = army.x + i32::try_from(i).unwrap_or(0);
        let bg = buf.get(x, army.y).map_or(ink, |cell| cell.bg);
        buf.set(x, army.y, Cell::new(ch, ink, bg));
    }
    for dx in [-1, 2] {
        blank(buf, Pos::new(army.x + dx, army.y));
    }
    let bg = buf.get(army.x, army.y - 1).map_or(ink, |cell| cell.bg);
    buf.set(army.x, army.y - 1, Cell::new('▼', player, bg));
    // The pictures in pixels, under the glyphs: every road's outline
    // first, then its band (so roads that meet join), then the plates.
    // The roads not open yet first, dimmed, so open roads that meet them
    // are drawn over them.
    let runs = |roads: &[Vec<Pos>]| -> Vec<(Pos, Pos)> {
        let pairs = roads.iter().flat_map(|r| r.windows(2));
        pairs.map(|p| (on_screen(p[0]), on_screen(p[1]))).collect()
    };
    let shade = named("road_bg").lerp(named("road"), ROAD_SHADE);
    paint_bands(
        buf,
        &runs(&view.locked_roads),
        named(LOCKED),
        named("road_bg"),
    );
    paint_bands(
        buf,
        &runs(&view.roads),
        ctx.palette.get(UiColor::Path),
        shade,
    );
    for place in view.places.iter().filter(|p| p.mark != PlaceMark::Empty) {
        let (color, _) = plate(place.mark);
        let color = if place.locked { LOCKED } else { color };
        let rect = plate_px(on_screen(place.pos), PLATE_PX.0);
        buf.add_overlay(Overlay::new(rect, named(color), Layer::Under));
    }
    // The army's plate is round its two letters, the cell beside its own.
    let army_rect = plate_px(Pos::new(army.x, army.y), ARMY_PX);
    let army_rect = PxRect::new(
        army_rect.x + CELL.0 / 2,
        army_rect.y,
        army_rect.w,
        army_rect.h,
    );
    buf.add_overlay(Overlay::new(army_rect, player, Layer::Under));
    let cursor = if view.cursor_on_army {
        army_rect
    } else {
        plate_px(on_screen(view.cursor), PLATE_PX.0)
    };
    paint_cursor(ctx, cursor, view.cursor_brightness, buf);
}

/// The cursor's corner marks round `rect`, as bright as `brightness`.
fn paint_cursor(ctx: &Ctx, rect: PxRect, brightness: f32, buf: &mut GlyphBuffer) {
    let dark = ctx.palette.get(UiColor::Black);
    let color = ctx
        .palette
        .get(UiColor::Cursor)
        .lerp(dark, 1.0 - brightness);
    for arm in corner_arms(rect) {
        buf.add_overlay(Overlay::new(arm, color, Layer::Over));
    }
}

/// The map's ground in its cells, cell `(0, 0)` at `(ox, oy)`.
fn paint_ground(
    def: &WorldMapDef,
    (ox, oy): (i32, i32),
    buf: &mut GlyphBuffer,
    named: &dyn Fn(&str) -> Rgb,
) {
    let (w, h) = def.size;
    for y in 0..h {
        for x in 0..w {
            let (ch, fg, bg) = ground(def.ground_at(Pos::new(x, y)), x, y);
            buf.set(ox + x, oy + y, Cell::new(ch, named(fg), named(bg)));
        }
    }
}

/// Road bands along `runs`: every run's outline in `edge` first, then
/// every band in `band`, so roads that meet join.
fn paint_bands(buf: &mut GlyphBuffer, runs: &[(Pos, Pos)], edge: Rgb, band: Rgb) {
    for &(a, b) in runs {
        buf.add_overlay(Overlay::new(run_px(a, b, 1), edge, Layer::Under));
    }
    for &(a, b) in runs {
        buf.add_overlay(Overlay::new(run_px(a, b, 0), band, Layer::Under));
    }
}

/// The eight 1 px arms of corner marks just outside `rect`.
fn corner_arms(rect: PxRect) -> [PxRect; 8] {
    let (l, t) = (rect.x - 1, rect.y);
    let (r, b) = (rect.x + rect.w, rect.y + rect.h - 1);
    let across = |x, y| PxRect::new(x, y, ARM, 1);
    let down = |x, y| PxRect::new(x, y, 1, ARM);
    [
        across(l, t),
        down(l, t),
        across(r - ARM + 1, t),
        down(r, t),
        across(l, b),
        down(l, b - ARM + 1),
        across(r - ARM + 1, b),
        down(r, b - ARM + 1),
    ]
}

/// The panel: the place under the cursor.
pub(super) fn paint_panel(ctx: &Ctx, view: &WorldMapView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (border, bg) = (c(UiColor::PanelBorderFocus), c(UiColor::PanelBg));
    buf.fill_rect(PANEL, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(PANEL, BoxStyle::Double, border, bg);
    let panel = &view.panel;
    let title = format!(" {} ", panel.title);
    let width = i32::try_from(title.chars().count()).unwrap_or(0);
    let x = PANEL.x + (PANEL.w - width).max(0) / 2;
    buf.print(x, PANEL.y, &title, c(UiColor::TextHighlight), bg);
    let x = PANEL.x + 2;
    buf.print(x, PANEL.y + 1, &panel.act, c(UiColor::TextDim), bg);
    buf.print(x, PANEL.y + 3, &panel.kind, c(UiColor::TextHighlight), bg);
    for (i, line) in panel.lines.iter().enumerate() {
        let y = PANEL.y + 4 + i32::try_from(i).unwrap_or(0);
        buf.print(x, y, line, c(UiColor::Text), bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_road_runs_through_every_cell_between_its_turns() {
        let p = Pos::new;
        assert_eq!(
            road_cells(&[p(1, 1), p(1, 3), p(3, 3)]),
            [p(1, 1), p(1, 2), p(1, 3), p(1, 3), p(2, 3), p(3, 3)]
        );
    }

    /// The band is as wide across as down (Nick: not narrow one way and
    /// wide the other).
    #[test]
    fn the_road_band_is_as_wide_across_as_down() {
        let p = Pos::new;
        let across = run_px(p(1, 1), p(5, 1), 0);
        let down = run_px(p(1, 1), p(1, 5), 0);
        assert_eq!((across.h, down.w), (ROAD_PX, ROAD_PX));
    }

    /// A map smaller than the screen's map area is centred in it.
    #[test]
    fn a_small_map_is_centred() {
        let ctx = crate::screen::tests::ctx();
        let maps = &ctx.game.content.worldmaps;
        assert_eq!(origin(&maps["test_world"]), (0, 0));
        // 40 × 12 in 70 × 30.
        assert_eq!(origin(&maps["test_world_2"]), (15, 9));
    }

    #[test]
    fn the_ground_speckle_is_a_percentage() {
        for x in 0..70 {
            for y in 0..30 {
                assert!(speckle(x, y) < 100);
            }
        }
    }
}
