//! Drawing portraits (ADR-0043): each is one sprite item, the expression's
//! image at the largest whole scale that fits the 32×16-cell frame, centred.
//! Fitting a picture to an area is Shuyi Engine's
//! ([`shuyi_ui::frame::portrait`], ticket 1108).

pub use shuyi_ui::frame::portrait::{dim_opacity, draw_fitted, fit_whole_scale};
use trpg_content::Portrait;

use crate::glyph_buffer::GlyphBuffer;

/// The space a portrait is drawn in, in cells: 256×256 console pixels
/// (`trpg_content::portrait::FRAME_PX`).
pub const FRAME_CELLS: (i32, i32) = (32, 16);

/// Draws expression `expr` of `portrait` in the [`FRAME_CELLS`] area whose
/// top-left cell is `(x, y)`, as one sprite under the glyphs: a transparent
/// pixel shows the cells' background, so clear the area first. `dim` (0 =
/// full colour, 1 = gone) fades it toward that background; `mirror` flips
/// it left to right. Clipped to the buffer. Returns `false`, drawing
/// nothing, when the portrait has no such expression.
pub fn draw_portrait(
    buf: &mut GlyphBuffer,
    at: (i32, i32),
    portrait: &Portrait,
    expr: &str,
    dim: f32,
    mirror: bool,
) -> bool {
    draw_portrait_in(buf, at, FRAME_CELLS, portrait, expr, dim, mirror)
}

/// [`draw_portrait`] in an area of `(w, h)` cells instead of
/// [`FRAME_CELLS`]: at the largest whole scale that fits it (a small face
/// in the dialogue log, ticket 0729). Draws nothing (and returns `true`)
/// if the picture is too big for the area even at 1×.
pub fn draw_portrait_in(
    buf: &mut GlyphBuffer,
    at: (i32, i32),
    cells: (i32, i32),
    portrait: &Portrait,
    expr: &str,
    dim: f32,
    mirror: bool,
) -> bool {
    let Some(expression) = portrait.expression(expr) else {
        return false;
    };
    let picture = (expression.image, expression.size);
    draw_fitted(buf, at, cells, picture, dim, mirror);
    true
}

#[cfg(test)]
mod tests;
