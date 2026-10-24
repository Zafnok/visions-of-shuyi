//! The Playdate's look of the combat stage (ticket 1041, ADR-0054,
//! `docs/design/playdate.md`, *The combat picture*): the PC's
//! [`StageView`], painted on the one-bit screen as 0062 decided. The
//! backdrop shows only above a black horizon line; below it the ground is
//! plain white, and the two fighters stand on it at the PC's double size,
//! each with the white rim its one-bit picture was made with
//! (`trpg_content::playdate_images`). Their plates sit along the bottom,
//! the striker's double edged, and the result pops up over the target in
//! a white box with a black edge. The motions are the PC's
//! ([`offset_px`]), at the same size: a fighter is as many pixels big
//! here as on the PC's console.

use crate::prelude::*;
use trpg_content::battler::{BACKDROP_PX, Facing};
use trpg_content::{Battler, Ink};
use trpg_core::StatKind;

use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{
    BoxStyle, Cell, GlyphBuffer, Layer, Overlay, Paint, PxRect, Rect, Sprite,
};
use crate::map_view::grid::px_rect;
use crate::portrait::dim_opacity;
use crate::screen::Ctx;
use crate::screens::battle::info::stat_name;
use crate::screens::battle::stage::glyph::{OVERLAP_PX, STEP_PX, offset_px};
use crate::screens::battle::stage::view::{FighterView, StageView};
use crate::screens::playdate::{self as pd, cells, cut, inks, to_usize};

/// Rows of a plate: its frame, with the name and HP in its top, and the
/// fighter's numbers on the row between.
pub const PLATE_ROWS: i32 = 3;
/// How far below the scene's top the horizon line is, in pixels
/// (*tunable*: 110 in 0062's mockup).
pub const HORIZON_Y: i32 = 110;
/// The row of the enlarged backdrop at the scene's top: 0062's mockup
/// showed its rows from this one down to the horizon.
pub const BACKDROP_TOP_PX: i32 = 69;
/// How much the Playdate's picture of a backdrop is enlarged: to the
/// screen's width (`trpg_content::playdate_images`).
pub const BACKDROP_SCALE: (u32, u32) = (25, 21);
/// How far above the plates the fighters' feet are, in pixels.
pub const FEET_ABOVE_PX: i32 = 4;
/// How far each fighter's middle is from the scene's, in pixels.
pub const HALF_APART_PX: i32 = 88;

/// A cell's size in pixels.
const CELL_W: i32 = ConsoleSpec::PLAYDATE.cell_w as i32;
const CELL_H: i32 = ConsoleSpec::PLAYDATE.cell_h as i32;

/// The pictures' part of `area` (the whole screen), in cells: all of it
/// above the plates.
pub fn scene(area: Rect) -> Rect {
    Rect::new(area.x, area.y, area.w, area.h - PLATE_ROWS)
}

/// Where the fighters' feet are in `scene`, in pixels down from its top.
fn feet_y(scene: PxRect) -> i32 {
    scene.h - FEET_ABOVE_PX
}

/// The middle of the left and the right fighter in `scene`, in pixels
/// from the screen's left edge.
pub fn middles(scene: PxRect) -> [i32; 2] {
    let middle = scene.x + scene.w / 2;
    [middle - HALF_APART_PX, middle + HALF_APART_PX]
}

/// `v` as an `i32` (pictures are small).
fn int(v: u32) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

/// `v` pixels enlarged `num / den` times, rounded to the nearest pixel as
/// the converter rounds (`shuyi_assets::bitmap::convert`): the size of a
/// one-bit picture made from a PNG side `v` pixels long, so each of its
/// pixels lands on one of the screen's.
fn enlarged(v: u32, (num, den): (u32, u32)) -> i32 {
    let den = den.max(1);
    int((v * num + den / 2) / den)
}

/// A fighter's picture's size on the screen: twice its sidecar's scale,
/// as its one-bit picture was made.
pub fn size_px(picture: &Battler) -> (i32, i32) {
    let (num, den) = picture.scale;
    let side = |v| enlarged(v, (2 * num, den));
    (side(picture.size.width), side(picture.size.height))
}

/// How far fighter `i`'s lunge takes it when it reaches its opponent, in
/// pixels: as on the PC, until the pictures overlap by [`OVERLAP_PX`] hand
/// to hand, else [`STEP_PX`].
pub fn reach_px(view: &StageView, i: usize) -> i32 {
    if !view.melee {
        return STEP_PX;
    }
    let width = |f: &FighterView| f.picture.as_ref().map_or(0, |p| size_px(p).0);
    let widths = width(&view.fighters[i]) + width(&view.fighters[1 - i]);
    (2 * HALF_APART_PX - widths / 2 + OVERLAP_PX).max(STEP_PX)
}

/// The backdrop `image` above the horizon of `scene` (in pixels): its
/// picture enlarged to the screen's width, from [`BACKDROP_TOP_PX`] down.
pub fn backdrop_sprite(image: trpg_content::ImageId, scene: PxRect) -> Option<Sprite> {
    let (w, h) = BACKDROP_PX;
    let (dest_w, dest_h) = (enlarged(w, BACKDROP_SCALE), enlarged(h, BACKDROP_SCALE));
    let dest = PxRect::new(scene.x, scene.y - BACKDROP_TOP_PX, dest_w, dest_h);
    let src = PxRect::new(0, 0, int(w), int(h));
    let mut sprite = Sprite::new(image, src, dest, Layer::Under);
    let above = PxRect::new(scene.x, scene.y, scene.w, HORIZON_Y);
    sprite.clip = dest.intersect(&above)?;
    Some(sprite)
}

/// Fighter `i`'s picture in `scene` (in pixels), if it has one: feet on
/// the ground, turned toward its opponent, moved by its pose, a solid
/// `flash` shape while it is hit.
pub fn fighter_sprite(view: &StageView, i: usize, scene: PxRect, flash: Rgb) -> Option<Sprite> {
    let fighter = view.fighters.get(i)?;
    let picture = fighter.picture.as_ref()?;
    let on_left = view.left == i;
    let (w, h) = size_px(picture);
    let (toward, down) = offset_px(&fighter.pose, reach_px(view, i));
    let middle = middles(scene)[usize::from(!on_left)];
    let dx = if on_left { toward } else { -toward };
    let top = scene.y + feet_y(scene) - h + down;
    let dest = PxRect::new(middle - w / 2 + dx, top, w, h);
    let src = PxRect::new(0, 0, int(picture.size.width), int(picture.size.height));
    let mut sprite = Sprite::new(picture.image, src, dest, Layer::Under);
    sprite.clip = dest.intersect(&scene)?;
    // The left fighter looks right, the right one left.
    sprite.flip_x = (picture.faces == Facing::Right) != on_left;
    sprite.opacity = dim_opacity(fighter.pose.fade);
    if fighter.pose.flash {
        sprite.paint = Paint::Solid(flash);
    }
    Some(sprite)
}

/// The row of the result over fighter `i` in `scene`: the one above its
/// picture's top, and never above `first`.
pub fn result_row(view: &StageView, i: usize, scene: Rect, first: i32) -> i32 {
    let height = view.fighters[i]
        .picture
        .as_ref()
        .map_or(0, |p| size_px(p).1);
    let top = feet_y(px_rect(scene)) - height;
    scene.y + (top.div_euclid(CELL_H) - 1).max(first)
}

/// The ink of the palette's colour `name`, or `fallback`'s (in a test on
/// the PC's palette).
fn named_ink(ctx: &Ctx, name: &str, fallback: Ink) -> Rgb {
    ctx.palette
        .lookup(name)
        .unwrap_or_else(|| Rgb::from(fallback.code()))
}

/// `text` in a box one row tall on row `y` of `scene`, centred on pixel
/// column `middle` and kept inside it: white, black letters, and a black
/// edge a pixel wide just outside its cells.
fn label(ctx: &Ctx, buf: &mut GlyphBuffer, scene: Rect, (middle, y): (i32, i32), text: &str) {
    let ink = inks(ctx);
    let shown = cut(text, to_usize(scene.w - 2));
    let w = cells(&shown) + 2;
    let x = (middle / CELL_W - w / 2).clamp(scene.x, scene.x + scene.w - w);
    let rect = Rect::new(x, y, w, 1);
    buf.fill_rect(rect, Cell::new(' ', ink.text, ink.inside));
    buf.print(x + 1, y, &shown, ink.text, ink.inside);
    let (px, inside) = (px_rect(rect), px_rect(scene));
    let edges = [
        PxRect::new(px.x - 1, px.y - 1, px.w + 2, 1),
        PxRect::new(px.x - 1, px.y + px.h, px.w + 2, 1),
        PxRect::new(px.x - 1, px.y, 1, px.h),
        PxRect::new(px.x + px.w, px.y, 1, px.h),
    ];
    for edge in edges.iter().filter_map(|e| e.intersect(&inside)) {
        buf.add_overlay(Overlay::new(edge, ink.frame, Layer::Over));
    }
}

/// The fighter's numbers for the plate, in `room` cells: as the PC's, the
/// gaps one space wide where two don't fit.
fn numbers(ctx: &Ctx, fighter: &FighterView, room: usize) -> String {
    let words = ctx.words();
    let Some(n) = fighter.numbers else {
        return words.text("battle.forecast.no_counter").to_owned();
    };
    let key = if n.heals {
        "battle.forecast.strike_heal"
    } else {
        "battle.forecast.strike"
    };
    let parts = [
        words.text_with(key, &[("n", &n.damage)]),
        format!("{} {}", words.text("battle.forecast.hit"), n.hit),
        format!("{} {}", words.text("battle.forecast.crit"), n.crit),
    ];
    let wide = parts.join("  ");
    if wide.chars().count() <= room {
        wide
    } else {
        parts.join(" ")
    }
}

/// The plate of `fighter` at `rect`: its name in the top of the frame
/// (the bar when it is about to strike) and its HP at the frame's right,
/// its numbers inside. A `striking` fighter's frame is double.
fn plate(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, fighter: &FighterView, striking: bool) {
    let ink = inks(ctx);
    buf.fill_rect(rect, Cell::new(' ', ink.text, ink.inside));
    let style = if striking {
        BoxStyle::Double
    } else {
        BoxStyle::Single
    };
    buf.draw_box(rect, style, ink.frame, ink.inside);
    let hp_name = stat_name(ctx.words(), StatKind::Hp);
    let hp = format!(" {hp_name} {}/{} ", fighter.hp, fighter.max);
    let hp_x = rect.x + rect.w - 2 - cells(&hp);
    buf.print(hp_x, rect.y, &hp, ink.text, ink.inside);
    // check-text: not a data name (the view's own)
    let name = cut(&fighter.name, to_usize(hp_x - rect.x - 4));
    if fighter.lit {
        pd::bar(ctx, buf, rect.x + 2, rect.y, cells(&name) + 2, &name);
    } else {
        buf.print(
            rect.x + 2,
            rect.y,
            &format!(" {name} "),
            ink.text,
            ink.inside,
        );
    }
    // Up to the frame, if it must.
    let room = to_usize(rect.w - 3);
    let line = cut(&numbers(ctx, fighter, room - 1), room);
    buf.print(rect.x + 2, rect.y + 1, &line, ink.text, ink.inside);
}

/// Paints `view` over `area`, the whole screen.
pub fn paint(ctx: &Ctx, view: &StageView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    buf.fill_rect(area, Cell::new(' ', ink.text, ink.ground));
    let cells_scene = scene(area);
    let scene = px_rect(cells_scene);
    if let Some(backdrop) = view.backdrop.and_then(|b| backdrop_sprite(b, scene)) {
        buf.add_sprite(backdrop);
    }
    let horizon = PxRect::new(scene.x, scene.y + HORIZON_Y, scene.w, 1);
    buf.add_overlay(Overlay::new(horizon, ink.frame, Layer::Under));
    // The striker last: in front of its target when they meet.
    let flash = named_ink(ctx, "stage_flash", Ink::Black);
    let behind = view.striker.map_or(0, |s| 1 - s);
    for i in [behind, 1 - behind] {
        if let Some(sprite) = fighter_sprite(view, i, scene, flash) {
            buf.add_sprite(sprite);
        }
    }
    if view.veil {
        let veil = named_ink(ctx, "stage_veil", Ink::Invert);
        buf.add_overlay(Overlay::new(scene, veil, Layer::Over));
    }
    if let Some(name) = &view.banner {
        label(
            ctx,
            buf,
            cells_scene,
            (scene.x + scene.w / 2, cells_scene.y),
            name,
        );
    }
    if let Some((on, text)) = &view.result {
        let first = i32::from(view.banner.is_some());
        let slot = usize::from(view.left != *on);
        let row = result_row(view, *on, cells_scene, first);
        label(ctx, buf, cells_scene, (middles(scene)[slot], row), text);
    }
    let w = area.w / 2;
    let y = cells_scene.y + cells_scene.h;
    for (slot, i) in [view.left, 1 - view.left].into_iter().enumerate() {
        let x = area.x + pd::to_i32(slot) * w;
        let rect = Rect::new(x, y, if slot == 0 { w } else { area.w - w }, PLATE_ROWS);
        plate(ctx, buf, rect, &view.fighters[i], view.striker == Some(i));
    }
}

#[cfg(test)]
mod tests;
