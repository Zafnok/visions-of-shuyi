//! The picture look of the world map (Nick, ticket 1007,
//! `docs/design/look-and-feel.md` *The world map*), for a map with a
//! picture when the player's map look is pictures: the map's picture
//! under everything, the roads as pale dots with a dark edge (like the
//! routes of Sacred Stones' map), the same symbol plates and level markers
//! as the glyph look, with a dark edge, the army as the lead's map sprite
//! (no flag), and the battle map's picture cursor (thick marks with a dark
//! edge). The panel, a list and the help line are the glyph look's.

use trpg_core::{CharacterId, ClassId, Pos};

use super::glyph::{CELL, LIST, LOCKED, PLATE_PX, level_ink, origin, paint_panel, plate, plate_px};
use super::view::{PlaceMark, WorldMapView};
use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Overlay, PxRect, Sprite};
use crate::map_view::GAME_TILESET;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::widgets::menu::glyph as menu_glyph;

/// A road's dots: how far apart, how big, and their edge.
const DOT_STEP: i32 = 8;
const DOT: i32 = 4;
/// The cursor's marks: thick, long, and inside a dark edge (as the battle
/// map's at 1×).
const MARK: (i32, i32) = (2, 4);
/// The lead's map sprite is drawn at twice its size.
const SPRITE_SCALE: i32 = 2;

fn i32_of(v: u32) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

/// Paints `view` over the whole of `buf`, the map's ground being the
/// picture at image path `picture`.
pub fn paint(ctx: &Ctx, view: &WorldMapView, picture: &str, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let named = |n: &str| ctx.palette.lookup(n).unwrap_or_else(|| c(UiColor::Text));
    let (black, ink, dark) = (c(UiColor::Black), named("panel_bg"), c(UiColor::Black));
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    let content = &ctx.game.content;
    let Some(def) = content.worldmaps.get(&view.map) else {
        return;
    };
    let (ox, oy) = origin(def);
    let on_screen = |p: Pos| Pos::new(p.x + ox, p.y + oy);
    // The symbols and level markers are glyphs over the pictures.
    for place in &view.places {
        let p = on_screen(place.pos);
        let (color, symbol) = plate(place.mark);
        let color = if place.locked { LOCKED } else { color };
        let fg = if place.mark == PlaceMark::Empty {
            named(color)
        } else {
            ink
        };
        buf.set(p.x, p.y, Cell::new(symbol, fg, black));
        if let Some(level) = &place.level {
            let len = i32::try_from(level.chars().count()).unwrap_or(0);
            let x = p.x - (len - 1) / 2;
            buf.print(x, p.y - 1, level, named(level_ink(place.mark)), black);
        }
    }
    let (cw, ch) = CELL;
    if let Some(id) = content.images.id(picture)
        && let Some(info) = content.images.info(id)
    {
        let src = PxRect::new(0, 0, i32_of(info.width), i32_of(info.height));
        let dest = PxRect::new(ox * cw, oy * ch, def.size.0 * cw, def.size.1 * ch);
        buf.add_sprite(Sprite::new(id, src, dest, Layer::Under));
    }
    // Roads not open yet in dim dots, then the open ones.
    let centre = |p: Pos| {
        let p = on_screen(p);
        (p.x * cw + cw / 2, p.y * ch + ch / 2)
    };
    paint_dots(buf, &view.locked_roads, &centre, named(LOCKED), dark);
    paint_dots(buf, &view.roads, &centre, c(UiColor::Path), dark);
    for place in view.places.iter().filter(|p| p.mark != PlaceMark::Empty) {
        let (color, _) = plate(place.mark);
        let color = if place.locked { LOCKED } else { color };
        let rect = plate_px(on_screen(place.pos), PLATE_PX.0);
        let edge = PxRect::new(rect.x - 1, rect.y - 1, rect.w + 2, rect.h + 2);
        buf.add_overlay(Overlay::new(edge, dark, Layer::Under));
        buf.add_overlay(Overlay::new(rect, named(color), Layer::Under));
    }
    let army = plate_px(on_screen(view.army), PLATE_PX.0);
    paint_army(ctx, army, buf);
    paint_cursor(
        ctx,
        plate_px(on_screen(view.cursor), PLATE_PX.0),
        view.cursor_brightness,
        buf,
    );
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

/// A dot every [`DOT_STEP`] pixels along each of `roads`, from turn to
/// turn (`centre` gives a cell's middle in pixels), in `color` with a
/// `dark` edge.
fn paint_dots(
    buf: &mut GlyphBuffer,
    roads: &[Vec<Pos>],
    centre: &dyn Fn(Pos) -> (i32, i32),
    color: Rgb,
    dark: Rgb,
) {
    for pair in roads.iter().flat_map(|r| r.windows(2)) {
        let ((ax, ay), (bx, by)) = (centre(pair[0]), centre(pair[1]));
        let (dx, dy) = ((bx - ax).signum(), (by - ay).signum());
        let len = (bx - ax).abs() + (by - ay).abs();
        for t in (0..=len).step_by(usize::try_from(DOT_STEP).unwrap_or(1)) {
            let (x, y) = (ax + dx * t, ay + dy * t);
            let edge = PxRect::new(x - DOT / 2 - 1, y - DOT / 2 - 1, DOT + 2, DOT + 2);
            buf.add_overlay(Overlay::new(edge, dark, Layer::Under));
            let fill = PxRect::new(x - DOT / 2, y - DOT / 2, DOT, DOT);
            buf.add_overlay(Overlay::new(fill, color, Layer::Under));
        }
    }
}

/// The army: the lead's map sprite standing on its place's plate (its
/// first frame, facing down), or a blue plate when the game has no unit
/// pictures.
fn paint_army(ctx: &Ctx, plate: PxRect, buf: &mut GlyphBuffer) {
    let content = &ctx.game.content;
    let lead = &ctx.game.lead;
    let tileset = content.tilesets.get(GAME_TILESET);
    let Some(tileset) = tileset else {
        let player = ctx.palette.get(UiColor::Player);
        buf.add_overlay(Overlay::new(plate, player, Layer::Under));
        return;
    };
    let ids = [
        CharacterId(lead.portrait_for(trpg_core::LEAD_ID).to_owned()),
        CharacterId(trpg_core::LEAD_ID.to_owned()),
    ];
    let picture = tileset.unit_picture(&ids, &ClassId(String::new()));
    let frame = tileset.walk_frame(picture, 0, 0).unwrap_or(picture);
    let (w, h) = (i32_of(frame.rect.w), i32_of(frame.rect.h));
    let src = PxRect::new(i32_of(frame.rect.x), i32_of(frame.rect.y), w, h);
    let (dw, dh) = (w * SPRITE_SCALE, h * SPRITE_SCALE);
    let dest = PxRect::new(
        plate.x + plate.w / 2 - dw / 2,
        plate.y + plate.h - dh,
        dw,
        dh,
    );
    buf.add_sprite(Sprite::new(frame.image, src, dest, Layer::Over));
}

/// The cursor round `rect`: at each corner two thick arms inside a dark
/// edge, the arms pulsing with `brightness`, the edge not.
fn paint_cursor(ctx: &Ctx, rect: PxRect, brightness: f32, buf: &mut GlyphBuffer) {
    let dark = ctx.palette.get(UiColor::Black);
    let mark = ctx
        .palette
        .get(UiColor::Cursor)
        .lerp(dark, 1.0 - brightness);
    let (t, l) = MARK;
    let corners = [
        (rect.x - 1, rect.y - 1, 1, 1),
        (rect.x + rect.w + 1, rect.y - 1, -1, 1),
        (rect.x - 1, rect.y + rect.h + 1, 1, -1),
        (rect.x + rect.w + 1, rect.y + rect.h + 1, -1, -1),
    ];
    for (x, y, sx, sy) in corners {
        let arm = |len: i32, thick: i32, grow: i32, across: bool| {
            let (w, h) = if across { (len, thick) } else { (thick, len) };
            let (w, h) = (w + grow * 2, h + grow * 2);
            let x0 = if sx == 1 { x - grow } else { x - w + grow };
            let y0 = if sy == 1 { y - grow } else { y - h + grow };
            PxRect::new(x0, y0, w, h)
        };
        for across in [true, false] {
            buf.add_overlay(Overlay::new(arm(l, t, 1, across), dark, Layer::Over));
        }
        for across in [true, false] {
            buf.add_overlay(Overlay::new(arm(l, t, 0, across), mark, Layer::Over));
        }
    }
}
