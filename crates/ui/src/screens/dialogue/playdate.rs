//! The Playdate's look of the dialogue screen (ticket 1043): paints a
//! [`DialogueView`] on the 50 × 15 one-bit screen. The PC's arrangement at
//! half size (`docs/design/playdate.md`, *Portraits*): the two faces at 2×
//! along the top, whoever speaks in a double frame and the other in a
//! single one, faded; each name in the bottom of its frame; a 3-line text
//! box of [`PLAYDATE_TEXT_W`](super::PLAYDATE_TEXT_W) letters across the
//! bottom, a window of the one-bit look ([`crate::screens::playdate`]).
//!
//! - **The caption** is on the top row, across the frames' tops.
//! - **A choice** grows the box upward over the faces, the replies under
//!   the line being answered, the focused one the bar.
//! - **The key that goes on** and a blinking `▼` are in the bottom of the
//!   box's frame, the log's hint before them.
//! - **The place behind the talk** is the Playdate's map skin, darkened in
//!   one bit: its white ground turns to light dots ([`darken`]); one bit has
//!   no brightness, so a place asked to be darkened less looks the same.
//!   An opening shows it undarkened and cuts to darkened halfway through
//!   the PC's fade (as the title cinematic cuts, 1044). A battle's scene
//!   darkens the battle below it the same way.
//!
//! Nothing here decides what the screen shows or does (ADR-0054).

use trpg_content::Side;

use super::glyph::{FaceFrame, LISTENER_DIM};
use super::log;
use super::view::{DialogueView, FaceView, TextView, WaitView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{
    Item, MORE_BELOW, cells, clear, cut, inks, item, to_i32, to_usize, window,
};

/// The 2× bust, in cells (128 × 128 pixels).
const BUST: (i32, i32) = (16, 8);
/// A portrait's frame: the 2× bust and its border, as on the lead screen.
pub const FRAME: (i32, i32) = (18, 10);
/// Top row of the frames.
const FRAME_Y: i32 = 0;
/// Left column of the left frame and of the right one: a cell in from
/// each edge.
const FRAME_X: [i32; 2] = [1, 31];
/// The text box when it holds a page: under the frames, to the bottom.
pub const TEXT_BOX: Rect = Rect::new(0, 10, 50, 5);
/// Left column of the text: a cell in from the frame, so a line of 47
/// letters ends a cell before the right edge.
pub const TEXT_X: i32 = 2;
/// The rows of a page.
const PAGE_ROWS: usize = super::TEXT_LINES;
/// How far an opening's place has darkened when the Playdate cuts to its
/// darkened look: halfway.
const CUT_AT: f32 = 0.5;

/// Paints `view`: the log over the whole of `buf` while it is open;
/// otherwise the faces, the caption and the text box over the scene's
/// place, darkened, or over the white ground; or, for an overlay with no
/// place, over the screen below, darkened.
pub fn paint(ctx: &Ctx, view: &DialogueView, buf: &mut GlyphBuffer) {
    if let Some(log) = &view.log {
        log::playdate::paint(ctx, log, buf);
        return;
    }
    let darkened = view.opening.is_none_or(|o| o.darkened >= CUT_AT);
    if let Some(place) = &view.place {
        clear(ctx, buf);
        ctx.game.map_skin.paint(ctx, place, buf.bounds(), buf);
        if darkened {
            darken(ctx, buf);
        }
    } else if view.overlay {
        darken(ctx, buf);
    } else {
        clear(ctx, buf);
    }
    if view.opening.is_none() {
        for (side, x) in [Side::Left, Side::Right].into_iter().zip(FRAME_X) {
            if let Some(face) = view.face(side) {
                paint_side(ctx, buf, face, x, side, view.speaker == Some(side));
            }
        }
    }
    paint_caption(ctx, buf, view.caption.as_deref());
    if view.opening.is_none() {
        paint_text_box(ctx, buf, &view.text);
    }
}

/// The dark look of a place in one bit: every cell on the white ground
/// gets the ground of light dots (the palette's `place_ground`).
pub fn darken(ctx: &Ctx, buf: &mut GlyphBuffer) {
    let ground = inks(ctx).ground;
    let dots = ctx
        .palette
        .lookup("place_ground")
        .unwrap_or_else(|| ctx.palette.get(UiColor::MoveRange));
    for y in 0..i32::from(buf.height()) {
        for x in 0..i32::from(buf.width()) {
            if let Some(&cell) = buf.get(x, y).filter(|c| c.bg == ground) {
                buf.set(x, y, Cell { bg: dots, ..cell });
            }
        }
    }
}

/// The caption centred on the top row, on the ground.
fn paint_caption(ctx: &Ctx, buf: &mut GlyphBuffer, caption: Option<&str>) {
    if let Some(caption) = caption {
        let ink = inks(ctx);
        let text = cut(&format!(" {caption} "), usize::from(buf.width()));
        crate::screens::print_centred(buf, 0, &text, ink.text, ink.ground);
    }
}

/// One side's face in its frame at column `x`.
fn paint_side(ctx: &Ctx, buf: &mut GlyphBuffer, face: &FaceView, x: i32, side: Side, lit: bool) {
    let frame = FaceFrame {
        at: (x, FRAME_Y),
        portrait: &face.portrait,
        expression: &face.expression,
        // check-text: not a data name (the view's own)
        name: &face.name,
        speaking: lit,
        mirror: side == Side::Right,
    };
    draw_face(ctx, buf, &frame);
}

/// Draws `face` in the Playdate's look: a [`FRAME`] (double while it
/// speaks, else single with the portrait faded), the portrait at 2× in
/// it, the name centred in the bottom of the frame. Clipped to the buffer.
pub fn draw_face(ctx: &Ctx, buf: &mut GlyphBuffer, face: &FaceFrame) {
    let ink = inks(ctx);
    let (x, y) = face.at;
    let frame = Rect::new(x, y, FRAME.0, FRAME.1);
    buf.fill_rect(frame, Cell::new(' ', ink.text, ink.inside));
    let (style, dim) = if face.speaking {
        (BoxStyle::Double, 0.0)
    } else {
        (BoxStyle::Single, LISTENER_DIM)
    };
    buf.draw_box(frame, style, ink.frame, ink.inside);
    if let Some(art) = ctx.game.content.portraits.get(face.portrait) {
        let at = (x + 1, y + 1);
        draw_portrait_in(buf, at, BUST, art, face.expression, dim, face.mirror);
    }
    // check-text: not a data name (the view's own)
    let name = cut(&format!(" {} ", face.name), to_usize(FRAME.0 - 2));
    let name_x = x + (FRAME.0 - cells(&name)) / 2;
    buf.print(name_x, y + FRAME.1 - 1, &name, ink.text, ink.inside);
}

/// The box `text` is painted in, and its first text row: [`TEXT_BOX`],
/// grown upward (its bottom stays put) to fit more lines than a page (a
/// talk shot's), or the line being answered, a blank row and a row per
/// reply.
pub fn box_rect(text: &TextView) -> (Rect, i32) {
    let lines = text.lines.len();
    let rows = text.choices.as_ref().map_or(lines, |choices| {
        lines + usize::from(lines > 0) + choices.items.len()
    });
    let rows = to_i32(rows.max(PAGE_ROWS));
    let bottom = TEXT_BOX.y + TEXT_BOX.h;
    let top = (bottom - rows - 2).max(0);
    (
        Rect::new(TEXT_BOX.x, top, TEXT_BOX.w, bottom - top),
        top + 1,
    )
}

/// The text box: the skip question, or the page as revealed so far with
/// the replies under it while a choice is open; while the page waits,
/// the key that goes on in the bottom of its frame.
fn paint_text_box(ctx: &Ctx, buf: &mut GlyphBuffer, text: &TextView) {
    let ink = inks(ctx);
    let (frame, text_y) = box_rect(text);
    window(ctx, buf, frame, "");
    if let Some(skip) = &text.skip {
        buf.print(TEXT_X, text_y, &skip.question, ink.text, ink.inside);
        buf.print(TEXT_X, text_y + 1, &skip.answers, ink.text, ink.inside);
        return;
    }
    let lines = (text_y..).zip(text.revealed());
    for (y, line) in lines {
        buf.print(TEXT_X, y, &line, ink.text, ink.inside);
    }
    if let Some(choices) = &text.choices {
        let gap = i32::from(!text.lines.is_empty());
        let first = text_y + to_i32(text.lines.len()) + gap;
        let w = frame.w - 2 * TEXT_X;
        for (y, reply) in (first..).zip(&choices.items) {
            let label = ctx.label(&reply.label);
            let state = Item::of(reply.focused, reply.enabled);
            item(ctx, buf, (TEXT_X, y), w, &label, state);
        }
    }
    if let Some(waiting) = &text.waiting {
        paint_waiting(ctx, buf, frame, waiting);
    }
}

/// At the right of the bottom of the box's frame: the log's hint, the
/// key that goes on, and the `▼` while it is lit.
fn paint_waiting(ctx: &Ctx, buf: &mut GlyphBuffer, frame: Rect, waiting: &WaitView) {
    let ink = inks(ctx);
    let marker = if waiting.marker { MORE_BELOW } else { " " };
    let mut shown = format!(" {} {marker} ", waiting.key);
    if let Some(hint) = &waiting.log_hint {
        shown = format!(" {hint} ·{shown}");
    }
    let x = frame.x + frame.w - 2 - cells(&shown);
    buf.print(x, frame.y + frame.h - 1, &shown, ink.text, ink.inside);
}

#[cfg(test)]
mod tests;
