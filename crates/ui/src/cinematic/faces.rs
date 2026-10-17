//! The characters shot (ticket 0818): faces in the dialogue screen's
//! portrait frames, each with its name under it, crossing the screen one
//! after another. Where they are comes from the time alone ([`first_x`]).

use crate::prelude::*;
use trpg_content::Side;
use trpg_core::{CharacterId, LEAD_ID, LeadProfile};

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::Ctx;
use crate::screens::dialogue::glyph::{FRAME, FaceFrame, draw_face};

/// Columns from one face's frame to the next one's: with a face in the
/// middle of the screen, half of the one behind it has come in and half
/// of the one ahead has left (*tunable*).
pub const FACE_SPACING: i32 = 50;

/// The suffix of the names-table id of a character's short name (ticket
/// 0712): `retainer.first` is "Hollis".
const SHORT_NAME: &str = ".first";

/// One face of the shot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Face {
    /// The id of the portrait drawn (the lead's is the one of their look).
    pub portrait: String,
    /// The name under it.
    pub name: String,
}

/// The faces of `who`, in order. The lead's is `lead`'s look and name;
/// anyone else has their short name, in the player's language (their
/// whole name if the names table has no short one).
pub fn faces(ctx: &Ctx, who: &[CharacterId], lead: &LeadProfile) -> Vec<Face> {
    let (words, names) = (ctx.words(), &ctx.game.content.names);
    let face = |character: &CharacterId| {
        let id = character.0.as_str();
        let short = format!("{id}{SHORT_NAME}");
        let name = if id == LEAD_ID {
            // check-text: not a data name (the player's own)
            lead.name.clone()
        } else if names.get(&short).is_some() {
            words.name(names, &short).to_owned()
        } else {
            words.name(names, id).to_owned()
        };
        Face {
            portrait: lead.portrait_for(id).to_owned(),
            name,
        }
    };
    who.iter().map(face).collect()
}

/// The column of the first face's frame `progress` (0 to 1) through a
/// shot of `count` faces coming in from `from`, on a screen `screen_w`
/// columns wide, and the columns from each face to the one after it
/// (below 0 when they come from the left: the others follow on its left).
///
/// At 0 the first face is just off the edge it comes from; at 1 the last
/// has just left by the other. They move at one speed, in whole cells.
pub fn first_x(progress: f32, count: usize, from: Side, screen_w: i32) -> (i32, i32) {
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let behind = i32::try_from(count.saturating_sub(1)).unwrap_or(i32::MAX);
    let way = screen_w
        .saturating_add(behind.saturating_mul(FACE_SPACING))
        .saturating_add(FRAME.0);
    // A way is a few hundred columns.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let gone = (progress * way as f32).floor() as i32;
    let from_right = screen_w - gone;
    match from {
        Side::Right => (from_right, FACE_SPACING),
        // The mirror picture: the frame's left edge where its right was.
        Side::Left => (screen_w - FRAME.0 - from_right, -FACE_SPACING),
    }
}

/// Draws `faces` wearing `expression` on `buf`, the first one's frame at
/// column `x` and each next one `step` columns on, in the middle of the
/// screen's height. A face looks the way it is going. Clipped to the
/// buffer.
pub fn draw_faces(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    faces: &[Face],
    expression: &str,
    (x, step): (i32, i32),
) {
    // The frame and the name's row under it.
    let top = (i32::from(buf.height()) - (FRAME.1 + 1)) / 2;
    let columns = (0..).map(|i: i32| x.saturating_add(i.saturating_mul(step)));
    for (face, x) in faces.iter().zip(columns) {
        let frame = FaceFrame {
            at: (x, top),
            portrait: &face.portrait,
            expression,
            // check-text: not a data name (the view's own)
            name: &face.name,
            speaking: true,
            // The art looks right; coming from the right, they go left.
            mirror: step.is_positive(),
        };
        draw_face(ctx, buf, &frame);
    }
}
