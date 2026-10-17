//! The combat stage's look (ADR-0054, ADR-0056): the backdrop and the two
//! pictures as sprite items at double size over the map and the side
//! panel, a plate per fighter under them, the strike's result over its
//! target.

use crate::prelude::*;
use trpg_content::Battler;
use trpg_content::battler::Facing;
use trpg_core::StatKind;

use super::motion::Pose;
use super::view::{FighterView, StageView};
use crate::color::{Rgb, UiColor};
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Layer, Paint, PxRect, Rect, Sprite};
use crate::portrait::dim_opacity;
use crate::screen::Ctx;
use crate::screens::battle::info::stat_name;
use crate::screens::battle::layout::{MAP_VIEW, SIDE_PANEL};
use crate::screens::battle::units::{faction_color, hp_fill};

/// Console pixels per picture pixel.
pub const ZOOM: u32 = 2;
/// The whole stage, in cells: the map view and the side panel, which says
/// nothing during a fight that the plates don't.
pub const STAGE: Rect = Rect::new(
    MAP_VIEW.x,
    MAP_VIEW.y,
    MAP_VIEW.w + SIDE_PANEL.w,
    MAP_VIEW.h,
);
/// Rows of the pictures' part of the stage, from its top.
pub const SCENE_ROWS: i32 = 24;
/// The pictures' part of the stage, in cells.
pub const SCENE: Rect = Rect::new(STAGE.x, STAGE.y, STAGE.w, SCENE_ROWS);
/// The part of a backdrop that is shown, in its own pixels: its whole
/// width. At [`ZOOM`] it is as tall as [`SCENE`] and [`EDGE_PX`] narrower
/// on each side.
pub const BACKDROP_SRC: PxRect = PxRect::new(0, 40, 336, 192);
/// The strip of the backdrop that is drawn again, mirrored, beside each of
/// its edges to fill the scene's width, in backdrop pixels.
pub const EDGE_PX: i32 = 32;
/// Where the fighters' feet are, in console pixels down from the scene's
/// top.
pub const FEET_Y: i32 = 276;
/// The middle of the left and the right fighter, in console pixels from
/// the scene's left edge.
pub const MIDDLES: [i32; 2] = [250, 550];
/// How far a striker's picture overlaps its target's when it arrives.
pub const OVERLAP_PX: i32 = 24;
/// How far a striker steps forward when it doesn't fight hand to hand.
pub const STEP_PX: i32 = 32;
/// How far a critical's striker leans back first.
pub const WIND_UP_PX: f32 = 16.0;
/// How far a missed target steps back.
pub const DODGE_PX: f32 = 28.0;
/// How far a hit target is shaken to each side.
pub const SHAKE_PX: f32 = 5.0;
/// How far a falling fighter sinks.
pub const SINK_PX: f32 = 12.0;
/// How solid the white over the stage is at a critical.
pub const VEIL_OPACITY: u8 = 150;
/// A plate's size in cells.
pub const PLATE: (i32, i32) = (STAGE.w / 2, STAGE.h - SCENE_ROWS);
/// Cells of a plate's HP bar.
pub const BAR_CELLS: i32 = 20;

/// A cell's size in console pixels: the battle screen is laid out on the
/// PC console.
const CELL_W: i32 = ConsoleSpec::PC.cell_w as i32;
const CELL_H: i32 = ConsoleSpec::PC.cell_h as i32;

/// The cells `cells` in console pixels.
fn cells_px(cells: Rect) -> PxRect {
    PxRect::new(
        cells.x * CELL_W,
        cells.y * CELL_H,
        cells.w * CELL_W,
        cells.h * CELL_H,
    )
}

/// The scene in console pixels.
fn scene_px() -> PxRect {
    cells_px(SCENE)
}

/// The backdrop `image` across the scene: its window in the middle, and
/// the strip at each of its edges again, mirrored, outside it (a mirrored
/// edge continues the picture without a seam).
pub fn backdrop_sprites(image: trpg_content::ImageId) -> [Sprite; 3] {
    let (scene, src) = (scene_px(), BACKDROP_SRC);
    let zoom = int(ZOOM);
    let edge = EDGE_PX * zoom;
    let middle = PxRect::new(scene.x + edge, scene.y, src.w * zoom, scene.h);
    let strip = |src_x: i32, dest_x: i32| {
        let mut s = Sprite::new(
            image,
            PxRect::new(src_x, src.y, EDGE_PX, src.h),
            PxRect::new(dest_x, scene.y, edge, scene.h),
            Layer::Under,
        );
        s.flip_x = true;
        s
    };
    [
        Sprite::new(image, src, middle, Layer::Under),
        strip(src.x, scene.x),
        strip(src.x + src.w - EDGE_PX, middle.x + middle.w),
    ]
}

/// A whole number of pixels for `v` (a bad number counts as 0).
fn px(v: f32) -> i32 {
    #[allow(clippy::cast_possible_truncation)]
    let whole = v.round() as i32;
    whole
}

/// `v` as an `i32` (pictures are small).
fn int(v: u32) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

/// A picture's size on the stage, in console pixels.
fn size_px(picture: &Battler) -> (i32, i32) {
    let (w, h) = picture.scaled(ZOOM);
    (int(w), int(h))
}

/// How far fighter `i`'s lunge takes it when it reaches its opponent, in
/// console pixels: for hand to hand, until the pictures overlap by
/// [`OVERLAP_PX`]; otherwise [`STEP_PX`].
pub fn reach_px(view: &StageView, i: usize) -> i32 {
    if !view.melee {
        return STEP_PX;
    }
    let width = |f: &FighterView| f.picture.as_ref().map_or(0, |p| size_px(p).0);
    let widths = width(&view.fighters[i]) + width(&view.fighters[1 - i]);
    let apart = MIDDLES[1] - MIDDLES[0];
    (apart - widths / 2 + OVERLAP_PX).max(STEP_PX)
}

/// How far `pose` moves a picture toward its opponent (negative: away)
/// and down, in console pixels, when a full lunge is `reach` pixels.
pub fn offset_px(pose: &Pose, reach: i32) -> (i32, i32) {
    #[allow(clippy::cast_precision_loss)] // a few hundred pixels
    let lunge = if pose.lunge >= 0.0 {
        pose.lunge * reach as f32
    } else {
        pose.lunge * WIND_UP_PX
    };
    let toward = lunge - pose.dodge * DODGE_PX - pose.shake * SHAKE_PX;
    (px(toward), px(pose.fade * SINK_PX))
}

/// Fighter `i`'s picture as a sprite, if it has one: feet on the ground
/// line, turned toward its opponent, moved by its pose.
pub fn fighter_sprite(view: &StageView, i: usize, white: Rgb) -> Option<Sprite> {
    let fighter = view.fighters.get(i)?;
    let picture = fighter.picture.as_ref()?;
    let on_left = view.left == i;
    let (w, h) = size_px(picture);
    let (toward, down) = offset_px(&fighter.pose, reach_px(view, i));
    let scene = scene_px();
    let middle = MIDDLES[usize::from(!on_left)];
    let dx = if on_left { toward } else { -toward };
    let dest = PxRect::new(
        scene.x + middle - w / 2 + dx,
        scene.y + FEET_Y - h + down,
        w,
        h,
    );
    let src = PxRect::new(0, 0, int(picture.size.width), int(picture.size.height));
    let mut sprite = Sprite::new(picture.image, src, dest, Layer::Under);
    sprite.clip = dest.intersect(&scene)?;
    // The left fighter looks right, the right one left.
    sprite.flip_x = (picture.faces == Facing::Right) != on_left;
    sprite.opacity = dim_opacity(fighter.pose.fade);
    if fighter.pose.flash {
        sprite.paint = Paint::Solid(white);
    }
    Some(sprite)
}

/// The row of the result over fighter `i`: the one above its picture's
/// top (never above the scene's second row).
pub fn result_row(view: &StageView, i: usize) -> i32 {
    let height = view.fighters[i]
        .picture
        .as_ref()
        .map_or(0, |p| size_px(p).1);
    SCENE.y + ((FEET_Y - height) / CELL_H - 1).max(1)
}

/// `text` in a label `w` cells wide at most, centred on console pixel
/// column `middle` (from the scene's left edge), kept inside the scene.
fn label(buf: &mut GlyphBuffer, middle: i32, y: i32, text: &str, fg: Rgb, bg: Rgb) {
    let width = usize::try_from(SCENE.w - 2).unwrap_or(0);
    let cut: String = text.chars().take(width).collect();
    let w = i32::try_from(cut.chars().count()).unwrap_or(0) + 2;
    let x = (SCENE.x + middle / CELL_W - w / 2).clamp(SCENE.x, SCENE.x + SCENE.w - w);
    buf.fill_rect(Rect::new(x, y, w, 1), Cell::new(' ', fg, bg));
    buf.print(x + 1, y, &cut, fg, bg);
}

/// The plate of the fighter in `slot` (0 left, 1 right): its name (lit
/// when it is about to strike), HP with a bar, and its numbers.
fn plate(ctx: &Ctx, buf: &mut GlyphBuffer, slot: i32, fighter: &FighterView) {
    let c = |u| ctx.palette.get(u);
    let words = ctx.words();
    let (bg, dim) = (c(UiColor::PanelBg), c(UiColor::TextDim));
    let color = c(faction_color(fighter.faction));
    let rect = Rect::new(
        STAGE.x + slot * PLATE.0,
        SCENE.y + SCENE.h,
        PLATE.0,
        PLATE.1,
    );
    buf.draw_box(rect, BoxStyle::Single, color, bg);
    let (x, y) = (rect.x + 2, rect.y + 1);
    let inner = usize::try_from(rect.w - 4).unwrap_or(0);
    let cut = |text: &str| -> String { text.chars().take(inner).collect() };
    let (fg, name_bg) = if fighter.lit {
        (bg, color)
    } else {
        (color, bg)
    };
    // check-text: not a data name (the view's own)
    buf.print(x, y, &cut(&fighter.name), fg, name_bg);
    let hp = format!("{} {:>2}", stat_name(words, StatKind::Hp), fighter.hp);
    buf.print(x, y + 1, &hp, c(UiColor::Text), bg);
    let (filled, bar) = hp_fill(fighter.hp, fighter.max, BAR_CELLS);
    for i in 0..BAR_CELLS {
        let (glyph, fg) = if i < filled {
            ('█', c(bar))
        } else {
            ('░', dim)
        };
        buf.set(x + 6 + i, y + 1, Cell::new(glyph, fg, bg));
    }
    buf.print(
        x + 7 + BAR_CELLS,
        y + 1,
        &format!("/{}", fighter.max),
        dim,
        bg,
    );
    let numbers = match fighter.numbers {
        Some(n) => {
            let key = if n.heals {
                "battle.forecast.strike_heal"
            } else {
                "battle.forecast.strike"
            };
            format!(
                "{}  {} {}  {} {}",
                words.text_with(key, &[("n", &n.damage)]),
                words.text("battle.forecast.hit"),
                n.hit,
                words.text("battle.forecast.crit"),
                n.crit,
            )
        }
        None => words.text("battle.forecast.no_counter").to_owned(),
    };
    buf.print(x, y + 3, &cut(&numbers), dim, bg);
}

/// Paints `view` over the map and the side panel.
pub fn paint(ctx: &Ctx, view: &StageView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (bg, white) = (c(UiColor::PanelBg), c(UiColor::White));
    buf.fill_rect(STAGE, Cell::new(' ', c(UiColor::Text), bg));
    let scene = scene_px();
    if let Some(backdrop) = view.backdrop {
        let paint = if view.shade.is_plain() {
            Paint::Image
        } else {
            Paint::Shaded(view.shade)
        };
        for sprite in backdrop_sprites(backdrop) {
            buf.add_sprite(sprite.painted(paint));
        }
    }
    // The striker last: in front of its target when they meet.
    let behind = view.striker.map_or(0, |s| 1 - s);
    for i in [behind, 1 - behind] {
        if let Some(sprite) = fighter_sprite(view, i, white) {
            buf.add_sprite(sprite);
        }
    }
    if let Some(backdrop) = view.backdrop.filter(|_| view.veil) {
        for sprite in backdrop_sprites(backdrop) {
            let mut veil = sprite.painted(Paint::Solid(white));
            veil.opacity = VEIL_OPACITY;
            buf.add_sprite(veil);
        }
    }
    let highlight = c(UiColor::TextHighlight);
    if let Some(name) = &view.banner {
        label(buf, scene.w / 2, SCENE.y, name, highlight, bg);
    }
    if let Some((on, text)) = &view.result {
        let slot = usize::from(view.left != *on);
        label(
            buf,
            MIDDLES[slot],
            result_row(view, *on),
            text,
            highlight,
            bg,
        );
    }
    plate(ctx, buf, 0, &view.fighters[view.left]);
    plate(ctx, buf, 1, &view.fighters[1 - view.left]);
}

#[cfg(test)]
mod tests;
