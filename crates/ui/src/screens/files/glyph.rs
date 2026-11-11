//! The glyph look of the place line: dim text two rows over the help line,
//! centred, the path shortened from the left (`…\tactical-rpg`) so the
//! line fits the buffer.

use super::PlaceView;
use crate::color::UiColor;
use crate::glyph_buffer::GlyphBuffer;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;

/// Cells kept clear at each end of the line.
const MARGIN: usize = 2;

/// Paints `place` two rows over the bottom (help) row of `buf`.
pub fn paint_place(ctx: &Ctx, place: &PlaceView, buf: &mut GlyphBuffer) {
    let width = usize::from(buf.width()).saturating_sub(2 * MARGIN);
    let line = place_line(&place.label, &place.path, width);
    let y = i32::from(buf.height()) - 3;
    let (dim, black) = (
        ctx.palette.get(UiColor::TextDim),
        ctx.palette.get(UiColor::Black),
    );
    print_centred(buf, y, &line, dim, black);
}

/// `label` and `path` on one line of at most `width` characters, the path
/// losing characters from its start (`…` in their place) if they don't
/// fit.
pub fn place_line(label: &str, path: &str, width: usize) -> String {
    let room = width.saturating_sub(label.chars().count() + 1);
    let count = path.chars().count();
    if count <= room {
        return format!("{label} {path}");
    }
    let keep = room.saturating_sub(1);
    let tail: String = path.chars().skip(count - keep).collect();
    format!("{label} …{tail}")
}
