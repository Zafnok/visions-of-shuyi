//! The glyph look of the credits screen: one single-bordered panel with
//! its title on the top border, the rows of the list indented by kind
//! (headings, entries, details) and a mark at the right where the list goes
//! on. Everything about the look lives here; nothing here decides what the
//! screen shows. Another look is another `paint` of the same view
//! (ADR-0054).

use super::view::{CreditRowKind, CreditsView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;

/// The panel, in cells.
const PANEL: Rect = Rect::new(2, 1, 96, 29);
/// Column of the group headings: one blank cell in from the panel's left
/// border (a test pins it to the panel).
const HEADING_X: i32 = 4;
/// Column of an entry's first line, two cells right of the headings.
const ENTRY_X: i32 = HEADING_X + 2;
/// Column of an entry's license and link, two cells right of its title.
const DETAIL_X: i32 = ENTRY_X + 2;
/// First row of the list: a blank row under the border.
const LIST_Y: i32 = PANEL.y + 2;
/// Last column text may use; the scroll marks' column, one further right,
/// stays clear.
const TEXT_END_X: i32 = PANEL.x + PANEL.w - 4;
/// Rows of the list the panel has room for.
const ROWS: i32 = 25;
/// Shown at the list's top right when there is more above.
const MORE_ABOVE: char = '▲';
/// Shown at the list's bottom right when there is more below.
const MORE_BELOW: char = '▼';

/// The column a row of `kind` starts at.
fn x(kind: CreditRowKind) -> i32 {
    match kind {
        CreditRowKind::Heading => HEADING_X,
        CreditRowKind::Entry => ENTRY_X,
        CreditRowKind::Detail => DETAIL_X,
    }
}

/// Paints `view` over the whole of `buf`. The panel has room for
/// 25 rows of the list; the screen gives no more.
pub fn paint(ctx: &Ctx, view: &CreditsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, bg) = (c(UiColor::Black), c(UiColor::PanelBg));
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    buf.fill_rect(PANEL, Cell::new(' ', text, bg));
    buf.draw_box(PANEL, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    buf.print(HEADING_X, PANEL.y, &title, c(UiColor::TextHighlight), bg);

    for (y, row) in (LIST_Y..).zip(&view.rows) {
        let fg = match row.kind {
            CreditRowKind::Heading => c(UiColor::PanelBorderFocus),
            CreditRowKind::Entry => text,
            CreditRowKind::Detail => dim,
        };
        buf.print(x(row.kind), y, &row.text, fg, bg);
    }
    let last_y = LIST_Y + ROWS - 1;
    let mark_x = TEXT_END_X + 1;
    if view.more_above {
        buf.print(mark_x, LIST_Y, &MORE_ABOVE.to_string(), dim, bg);
    }
    if view.more_below {
        buf.print(mark_x, last_y, &MORE_BELOW.to_string(), dim, bg);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, dim, black);
}

#[cfg(test)]
mod tests;
