//! The glyph look of the dialogue screen: paints a [`DialogueView`].
//! Everything about the look lives here (where things go, the frames, the
//! colours, how the listener is dimmed); nothing here decides what the
//! screen shows or does (ADR-0054).
//!
//! Layout (100×32): 32×16-cell portraits in 34×18 frames at `x = 1` and
//! `x = 65` from row 1, name plates on row 19, the text box on rows 21–27.
//! While the lead's replies are up, the text box grows upward to list them
//! under the line being answered (Nick, 0708: like Stardew Valley).

use trpg_content::Side;

use super::log;
use super::view::{DialogueView, FaceView, TextView, WaitView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;
use crate::widgets::menu::MenuView;

/// Top row of the portrait frames.
const FRAME_Y: i32 = 1;
/// Portrait frame size: a 32×16-cell portrait plus its border.
pub const FRAME: (i32, i32) = (34, 18);
/// Left column of the left frame.
const LEFT_X: i32 = 1;
/// Left column of the right frame.
const RIGHT_X: i32 = 65;
/// The text box.
const TEXT_BOX: Rect = Rect::new(0, 21, 100, 7);
/// Left column of the text.
const TEXT_X: i32 = 4;
/// First text row.
const TEXT_Y: i32 = 23;
/// How much the listener (and both portraits during narration) is dimmed.
pub const LISTENER_DIM: f32 = 0.45;
/// Cells between the log's key hint and the Confirm key's name.
const LOG_HINT_GAP: i32 = 3;

/// Paints `view`: the log over the whole of `buf` while it is open;
/// otherwise the caption, portraits and text box, over the whole of `buf`
/// or, for an overlay, leaving the screen below showing around them.
pub fn paint(ctx: &Ctx, view: &DialogueView, buf: &mut GlyphBuffer) {
    if let Some(log) = &view.log {
        log::glyph::paint(ctx, log, buf);
        return;
    }
    let c = |u| ctx.palette.get(u);
    let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
    if view.overlay {
        // Below the text box is the battle's key help, whose keys do
        // nothing while the scene plays.
        // (Clipped to the buffer.)
        let below = TEXT_BOX.y + TEXT_BOX.h;
        let rest = Rect::new(0, below, i32::from(buf.width()), i32::from(buf.height()));
        buf.fill_rect(rest, blank);
    } else {
        buf.fill_rect(buf.bounds(), blank);
    }
    paint_stage(ctx, buf, view);
    paint_text_box(ctx, buf, view);
}

/// The caption and both sides' portraits.
fn paint_stage(ctx: &Ctx, buf: &mut GlyphBuffer, view: &DialogueView) {
    let c = |u| ctx.palette.get(u);
    if let Some(caption) = &view.caption {
        let text = format!(" {caption} ");
        print_centred(buf, 0, &text, c(UiColor::TextHighlight), c(UiColor::Black));
    }
    for side in [Side::Left, Side::Right] {
        if let Some(face) = view.face(side) {
            paint_side(ctx, buf, side, face, view.speaker == Some(side));
        }
    }
}

/// One side's frame, portrait and name plate.
fn paint_side(ctx: &Ctx, buf: &mut GlyphBuffer, side: Side, face: &FaceView, speaking: bool) {
    let x = match side {
        Side::Left => LEFT_X,
        Side::Right => RIGHT_X,
    };
    let frame = FaceFrame {
        at: (x, FRAME_Y),
        portrait: &face.portrait,
        expression: &face.expression,
        // check-text: not a data name (the view's own)
        name: &face.name,
        speaking,
        mirror: side == Side::Right,
    };
    draw_face(ctx, buf, &frame);
}

/// A portrait in its frame with a name plate under it.
#[derive(Debug, Clone, Copy)]
pub struct FaceFrame<'a> {
    /// The frame's top-left cell.
    pub at: (i32, i32),
    /// The portrait's id (an empty frame if the content has none).
    pub portrait: &'a str,
    /// Its expression.
    pub expression: &'a str,
    /// The name on the plate.
    pub name: &'a str,
    /// Lit, with a double border, as whoever speaks is; else dimmed.
    pub speaking: bool,
    /// Whether the portrait is flipped left to right.
    pub mirror: bool,
}

/// Draws `face`: a [`FRAME`] with its portrait, and the name centred on
/// the row under it. Clipped to the buffer.
pub fn draw_face(ctx: &Ctx, buf: &mut GlyphBuffer, face: &FaceFrame) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (x, y) = face.at;
    let frame = Rect::new(x, y, FRAME.0, FRAME.1);
    buf.fill_rect(frame, Cell::new(' ', c(UiColor::Text), bg));
    let (style, border, dim, name_fg) = if face.speaking {
        (
            BoxStyle::Double,
            UiColor::PanelBorderFocus,
            0.0,
            UiColor::Text,
        )
    } else {
        (
            BoxStyle::Single,
            UiColor::PanelBorder,
            LISTENER_DIM,
            UiColor::TextDim,
        )
    };
    buf.draw_box(frame, style, c(border), bg);
    if let Some(art) = ctx.content.portraits.get(face.portrait) {
        let at = (x + 1, y + 1);
        draw_portrait(buf, at, art, face.expression, dim, face.mirror);
    }
    let plate = Rect::new(x, y + FRAME.1, FRAME.0, 1);
    buf.fill_rect(plate, Cell::new(' ', c(UiColor::Text), bg));
    // check-text: not a data name (the view's own)
    let name = face.name;
    let w = i32::try_from(name.chars().count()).unwrap_or(0);
    buf.print(x + (FRAME.0 - w) / 2, plate.y, name, c(name_fg), bg);
}

/// The text box and its first text row. While the replies are up it grows
/// upward (its bottom stays put) to fit the line being answered, a blank
/// row and one row per reply.
fn text_box(text: &TextView) -> (Rect, i32) {
    let Some(choices) = &text.choices else {
        return (TEXT_BOX, TEXT_Y);
    };
    let question = text.lines.len();
    let gap = usize::from(question > 0);
    let rows = i32::try_from(question + gap + choices.items.len()).unwrap_or(0);
    // The last content row is the one the ▼ uses, free during a choice.
    let bottom = TEXT_BOX.y + TEXT_BOX.h - 1;
    let text_y = (bottom - rows).min(TEXT_Y);
    let top = text_y - (TEXT_Y - TEXT_BOX.y);
    (
        Rect::new(TEXT_BOX.x, top, TEXT_BOX.w, bottom + 1 - top),
        text_y,
    )
}

/// The text box: speaker name on the border, the revealed text (or the
/// skip question), the replies while a choice is open, and the blinking
/// `▼` while the page waits.
fn paint_text_box(ctx: &Ctx, buf: &mut GlyphBuffer, view: &DialogueView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let text = &view.text;
    let (text_box, text_y) = text_box(text);
    paint_box_frame(ctx, buf, view.speaking(), text_box);

    if let Some(skip) = &text.skip {
        buf.print(TEXT_X, TEXT_Y, &skip.question, c(UiColor::Text), bg);
        buf.print(TEXT_X, TEXT_Y + 1, &skip.answers, c(UiColor::TextDim), bg);
        return;
    }

    // Narration isn't centred vertically above replies, which follow
    // right after.
    let alone = text.choices.is_none();
    paint_lines(ctx, buf, text, text_y, alone);
    if let Some(choices) = &text.choices {
        let gap = i32::from(!text.lines.is_empty());
        let first = text_y + i32::try_from(text.lines.len()).unwrap_or(0) + gap;
        paint_choices(ctx, buf, choices, first);
    }
    if let Some(waiting) = &text.waiting {
        paint_waiting(ctx, buf, waiting);
    }
}

/// The text box `text_box`, empty, with `speaker`'s name on its border.
fn paint_box_frame(ctx: &Ctx, buf: &mut GlyphBuffer, speaker: Option<&FaceView>, text_box: Rect) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(text_box, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(text_box, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    if let Some(speaker) = speaker {
        // check-text: not a data name (the view's own)
        let name = format!(" {} ", speaker.name);
        let fg = c(UiColor::TextHighlight);
        buf.print(text_box.x + 3, text_box.y, &name, fg, bg);
    }
}

/// The revealed part of the page from row `text_y`. Narration is dim and
/// centred; vertically too when `alone` (nothing follows it in the box):
/// one line goes in the middle of the three rows (two or three fill them
/// from the top).
fn paint_lines(ctx: &Ctx, buf: &mut GlyphBuffer, text: &TextView, text_y: i32, alone: bool) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let narration = text.narration;
    let fg = if narration {
        c(UiColor::TextDim)
    } else {
        c(UiColor::Text)
    };
    let top = text_y + i32::from(narration && text.lines.len() == 1 && alone);
    for ((y, line), visible) in (top..).zip(&text.lines).zip(text.revealed()) {
        let x = if narration {
            let len = i32::try_from(line.chars().count()).unwrap_or(0);
            TEXT_BOX.x + (TEXT_BOX.w - len) / 2
        } else {
            TEXT_X
        };
        buf.print(x, y, &visible, fg, bg);
    }
}

/// The replies from row `first`, the focused one marked and bright.
fn paint_choices(ctx: &Ctx, buf: &mut GlyphBuffer, choices: &MenuView, first: i32) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    for (y, item) in (first..).zip(&choices.items) {
        let (marker, fg) = if item.focused {
            ("> ", c(UiColor::TextHighlight))
        } else {
            ("  ", c(UiColor::Text))
        };
        buf.print(TEXT_X, y, &format!("{marker}{}", item.label), fg, bg);
    }
}

/// At the right of the text box's last row: the log's hint, the key that
/// goes on, and the `▼` while it is lit.
fn paint_waiting(ctx: &Ctx, buf: &mut GlyphBuffer, waiting: &WaitView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let right = TEXT_BOX.x + TEXT_BOX.w - 4;
    let bottom = TEXT_BOX.y + TEXT_BOX.h - 2;
    let key = &waiting.key;
    let w = i32::try_from(key.chars().count()).unwrap_or(0);
    buf.print(right - 1 - w, bottom, key, c(UiColor::TextDim), bg);
    // The log's key, if it has one, before it.
    if let Some(hint) = &waiting.log_hint {
        let hint_w = i32::try_from(hint.chars().count()).unwrap_or(0);
        let x = right - 1 - w - LOG_HINT_GAP - hint_w;
        buf.print(x, bottom, hint, c(UiColor::TextDim), bg);
    }
    if waiting.marker {
        buf.print(right, bottom, "▼", c(UiColor::TextHighlight), bg);
    }
}

#[cfg(test)]
mod tests;
