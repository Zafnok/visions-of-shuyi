//! Screens any game can use, each with rows, entries or a legend the game
//! gives: the [`layout_picker`] and the [`key_bindings`] screen. Each
//! says what it shows as plain data (a view) and a skin paints it
//! (`glyph` for glyphs).

pub mod key_bindings;
pub mod layout_picker;

use crate::frame::color::Rgb;
use crate::frame::glyph_buffer::GlyphBuffer;

/// Column at which `width` cells are centred in `buf` (rounded left).
pub fn centre_x(buf: &GlyphBuffer, width: usize) -> i32 {
    let width = i32::try_from(width).unwrap_or(i32::MAX);
    (i32::from(buf.width()) - width) / 2
}

/// Prints `text` centred on row `y`.
pub fn print_centred(buf: &mut GlyphBuffer, y: i32, text: &str, fg: Rgb, bg: Rgb) {
    let x = centre_x(buf, text.chars().count());
    buf.print(x, y, text, fg, bg);
}

#[cfg(test)]
mod tests;
