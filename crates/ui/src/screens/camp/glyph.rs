//! The glyph look of the Camp screen: paints a [`CampView`] as a centred
//! heading, the sections' menu box on the left and the list in a panel on
//! the right, under its title (or what is said instead of an empty list),
//! the help line at the bottom. Everything about the look lives here;
//! nothing here decides what the screen shows (ADR-0054).

use super::view::CampView;
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;
use crate::widgets::menu::glyph as menu_glyph;

/// Row of the heading.
const HEADING_ROW: i32 = 2;
/// Where the sections' menu box starts.
const SECTIONS: (i32, i32) = (4, 6);
/// The list's panel.
const PANEL: Rect = Rect::new(28, 5, 68, 24);

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &CampView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    print_centred(
        buf,
        HEADING_ROW,
        &view.heading,
        c(UiColor::TextHighlight),
        black,
    );
    menu_glyph::paint(ctx, &view.sections, buf, SECTIONS.0, SECTIONS.1);
    let border = if view.sections_focused {
        UiColor::TextDim
    } else {
        UiColor::PanelBorder
    };
    buf.draw_box(PANEL, BoxStyle::Single, c(border), black);
    let x = PANEL.x + 2;
    if !view.list_title.is_empty() {
        let title = format!(" {} ", view.list_title);
        buf.print(x, PANEL.y, &title, c(UiColor::TextHighlight), black);
    }
    match &view.list {
        Some(list) => menu_glyph::paint(ctx, list, buf, x, PANEL.y + 1),
        None => {
            buf.print(x, PANEL.y + 2, &view.empty, c(UiColor::TextDim), black);
        }
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, c(UiColor::TextDim), black);
}
