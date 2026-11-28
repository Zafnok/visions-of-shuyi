// engine-friction: engine#3254, moves (game 1415 deletes it)
//! The glyph look of the place line: dim text two rows over the help line,
//! centred, the path shortened from the left (`…\tactical-rpg`) so the
//! line fits the buffer.

use super::PlaceView;
use crate::color::UiColor;
use crate::glyph_buffer::GlyphBuffer;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::{clip, print_centred};
use shuyi_assets::font::CellWidths;
use shuyi_assets::lang::text_width;

/// Cells kept clear, both ends of the line together.
const SIDES: usize = 4;

/// Paints `place` two rows over the bottom (help) row of `buf`.
pub fn paint_place(ctx: &Ctx, place: &PlaceView, buf: &mut GlyphBuffer) {
    let width = usize::from(buf.width()).saturating_sub(SIDES);
    let line = place_line(&place.label, &place.path, width, buf.widths());
    let y = i32::from(buf.height()) - 3;
    let (dim, black) = (
        ctx.palette.get(UiColor::TextDim),
        ctx.palette.get(UiColor::Black),
    );
    print_centred(buf, y, &line, dim, black);
}

/// `label` and `path` on one line of at most `width` cells as `widths`
/// measure them, the path losing characters from its start (`…` in their
/// place) if they don't fit.
pub fn place_line(label: &str, path: &str, width: usize, widths: &CellWidths) -> String {
    let room = width.saturating_sub(text_width(label, widths) + 1);
    if text_width(path, widths) <= room {
        return format!("{label} {path}");
    }
    let reversed: String = path.chars().rev().collect();
    let tail: String = clip(&reversed, room.saturating_sub(1), widths)
        .chars()
        .rev()
        .collect();
    format!("{label} …{tail}")
}
