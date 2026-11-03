//! The Playdate's look of the title screen (ticket 1044,
//! `docs/design/playdate.md`, *The title cinematic*): paints a
//! [`TitleView`] on the 400 × 240 one-bit screen, in the look of
//! [`crate::screens::playdate`].
//!
//! - **The backdrop** is the launch's title backdrop, lightened and
//!   dithered ahead of time like the launcher card's (1040), at the
//!   screen's width, its middle rows shown above the bottom row.
//! - **The logo** is the card's: white letters, a black shadow and a black
//!   rim, [`LETTER_BLOCK`] pixels a block (`trpg_content::playdate_images`,
//!   [`lettering`](trpg_content::playdate_images::lettering)), at the top.
//! - **The menu** is a window over the backdrop, its bottom on the row over
//!   the help line; the "press any button" line is a window there too. The
//!   bottom row is white, with the help line (or the notice) on it.
//! - **One bit has no fades**: the menu shows while it is at least half
//!   solid, and the cinematic once it is at least half bright, so each
//!   cuts in at the middle of its fade. No sparks: dots are lost in the
//!   backdrop's dither.
//!
//! A sprite is drawn over the cells' backgrounds, so the backdrop and the
//! logo are clipped around the menu's window ([`around`]).

use trpg_content::ImageId;
use trpg_content::battler::BACKDROP_PX;

use super::view::{FULL, TitleView};
use crate::glyph_buffer::{GlyphBuffer, Layer, PxRect, Rect, Sprite};
use crate::map_view::grid::px_rect;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::playdate::stage::{BACKDROP_SCALE, enlarged};
use crate::screens::paint_debug_hint;
use crate::screens::playdate::{self as pd, cells, centred, clear, menu_size};

/// Screen pixels per block of the logo: its one-bit picture's
/// (`trpg_content::playdate_images::LETTER_BLOCK`).
pub const LETTER_BLOCK: i32 = 3;
/// Pixels above the logo's rim.
pub const LOGO_TOP_PX: i32 = 2;

/// The screen's pixels, with the `bottom` row of cells left out: where the
/// title's picture is.
fn picture_area(buf: &GlyphBuffer) -> PxRect {
    let bottom = i32::from(buf.height()) - 1;
    px_rect(Rect::new(0, 0, i32::from(buf.width()), bottom))
}

/// `v` pixels as a coordinate.
fn int(v: u32) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

/// The parts of `area` outside `hole` (each a rectangle; all of `area`
/// with no hole).
pub fn around(area: PxRect, hole: Option<PxRect>) -> Vec<PxRect> {
    let Some(hole) = hole else {
        return vec![area];
    };
    let (top, bottom) = (hole.y, hole.y + hole.h);
    let (left, right) = (hole.x, hole.x + hole.w);
    let parts = [
        PxRect::new(area.x, area.y, area.w, top - area.y),
        PxRect::new(area.x, bottom, area.w, area.y + area.h - bottom),
        PxRect::new(area.x, top, left - area.x, hole.h),
        PxRect::new(right, top, area.x + area.w - right, hole.h),
    ];
    parts.iter().filter_map(|p| p.intersect(&area)).collect()
}

/// `sprite` drawn in `area` but for `hole`: a copy clipped to each part.
fn clipped(sprite: Sprite, area: PxRect, hole: Option<PxRect>) -> Vec<Sprite> {
    around(area, hole)
        .into_iter()
        .filter_map(|part| {
            let mut s = sprite;
            s.clip = sprite.clip.intersect(&part)?;
            Some(s)
        })
        .collect()
}

/// The backdrop `image` over `area` (the screen's width): its one-bit
/// picture, enlarged to that width, centred down it.
pub fn backdrop_sprite(image: ImageId, area: PxRect) -> Sprite {
    let (w, h) = BACKDROP_PX;
    let (dest_w, dest_h) = (enlarged(w, BACKDROP_SCALE), enlarged(h, BACKDROP_SCALE));
    let top = area.y + (area.h - dest_h) / 2;
    let dest = PxRect::new(area.x, top, dest_w, dest_h);
    let src = PxRect::new(0, 0, int(w), int(h));
    Sprite::new(image, src, dest, Layer::Under)
}

/// The logo `image` of `size` (its PNG's pixels), centred at the top: its
/// one-bit picture, a block of rim round the PNG, drawn whole.
pub fn logo_sprite(image: ImageId, size: (u32, u32), screen_w: i32) -> Sprite {
    let drawn = |blocks: u32| (int(blocks) + 2) * LETTER_BLOCK;
    let (w, h) = (drawn(size.0), drawn(size.1));
    let src = PxRect::new(0, 0, int(size.0), int(size.1));
    let dest = PxRect::new((screen_w - w) / 2, LOGO_TOP_PX, w, h);
    Sprite::new(image, src, dest, Layer::Under)
}

/// Paints the title's picture over the whole of `buf`: the white ground,
/// `backdrop` (if any) and the logo above the bottom row, neither over
/// `hole` (a window's pixels). The title screen draws its menu or prompt
/// over this; the cinematic's logo shot is this alone.
pub fn paint_logo(
    ctx: &Ctx,
    backdrop: Option<ImageId>,
    hole: Option<PxRect>,
    buf: &mut GlyphBuffer,
) {
    clear(ctx, buf);
    let area = picture_area(buf);
    let back = backdrop.map(|image| backdrop_sprite(image, area));
    let logo = ctx.game.content.title.logo;
    let logo = logo.map(|(image, size)| logo_sprite(image, size, area.w));
    for sprite in back.into_iter().chain(logo) {
        for part in clipped(sprite, area, hole) {
            buf.add_sprite(part);
        }
    }
}

/// What the title shows in a window over its picture.
enum Shown<'a> {
    /// The menu.
    Menu(&'a crate::widgets::MenuView),
    /// The "press any button" line.
    Prompt(&'a str),
}

/// The window of `shown`: centred, its bottom on the row over `buf`'s
/// bottom row.
fn window_of(ctx: &Ctx, buf: &GlyphBuffer, shown: &Shown<'_>) -> Rect {
    let (w, h) = match shown {
        Shown::Menu(menu) => menu_size(ctx, menu),
        Shown::Prompt(text) => (cells(text) + 4, 3),
    };
    let w = w.min(i32::from(buf.width()));
    let bottom = i32::from(buf.height()) - 1;
    Rect::new((i32::from(buf.width()) - w) / 2, bottom - h, w, h)
}

/// Paints `view` over the whole of `buf`: the cinematic while it is at
/// least half bright, else the title's picture, then (while it is at least
/// half solid) the menu or the prompt over the backdrop and the help line
/// or the notice on the bottom row.
pub fn paint(ctx: &Ctx, view: &TitleView, buf: &mut GlyphBuffer) {
    let half = FULL / 2;
    let bottom = i32::from(buf.height()) - 1;
    let solid = view.menu_opacity > half;
    let menu = view.menu.as_ref().map(Shown::Menu);
    let prompt = view.prompt.as_deref().map(Shown::Prompt);
    let shown = menu.or(prompt).filter(|_| solid);
    let window = shown.as_ref().map(|s| window_of(ctx, buf, s));
    match &view.cinematic {
        Some(shot) if shot.brightness > half => {
            #[allow(clippy::cast_precision_loss)] // far below 2^24 ms
            let t = shot.t_ms as f32 / 1000.0;
            shot.player.draw(ctx, t, 1.0, buf);
        }
        _ => paint_logo(ctx, view.backdrop, window.map(px_rect), buf),
    }
    match (shown, window) {
        (Some(Shown::Menu(m)), Some(rect)) => pd::menu(ctx, buf, (rect.x, rect.y), m),
        (Some(Shown::Prompt(text)), Some(rect)) => {
            pd::window(ctx, buf, rect, "");
            let ink = pd::inks(ctx);
            let text = pd::cut(text, pd::to_usize(rect.w - 4));
            buf.print(rect.x + 2, rect.y + 1, &text, ink.text, ink.inside);
        }
        _ => {}
    }
    let line = view.notice.as_ref().or(view.help.as_ref());
    if let Some(line) = line.filter(|_| solid) {
        centred(ctx, buf, bottom, line);
    }
    if let Some(hint) = &view.debug_hint {
        paint_debug_hint(&ctx.palette, hint, buf, bottom);
    }
}

#[cfg(test)]
mod tests;
