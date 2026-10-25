//! The glyph look of the dialogue screen: paints a [`DialogueView`].
//! Everything about the look lives here (where things go, the frames, the
//! colours, how the listener is dimmed); nothing here decides what the
//! screen shows or does (ADR-0054).
//!
//! The scene's place, if it names one, is painted first by the map skin
//! over the whole screen and darkened to [`PLACE_BRIGHTNESS`]; an overlay
//! with no place darkens the screen below it (the battle map) the same
//! way. Everything else goes on top at full brightness. While an opening
//! plays (0734) the place goes from full brightness down to
//! [`PLACE_BRIGHTNESS`] as it darkens, under the caption alone.
//!
//! Layout (100×32): 32×16-cell portraits in 34×18 frames at `x = 1` and
//! `x = 65` from row 1, name plates on row 19, the text box on rows 21–27.
//! While the lead's replies are up, the text box grows upward to list them
//! under the line being answered (Nick, 0708: like Stardew Valley).

use trpg_content::Side;

use super::log;
use super::view::{DialogueView, FaceView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;
use shuyi_ui::dialogue::text_box::{
    self,
    glyph::{BoxPlace, NarrationAlign},
};

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
/// How bright the place behind the talk is, and the battle map behind a
/// battle's scene: about 40% (`docs/design/look-and-feel.md`, *Places
/// behind conversations*, a little darker than the dimmed listener's
/// portrait so the faces stand out; *tunable*).
pub const PLACE_BRIGHTNESS: f32 = 0.4;

/// Paints `view`: the log over the whole of `buf` while it is open;
/// otherwise the caption, portraits and text box over the scene's place,
/// darkened, or over black; or, for an overlay with no place, leaving the
/// screen below showing around them, darkened.
pub fn paint(ctx: &Ctx, view: &DialogueView, buf: &mut GlyphBuffer) {
    if let Some(log) = &view.log {
        log::glyph::paint(ctx, log, buf);
        return;
    }
    let c = |u| ctx.palette.get(u);
    let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
    if let Some(opening) = &view.opening {
        buf.fill_rect(buf.bounds(), blank);
        if let Some(place) = &view.place {
            ctx.game.map_skin.paint(ctx, place, buf.bounds(), buf);
        }
        let darkened = opening.darkened.clamp(0.0, 1.0);
        buf.fade(1.0 + (PLACE_BRIGHTNESS - 1.0) * darkened);
        paint_caption(ctx, buf, view);
        return;
    }
    if let Some(place) = &view.place {
        buf.fill_rect(buf.bounds(), blank);
        ctx.game.map_skin.paint(ctx, place, buf.bounds(), buf);
        buf.fade(PLACE_BRIGHTNESS);
    } else if view.overlay {
        buf.fade(PLACE_BRIGHTNESS);
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

/// The caption, on the top row.
fn paint_caption(ctx: &Ctx, buf: &mut GlyphBuffer, view: &DialogueView) {
    let c = |u| ctx.palette.get(u);
    if let Some(caption) = &view.caption {
        let text = format!(" {caption} ");
        print_centred(buf, 0, &text, c(UiColor::TextHighlight), c(UiColor::Black));
    }
}

/// The caption and both sides' portraits.
fn paint_stage(ctx: &Ctx, buf: &mut GlyphBuffer, view: &DialogueView) {
    paint_caption(ctx, buf, view);
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
    if let Some(art) = ctx.game.content.portraits.get(face.portrait) {
        let at = (x + 1, y + 1);
        draw_portrait(buf, at, art, face.expression, dim, face.mirror);
    }
    let plate = Rect::new(x, y + FRAME.1, FRAME.0, 1);
    buf.fill_rect(plate, Cell::new(' ', c(UiColor::Text), bg));
    // check-text: not a data name (the view's own)
    buf.print_centred(plate, face.name, c(name_fg));
}

/// Where the text box goes: rows 21–27, the text from column 4 of row 23,
/// narration from there too, like speech.
pub const BOX_PLACE: BoxPlace =
    BoxPlace::new(TEXT_BOX, TEXT_X, TEXT_Y).with_narration(NarrationAlign::Left);

/// The text box ([`shuyi_ui::dialogue::text_box::glyph::paint`]), with the
/// name of whoever speaks on its border.
fn paint_text_box(ctx: &Ctx, buf: &mut GlyphBuffer, view: &DialogueView) {
    // check-text: not a data name (the view's own)
    let speaker = view.speaking().map(|face| face.name.as_str());
    text_box::glyph::paint(ctx, buf, BOX_PLACE, &view.text, speaker);
}

#[cfg(test)]
mod tests;
