//! The characters shot (ticket 0818): faces in the dialogue screen's
//! portrait frames, each with its name under it, crossing the screen one
//! after another; on the Playdate in its dialogue frames (ticket 1043,
//! [`FaceLook`]). Where they are comes from the time alone ([`first_x`]).

use crate::prelude::*;
use trpg_content::Side;
use trpg_core::{CharacterId, LEAD_ID, LeadProfile};

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, SkinSet};
use crate::screens::dialogue::glyph::{self, FaceFrame};
use crate::screens::dialogue::playdate;

/// Columns from one face's frame to the next one's on the PC: with a face
/// in the middle of the screen, half of the one behind it has come in and
/// half of the one ahead has left (*tunable*).
pub const FACE_SPACING: i32 = 50;

/// How the shot's faces look with a skin set: the dialogue screen's frame
/// they are in, how far apart they cross, and what draws one.
#[derive(Clone, Copy)]
pub struct FaceLook {
    /// A face's frame, in cells.
    pub frame: (i32, i32),
    /// Columns from one face's frame to the next one's.
    pub spacing: i32,
    /// Draws a face in its frame.
    pub draw: fn(&Ctx, &mut GlyphBuffer, &FaceFrame),
}

impl FaceLook {
    /// The PC's: the dialogue screen's glyph frames.
    pub const GLYPH: Self = Self {
        frame: glyph::FRAME,
        spacing: FACE_SPACING,
        draw: glyph::draw_face,
    };
    /// The Playdate's: its dialogue frames, half as far apart on its
    /// screen half as wide.
    pub const PLAYDATE: Self = Self {
        frame: playdate::FRAME,
        spacing: FACE_SPACING / 2,
        draw: playdate::draw_face,
    };

    /// The look of `ctx`'s skin set.
    pub fn of(ctx: &Ctx) -> Self {
        match ctx.game.skin_set {
            SkinSet::Glyph => Self::GLYPH,
            SkinSet::Playdate => Self::PLAYDATE,
        }
    }
}

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
/// shot of `count` faces in `look` coming in from `from`, on a screen
/// `screen_w` columns wide, and the columns from each face to the one
/// after it (below 0 when they come from the left: the others follow on
/// its left).
///
/// At 0 the first face is just off the edge it comes from; at 1 the last
/// has just left by the other. They move at one speed, in whole cells.
pub fn first_x(
    progress: f32,
    count: usize,
    from: Side,
    screen_w: i32,
    look: FaceLook,
) -> (i32, i32) {
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let behind = i32::try_from(count.saturating_sub(1)).unwrap_or(i32::MAX);
    let way = screen_w
        .saturating_add(behind.saturating_mul(look.spacing))
        .saturating_add(look.frame.0);
    // A way is a few hundred columns.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let gone = (progress * way as f32).floor() as i32;
    let from_right = screen_w - gone;
    match from {
        Side::Right => (from_right, look.spacing),
        // The mirror picture: the frame's left edge where its right was.
        Side::Left => (screen_w - look.frame.0 - from_right, -look.spacing),
    }
}

/// Draws `faces` wearing `expression` on `buf` in `ctx`'s [`FaceLook`],
/// the first one's frame at column `x` and each next one `step` columns
/// on, in the middle of the screen's height. A face looks the way it is
/// going. Clipped to the buffer.
pub fn draw_faces(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    faces: &[Face],
    expression: &str,
    (x, step): (i32, i32),
) {
    let look = FaceLook::of(ctx);
    // The frame and the name's row under it.
    let top = (i32::from(buf.height()) - (look.frame.1 + 1)) / 2;
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
        (look.draw)(ctx, buf, &frame);
    }
}
