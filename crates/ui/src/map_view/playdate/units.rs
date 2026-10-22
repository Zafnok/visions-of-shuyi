//! How the Playdate's map skin draws a unit (`docs/design/playdate.md`,
//! *Telling things apart without colour*): its two-letter label white in a
//! black tile (yours), with a white edge on a grey tile (an enemy), or
//! black in an outlined tile once it has acted (or is down, 0319: the
//! one-bit screen's grayed out); the HP line along the
//! bottom of the tile, as on the PC, its HP lost dithered on yours.

use trpg_core::Faction;

use super::{FLASH_SHOWN, Inks};
use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Overlay, PxRect, Rect};
use crate::map_view::glyph::TILE_W_CELLS;
use crate::map_view::glyph::units::{HP_BAR_H, HP_BAR_INSET, HP_BAR_W, hp_bar};
use crate::map_view::grid::CELL_PX;
use crate::map_view::scene::UnitView;

/// How far through its fall (0404) a unit is gone, for a screen with no
/// fading: the glyph skin's letters have faded out by then.
pub const GONE: f32 = 0.5;

/// How a unit's tile looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    /// The tile's fill, or `None` to keep the terrain's.
    pub tile: Option<Rgb>,
    /// The letters' ink.
    pub letters: Rgb,
    /// The outline round the tile, if it has one.
    pub outline: Option<Rgb>,
}

/// How `unit` looks in `inks`. Allies and neutral units (none in Chapter 1)
/// are their letters alone on the terrain until a chapter that has them
/// decides (`docs/design/playdate.md`, *Open sub-questions*).
pub fn look(unit: &UnitView, inks: &Inks) -> Look {
    if unit.acted || unit.is_down() {
        let (tile, letters, outline) = inks.acted;
        return Look {
            tile: Some(tile),
            letters,
            outline: Some(outline),
        };
    }
    let solid = |(tile, letters): (Rgb, Rgb)| Look {
        tile: Some(tile),
        letters,
        outline: None,
    };
    match unit.faction {
        Faction::Player => solid(inks.player),
        Faction::Enemy => solid(inks.enemy),
        Faction::Ally | Faction::Neutral => Look {
            tile: None,
            letters: inks.other,
            outline: None,
        },
    }
}

/// Whether a share (a fall's [`fade`](UnitView::fade), a blow's
/// [`flash`](UnitView::flash)) is at least `at`; `false` if it isn't a
/// number.
fn reached(share: f32, at: f32) -> bool {
    share.is_finite() && share >= at
}

/// Draws `unit` on the tile whose left cell is `(x, y)`, its items clipped
/// to `clip`: its tile and letters ([`look`]), its outline under the
/// letters, then its HP line (`hp_line`). A unit a battle note is about
/// ([`highlight`](UnitView::highlight)), or one a blow just landed on, is
/// flipped. One past the middle of its fall is gone: the terrain shows.
pub fn draw_unit(
    buf: &mut GlyphBuffer,
    palette: &Palette,
    inks: &Inks,
    unit: &UnitView,
    (x, y): (i32, i32),
    clip: PxRect,
) {
    if reached(unit.fade, GONE) {
        return;
    }
    let look = look(unit, inks);
    let mut label = unit.label.chars();
    for i in 0..TILE_W_CELLS {
        let glyph = label.next().unwrap_or(' ');
        if let Some(&cell) = buf.get(x + i, y) {
            let bg = look.tile.unwrap_or(cell.bg);
            buf.set(x + i, y, Cell::new(glyph, look.letters, bg));
        }
    }
    let (cw, ch) = CELL_PX;
    let tile = Rect::new(x * cw, y * ch, TILE_W_CELLS * cw, ch);
    let mut add = |rect: PxRect, color: Rgb, layer: Layer| {
        if let Some(rect) = rect.intersect(&clip) {
            buf.add_overlay(Overlay::new(rect, color, layer));
        }
    };
    if let Some(outline) = look.outline {
        for side in outline_sides(tile) {
            add(side, outline, Layer::Under);
        }
    }
    if unit.marks {
        for (rect, color) in hp_line(unit, &look, tile, palette, inks) {
            add(rect, color, Layer::Over);
        }
    }
    if unit.highlight || reached(unit.flash, FLASH_SHOWN) {
        add(tile, inks.highlight, Layer::Over);
    }
}

/// The four 1-pixel sides of `tile`: top, bottom, left, right.
fn outline_sides(tile: PxRect) -> [PxRect; 4] {
    let (right, bottom) = (tile.x + tile.w - 1, tile.y + tile.h - 1);
    [
        Rect::new(tile.x, tile.y, tile.w, 1),
        Rect::new(tile.x, bottom, tile.w, 1),
        Rect::new(tile.x, tile.y, 1, tile.h),
        Rect::new(right, tile.y, 1, tile.h),
    ]
}

/// `unit`'s HP line on `tile`, along its bottom a pixel in from each side,
/// as long as the HP left (the glyph skin's bar): on an enemy's tile (and a
/// neutral unit's) in its HP colour with the HP lost in the console's
/// ground, black then white in the Playdate's palette; on your unit's tile
/// (and an ally's) in its letters' ink with the HP lost dithered grey, so
/// the whole line shows on its black tile (Nick, 1037: "A for enemy, but C
/// for ally"). A part with no width is nothing (`add_overlay` drops it).
fn hp_line(
    unit: &UnitView,
    look: &Look,
    tile: PxRect,
    palette: &Palette,
    inks: &Inks,
) -> [(PxRect, Rgb); 2] {
    let (left, color) = hp_bar(unit.hp.0, unit.hp.1);
    let (x, y) = (tile.x + HP_BAR_INSET, tile.y + tile.h - HP_BAR_H);
    let (left_ink, lost_ink) = match unit.faction {
        Faction::Player | Faction::Ally => (look.letters, inks.hp_lost),
        Faction::Enemy | Faction::Neutral => (palette.get(color), palette.get(UiColor::Black)),
    };
    [
        (Rect::new(x, y, left, HP_BAR_H), left_ink),
        (Rect::new(x + left, y, HP_BAR_W - left, HP_BAR_H), lost_ink),
    ]
}

#[cfg(test)]
mod tests;
